//! TerminusDB-backed persistence for the knowledge platform (SPEC-006).
//!
//! Design: scoped property sets are stored as **immutable versioned
//! documents** — one document per (entity, instance, version). Each save is a
//! TerminusDB commit carrying the acting principal as `author` and a message
//! of the form `{user-message}|ps:{entity}:{instance}:v{version}`. Because
//! version documents never change, their content is valid "as of" any commit
//! after their creation — time-travel is reconstructed from the commit log.

#![recursion_limit = "512"]

use std::collections::HashMap;

use data_graph::{EntityKind, PropertySet, Scope, Status};
use terminusdb_client::{
    BranchSpec, DocumentInsertArgs, LogOpts, TerminusDBHttpClient, TerminusDBModel,
};
use terminusdb_schema::{EntityIDFor, ToTDBInstance};
use terminusdb_schema_derive::{FromTDBInstance, TerminusDBModel};
use uuid::Uuid;

/// Sentinel instance id for the global `common` truth layer (mirrors
/// data-graph's COMMON_INSTANCE).
pub const COMMON_INSTANCE: Uuid = Uuid::nil();

#[derive(Clone, Debug, PartialEq, TerminusDBModel, FromTDBInstance)]
#[tdb(id_field = "id", key = "random")]
pub struct EntityDoc {
    pub id: EntityIDFor<Self>,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, TerminusDBModel, FromTDBInstance)]
#[tdb(id_field = "id", key = "random")]
pub struct PropertySetDoc {
    id: EntityIDFor<Self>,
    pub entity_id: String,
    pub instance_id: String,
    pub scope: String,
    pub version: i64,
    pub status: String,
    /// serialized property map (sys:JSON)
    pub properties: serde_json::Value,
}

impl PropertySetDoc {
    pub fn to_property_set(&self) -> PropertySet {
        let scope = match self.scope.as_str() {
            "Common" => Scope::Common,
            "Org" => Scope::Org,
            "Workspace" => Scope::Workspace,
            _ => Scope::Project,
        };
        let status = match self.status.as_str() {
            "Retired" => Status::Retired,
            "Candidate" => Status::Candidate,
            _ => Status::Current,
        };
        let properties = self
            .properties
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<String, _>>()
            })
            .unwrap_or_default();
        PropertySet {
            scope,
            version: self.version as u64,
            status,
            properties,
        }
    }
}

/// Commit-log entry surface used by `resolve_at`.
#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub identifier: String,
    pub author: String,
    pub message: String,
}

pub mod audit;
pub mod stream;

#[derive(Clone)]
pub struct Repository {
    client: TerminusDBHttpClient,
    db: String,
    spec: BranchSpec,
}

impl Repository {
    pub async fn new(client: TerminusDBHttpClient, db: String) -> anyhow::Result<Self> {
        client.ensure_database(&db).await?;
        let args = DocumentInsertArgs::from(BranchSpec::from(db.as_str()));
        client.schema::<EntityDoc>(args.clone()).await?;
        client.schema::<PropertySetDoc>(args.clone()).await?;
        client
            .schema::<crate::audit::ChangeRequestDoc>(args.clone())
            .await?;
        client.schema::<crate::audit::DecisionDoc>(args).await?;
        Ok(Self {
            client,
            spec: BranchSpec::from(db.as_str()),
            db,
        })
    }

    pub fn client(&self) -> &TerminusDBHttpClient {
        &self.client
    }

    pub fn db(&self) -> &str {
        &self.db
    }

    pub async fn create_entity(&self, kind: EntityKind) -> anyhow::Result<Uuid> {
        self.create_entity_as(kind, "system").await
    }

    /// Create an entity recording the acting principal as commit author.
    pub async fn create_entity_as(&self, kind: EntityKind, author: &str) -> anyhow::Result<Uuid> {
        let id = Uuid::now_v7();
        let doc = EntityDoc {
            id: EntityIDFor::new(&format!("E:{id}"))?,
            kind: format!("{kind:?}"),
        };
        let mut args = DocumentInsertArgs::from(self.spec.clone());
        args.author = author.to_string();
        args.message = format!("create-entity|ent:{id}:{:?}", kind);
        self.client.insert(&doc, args).await?;
        Ok(id)
    }

    pub async fn save_property_set(
        &self,
        entity_id: Uuid,
        instance_id: Uuid,
        set: &PropertySet,
        author: &str,
        message: &str,
    ) -> anyhow::Result<()> {
        let version = set.version;
        let scope = format!("{:?}", set.scope);
        let status = format!("{:?}", set.status);
        let properties = serde_json::to_value(&set.properties)?;
        let bare_id = format!("PS:{entity_id}:{instance_id}:v{version}");
        let doc = PropertySetDoc {
            id: EntityIDFor::new(bare_id.as_str())?,
            entity_id: entity_id.to_string(),
            instance_id: instance_id.to_string(),
            scope: scope.clone(),
            version: version as i64,
            status,
            properties,
        };
        let mut args = DocumentInsertArgs::from(self.spec.clone());
        args.author = author.to_string();
        args.message = format!("{message}|ps:{entity_id}:{instance_id}:{scope}:v{version}");
        self.client.insert(&doc, args).await?;
        Ok(())
    }

    pub async fn load_property_sets(&self, entity_id: Uuid) -> anyhow::Result<Vec<PropertySetDoc>> {
        let docs = self
            .client
            .get_documents(
                vec![],
                &self.spec,
                terminusdb_client::GetOpts {
                    unfold: true,
                    type_filter: Some("PropertySetDoc".to_string()),
                    ..Default::default()
                },
            )
            .await?;
        let entity = entity_id.to_string();
        let mut out = Vec::new();
        for doc in docs {
            if doc["entity_id"].as_str() == Some(entity.as_str()) {
                let psd: PropertySetDoc = serde_json::from_value(doc)?;
                out.push(psd);
            }
        }
        Ok(out)
    }

    pub async fn latest_commit(&self) -> anyhow::Result<String> {
        Ok(self
            .client
            .get_latest_commit_id(&self.spec)
            .await?
            .to_string())
    }

    /// Commit-log entries that touched the given entity (message carries the
    /// `ps:{entity}:{instance}:v{version}` token).
    pub async fn commit_log_for_entity(&self, entity_id: Uuid) -> anyhow::Result<Vec<CommitEntry>> {
        let log = self.client.log(&self.spec, LogOpts::default()).await?;
        let needle = format!("ps:{entity_id}:");
        Ok(log
            .into_iter()
            .filter(|e| e.message.contains(&needle))
            .map(|e| CommitEntry {
                identifier: e.identifier,
                author: e.author,
                message: e.message,
            })
            .collect())
    }

    pub async fn log(&self) -> anyhow::Result<Vec<terminusdb_client::LogEntry>> {
        self.client.log(&self.spec, LogOpts::default()).await
    }
}

/// Resolve the effective property set for `entity_id` at a scope chain as of
/// `commit` (inclusive). Walks the chain nearest-first; returns the highest
/// version whose save-commit precedes or equals `commit`.
pub async fn resolve_at(
    repo: &Repository,
    entity_id: Uuid,
    chain: &[Uuid],
    commit: &str,
) -> anyhow::Result<PropertySet> {
    let log = repo.log().await?;
    // newest-first: collect ps tokens from the target commit (inclusive) and older
    let mut collecting = false;
    let mut versions: HashMap<Uuid, (u64, String)> = HashMap::new();
    for entry in &log {
        if entry.identifier == *commit {
            collecting = true;
        }
        if !collecting {
            continue;
        }
        if let Some((_, ps)) = entry.message.rsplit_once("|ps:") {
            let parts: Vec<&str> = ps.split(':').collect();
            let (instance_str, version_str) = if parts.len() >= 4 {
                (parts[1], parts[3])
            } else {
                (
                    parts.get(1).copied().unwrap_or(""),
                    parts.get(2).copied().unwrap_or(""),
                )
            };
            if parts[0] == entity_id.to_string() {
                let instance = instance_str.parse::<Uuid>()?;
                let version = version_str.trim_start_matches('v').parse::<u64>()?;
                let bare_id = format!("PS:{entity_id}:{instance}:v{version}");
                versions
                    .entry(instance)
                    .and_modify(|(v, id)| {
                        if version > *v {
                            *v = version;
                            *id = bare_id.clone();
                        }
                    })
                    .or_insert((version, bare_id));
            }
        }
    }
    for instance in chain {
        if let Some((version, bare_id)) = versions.get(instance) {
            let doc = repo
                .client
                .get_document(
                    &format!("PropertySetDoc/{bare_id}"),
                    &repo.spec,
                    terminusdb_client::GetOpts {
                        unfold: true,
                        ..Default::default()
                    },
                )
                .await?;
            let psd: PropertySetDoc = serde_json::from_value(doc)?;
            if psd.version as u64 == *version {
                return Ok(psd.to_property_set());
            }
        }
    }
    anyhow::bail!("no property set as of commit {commit} for entity {entity_id}")
}
