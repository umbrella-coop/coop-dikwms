//! Platform-wide audit machinery (SPEC-012).
//!
//! Persisted moderation ledger (ChangeRequestDoc/DecisionDoc, immutable +
//! append-only), correlation ids on writes (`|corr:{id}` tokens), auditable
//! reverts (`|rev:{target}` compensating commits), and audit projections
//! over the commit stream + ledger docs.

use terminusdb_schema::ToTDBInstance;
use uuid::Uuid;

use crate::stream::{self, CommitCursor};
use crate::{Repository, resolve_at};
use knowledge_domain::PropertySet;

#[derive(
    Clone,
    Debug,
    PartialEq,
    terminusdb_schema_derive::TerminusDBModel,
    terminusdb_schema_derive::FromTDBInstance,
)]
#[tdb(id_field = "id", key = "random")]
pub struct ChangeRequestDoc {
    id: terminusdb_schema::EntityIDFor<Self>,
    pub request_id: String,
    pub entity_id: String,
    pub instance_id: String,
    pub scope: String,
    pub proposed_version: i64,
    pub correlation_id: Option<String>,
    pub created_by: String,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    terminusdb_schema_derive::TerminusDBModel,
    terminusdb_schema_derive::FromTDBInstance,
)]
#[tdb(id_field = "id", key = "random")]
pub struct DecisionDoc {
    id: terminusdb_schema::EntityIDFor<Self>,
    pub request_id: String,
    pub decided_by: String,
    pub approve: bool,
    pub decided_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    EntityCreated,
    PropertySetSaved,
    Reverted,
    RequestSubmitted,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEntry {
    pub commit: String,
    pub actor: String,
    pub action: AuditAction,
    pub entity_id: Uuid,
    pub correlation_id: Option<String>,
    pub details: String,
}

impl Repository {
    // ------------------------------------------------------------------
    // Persisted moderation ledger
    // ------------------------------------------------------------------

    pub async fn save_change_request(
        &self,
        entity_id: Uuid,
        instance_id: Uuid,
        scope: &str,
        proposed_version: u64,
        created_by: &str,
        correlation_id: Option<&str>,
    ) -> anyhow::Result<Uuid> {
        let request_id = Uuid::now_v7();
        let bare_id = format!("CR:{request_id}");
        let doc = ChangeRequestDoc {
            id: terminusdb_schema::EntityIDFor::new(bare_id.as_str())?,
            request_id: request_id.to_string(),
            entity_id: entity_id.to_string(),
            instance_id: instance_id.to_string(),
            scope: scope.to_string(),
            proposed_version: proposed_version as i64,
            correlation_id: correlation_id.map(String::from),
            created_by: created_by.to_string(),
        };
        let mut args = terminusdb_client::DocumentInsertArgs::from(self.spec.clone());
        args.author = created_by.to_string();
        args.message = format!("request|req:{request_id}:{entity_id}:submitted");
        self.client.insert(&doc, args).await?;
        Ok(request_id)
    }

    pub async fn save_decision(
        &self,
        request_id: Uuid,
        entity_id: Uuid,
        decided_by: &str,
        approve: bool,
    ) -> anyhow::Result<()> {
        let bare_id = format!("DC:{request_id}:{}", Uuid::now_v7());
        let doc = DecisionDoc {
            id: terminusdb_schema::EntityIDFor::new(bare_id.as_str())?,
            request_id: request_id.to_string(),
            decided_by: decided_by.to_string(),
            approve,
            decided_at: chrono::Utc::now().timestamp(),
        };
        let mut args = terminusdb_client::DocumentInsertArgs::from(self.spec.clone());
        args.author = decided_by.to_string();
        args.message = format!("decision|req:{request_id}:{entity_id}:{approve}");
        self.client.insert(&doc, args).await?;
        Ok(())
    }

    /// All change requests with their decisions (any entity) — API surface.
    pub async fn requests_for_entity_any(
        &self,
    ) -> anyhow::Result<Vec<(ChangeRequestDoc, Vec<DecisionDoc>)>> {
        let docs = self
            .client
            .get_documents(
                vec![],
                &self.spec,
                terminusdb_client::GetOpts {
                    unfold: true,
                    type_filter: Some("ChangeRequestDoc".to_string()),
                    ..Default::default()
                },
            )
            .await?;
        let mut requests: Vec<ChangeRequestDoc> = Vec::new();
        for d in docs {
            requests.push(serde_json::from_value(d)?);
        }
        let decisions = self
            .client
            .get_documents(
                vec![],
                &self.spec,
                terminusdb_client::GetOpts {
                    unfold: true,
                    type_filter: Some("DecisionDoc".to_string()),
                    ..Default::default()
                },
            )
            .await?;
        let mut parsed: Vec<DecisionDoc> = Vec::new();
        for d in decisions {
            parsed.push(serde_json::from_value(d)?);
        }
        let mut out = Vec::new();
        for req in requests {
            let req_decisions = parsed
                .iter()
                .filter(|d| d.request_id == req.request_id)
                .cloned()
                .collect();
            out.push((req, req_decisions));
        }
        Ok(out)
    }

    pub async fn requests_for_entity(
        &self,
        entity_id: Uuid,
    ) -> anyhow::Result<Vec<(ChangeRequestDoc, Vec<DecisionDoc>)>> {
        let docs = self
            .client
            .get_documents(
                vec![],
                &self.spec,
                terminusdb_client::GetOpts {
                    unfold: true,
                    type_filter: Some("ChangeRequestDoc".to_string()),
                    ..Default::default()
                },
            )
            .await?;
        let entity = entity_id.to_string();
        let mut requests: Vec<ChangeRequestDoc> = Vec::new();
        for d in docs {
            if d["entity_id"].as_str() == Some(entity.as_str()) {
                requests.push(serde_json::from_value(d)?);
            }
        }
        let decisions = self
            .client
            .get_documents(
                vec![],
                &self.spec,
                terminusdb_client::GetOpts {
                    unfold: true,
                    type_filter: Some("DecisionDoc".to_string()),
                    ..Default::default()
                },
            )
            .await?;
        let mut parsed: Vec<DecisionDoc> = Vec::new();
        for d in decisions {
            parsed.push(serde_json::from_value(d)?);
        }
        let decisions = parsed;

        let mut out = Vec::new();
        for req in requests {
            let req_decisions = decisions
                .iter()
                .filter(|d| d.request_id == req.request_id)
                .cloned()
                .collect();
            out.push((req, req_decisions));
        }
        Ok(out)
    }

    // ------------------------------------------------------------------
    // Correlation + revert
    // ------------------------------------------------------------------

    /// Save a property set with an optional correlation id (`|corr:{id}`).
    pub async fn save_property_set_correlated(
        &self,
        entity_id: Uuid,
        instance_id: Uuid,
        set: &PropertySet,
        author: &str,
        message: &str,
        correlation_id: Option<&str>,
    ) -> anyhow::Result<()> {
        let msg = match correlation_id {
            Some(id) => format!("{message}|corr:{id}"),
            None => message.to_string(),
        };
        self.save_property_set(entity_id, instance_id, set, author, &msg)
            .await
    }

    /// Restore the property set as of `target_commit` with a `|rev:` marker.
    pub async fn revert_property_set(
        &self,
        entity_id: Uuid,
        instance_id: Uuid,
        chain: &[Uuid],
        target_commit: &str,
        author: &str,
        reason: &str,
    ) -> anyhow::Result<()> {
        let as_of = resolve_at(self, entity_id, chain, target_commit).await?;
        let current_version = self
            .resolve_at_current_version(entity_id, instance_id, chain)
            .await;
        let restored = PropertySet {
            scope: as_of.scope,
            version: current_version + 1,
            status: knowledge_domain::Status::Current,
            properties: as_of.properties,
        };
        let msg = format!("{reason}|rev:{target_commit}");
        self.save_property_set(entity_id, instance_id, &restored, author, &msg)
            .await
    }

    async fn resolve_at_current_version(
        &self,
        entity_id: Uuid,
        instance_id: Uuid,
        _chain: &[Uuid],
    ) -> u64 {
        // highest version saved at this instance (from the log tokens)
        let log = match self.log().await {
            Ok(l) => l,
            Err(_) => return 0,
        };
        let needle = format!("{entity_id}:{instance_id}:");
        for entry in &log {
            if let Some((_, ps)) = entry.message.rsplit_once("|ps:")
                && ps.starts_with(&needle)
            {
                let parts: Vec<&str> = ps.split(':').collect();
                if let Some(v) = parts
                    .last()
                    .and_then(|v| v.trim_start_matches('v').parse().ok())
                {
                    return v;
                }
            }
        }
        0
    }

    // ------------------------------------------------------------------
    // Audit projections
    // ------------------------------------------------------------------

    pub async fn audit_for_entity(&self, entity_id: Uuid) -> anyhow::Result<Vec<AuditEntry>> {
        let mut entries = self.audit_from_log(entity_id).await?;
        entries.extend(self.audit_from_ledger(entity_id).await?);
        entries.sort_by(|a, b| a.commit.cmp(&b.commit));
        Ok(entries)
    }

    pub async fn audit_for_actor(&self, actor: &str) -> anyhow::Result<Vec<AuditEntry>> {
        let all = self.audit_all_from_log().await?;
        Ok(all.into_iter().filter(|e| e.actor == actor).collect())
    }

    async fn audit_from_log(&self, entity_id: Uuid) -> anyhow::Result<Vec<AuditEntry>> {
        let (events, _) = stream::poll_events(self, &CommitCursor::default()).await?;
        let entity = entity_id.to_string();
        let mut out = Vec::new();
        for ev in events {
            let (id, action, _corr) = match ev {
                stream::DomainEvent::EntityCreated {
                    entity_id, commit, ..
                } if entity_id.to_string() == entity => {
                    (commit, AuditAction::EntityCreated, None::<String>)
                }
                stream::DomainEvent::PropertySetSaved {
                    entity_id, commit, ..
                } if entity_id.to_string() == entity => {
                    (commit, AuditAction::PropertySetSaved, None::<String>)
                }
                _ => continue,
            };
            // correlation + revert tokens come from the raw log (events don't carry them)
            let (corr, rev) = self.tokens_for_commit(&id).await;
            let action = if rev.is_some() {
                AuditAction::Reverted
            } else {
                action
            };
            out.push(AuditEntry {
                commit: id.clone(),
                actor: self.actor_for_commit(&id).await.unwrap_or_default(),
                action,
                entity_id,
                correlation_id: corr,
                details: rev.unwrap_or_default(),
            });
        }
        Ok(out)
    }

    async fn audit_from_ledger(&self, entity_id: Uuid) -> anyhow::Result<Vec<AuditEntry>> {
        let mut out = Vec::new();
        for (req, decisions) in self.requests_for_entity(entity_id).await? {
            let req_commit = format!("ledger:{}", req.request_id);
            out.push(AuditEntry {
                commit: req_commit.clone(),
                actor: req.created_by.clone(),
                action: AuditAction::RequestSubmitted,
                entity_id,
                correlation_id: req.correlation_id.clone(),
                details: format!(
                    "request {} proposed v{}",
                    req.request_id, req.proposed_version
                ),
            });
            for d in decisions {
                out.push(AuditEntry {
                    commit: format!("ledger:{}:{}", req.request_id, d.decided_at),
                    actor: d.decided_by.clone(),
                    action: if d.approve {
                        AuditAction::Approved
                    } else {
                        AuditAction::Rejected
                    },
                    entity_id,
                    correlation_id: req.correlation_id.clone(),
                    details: req.request_id.clone(),
                });
            }
        }
        Ok(out)
    }

    async fn audit_all_from_log(&self) -> anyhow::Result<Vec<AuditEntry>> {
        let (events, _) = stream::poll_events(self, &CommitCursor::default()).await?;
        let mut out = Vec::new();
        for ev in events {
            let (id, action) = match ev {
                stream::DomainEvent::EntityCreated { commit, .. } => {
                    (commit, AuditAction::EntityCreated)
                }
                stream::DomainEvent::PropertySetSaved { commit, .. } => {
                    (commit, AuditAction::PropertySetSaved)
                }
            };
            let (corr, rev) = self.tokens_for_commit(&id).await;
            let action = if rev.is_some() {
                AuditAction::Reverted
            } else {
                action
            };
            let (entity_id, actor) = self.entity_and_actor_for_commit(&id).await;
            out.push(AuditEntry {
                commit: id,
                actor,
                action,
                entity_id,
                correlation_id: corr,
                details: rev.unwrap_or_default(),
            });
        }
        Ok(out)
    }

    async fn tokens_for_commit(&self, commit: &str) -> (Option<String>, Option<String>) {
        let log = match self.log().await {
            Ok(l) => l,
            Err(_) => return (None, None),
        };
        for e in &log {
            if e.identifier == *commit {
                let corr = e
                    .message
                    .rsplit_once("|corr:")
                    .map(|(_, id)| id.split('|').next().unwrap_or("").to_string())
                    .filter(|s| !s.is_empty());
                let rev = e
                    .message
                    .rsplit_once("|rev:")
                    .map(|(_, t)| t.split('|').next().unwrap_or("").to_string())
                    .filter(|s| !s.is_empty());
                return (corr, rev);
            }
        }
        (None, None)
    }

    async fn actor_for_commit(&self, commit: &str) -> Option<String> {
        let log = self.log().await.ok()?;
        log.iter()
            .find(|e| e.identifier == *commit)
            .map(|e| e.author.clone())
    }

    async fn entity_and_actor_for_commit(&self, commit: &str) -> (Uuid, String) {
        let log = match self.log().await {
            Ok(l) => l,
            Err(_) => return (Uuid::nil(), String::new()),
        };
        for e in &log {
            if e.identifier == *commit {
                if let Some((_, ps)) = e.message.rsplit_once("|ps:") {
                    let parts: Vec<&str> = ps.split(':').collect();
                    if let Ok(u) = parts[0].parse() {
                        return (u, e.author.clone());
                    }
                }
                if let Some((_, ent)) = e.message.rsplit_once("|ent:") {
                    let parts: Vec<&str> = ent.split(':').collect();
                    if let Ok(u) = parts[0].parse() {
                        return (u, e.author.clone());
                    }
                }
            }
        }
        (Uuid::nil(), String::new())
    }
}
