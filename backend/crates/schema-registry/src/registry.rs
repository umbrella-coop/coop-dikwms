//! TerminusDB-backed schema registry store (SPEC-009).

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use prost::Message;
use prost_types::FileDescriptorSet;
use terminusdb_client::{BranchSpec, DocumentInsertArgs, TerminusDBHttpClient};
use terminusdb_schema::{EntityIDFor, ToTDBInstance};
use terminusdb_schema_derive::{FromTDBInstance, TerminusDBModel};

use crate::core;
use crate::descriptor;
use crate::lint;

#[derive(Clone, Debug, PartialEq, TerminusDBModel, FromTDBInstance)]
#[tdb(id_field = "id", key = "random")]
pub struct NamespaceDoc {
    id: EntityIDFor<Self>,
    pub package: String,
    pub version: String,
    /// base64-encoded FileDescriptorSet bytes
    pub descriptor_b64: String,
    /// type names defined by the namespace
    pub types: Vec<String>,
}

impl NamespaceDoc {
    pub fn descriptor(&self) -> anyhow::Result<FileDescriptorSet> {
        let bytes = B64.decode(&self.descriptor_b64)?;
        Ok(FileDescriptorSet::decode(bytes.as_slice())?)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    TagViolation(String),
    TypeNotFound(String),
    AlreadyRegistered { package: String, version: String },
    Parse(String),
    NotFound(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::TagViolation(msg) => write!(f, "tag violation: {msg}"),
            RegistryError::TypeNotFound(t) => write!(f, "type not found: {t}"),
            RegistryError::AlreadyRegistered { package, version } => {
                write!(f, "namespace already registered: {package}@{version}")
            }
            RegistryError::Parse(msg) => write!(f, "parse error: {msg}"),
            RegistryError::NotFound(msg) => write!(f, "not found: {msg}"),
        }
    }
}

impl std::error::Error for RegistryError {}

/// `additionalType` IRI format: `terminusdb://schema/{package}/{Type}`
pub const ADDITIONAL_TYPE_SCHEME: &str = "terminusdb://schema/";

pub struct SchemaRegistry {
    client: TerminusDBHttpClient,
    db: String,
    spec: BranchSpec,
}

impl SchemaRegistry {
    /// Initialize the registry: ensure the store and seed `core.v1` +
    /// `org.schema.v1`.
    pub async fn init(client: TerminusDBHttpClient, db: String) -> anyhow::Result<Self> {
        client.ensure_database(&db).await?;
        client
            .schema::<NamespaceDoc>(DocumentInsertArgs::from(BranchSpec::from(db.as_str())))
            .await?;
        let reg = Self {
            client,
            spec: BranchSpec::from(db.as_str()),
            db,
        };
        reg.register(&core::core_v1(), "core.v1", "1").await?;
        reg.register(&core::org_schema_v1(), "org.schema.v1", "1")
            .await?;
        Ok(reg)
    }

    pub fn client(&self) -> &TerminusDBHttpClient {
        &self.client
    }

    pub fn db(&self) -> &str {
        &self.db
    }

    /// Register a namespace version. Fails on tag violations vs the previous
    /// version and on duplicate (package, version).
    pub async fn register(
        &self,
        fds: &FileDescriptorSet,
        package: &str,
        version: &str,
    ) -> Result<(), RegistryError> {
        let pkg = descriptor::package(fds).unwrap_or_else(|| package.to_string());
        if let Some(prev) = self
            .latest_for_package(&pkg)
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))?
        {
            if prev.version == version {
                return Err(RegistryError::AlreadyRegistered {
                    package: pkg.clone(),
                    version: version.to_string(),
                });
            }
            let prev_fds = prev
                .descriptor()
                .map_err(|e| RegistryError::Parse(e.to_string()))?;
            lint::lint_tags(&prev_fds, fds)
                .map_err(|v| RegistryError::TagViolation(format!("{v:?}")))?;
        }
        let types = descriptor::type_names(fds);
        let bare_id = format!("NS:{pkg}:{version}");
        let doc = NamespaceDoc {
            id: EntityIDFor::new(bare_id.as_str())
                .map_err(|e| RegistryError::Parse(e.to_string()))?,
            package: pkg.clone(),
            version: version.to_string(),
            descriptor_b64: B64.encode(fds.encode_to_vec()),
            types,
        };
        let mut args = DocumentInsertArgs::from(self.spec.clone());
        args.author = "system".to_string();
        args.message = format!("register {pkg}@{version}");
        self.client
            .insert(&doc, args)
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))?;
        Ok(())
    }

    pub async fn get(&self, package: &str, version: &str) -> anyhow::Result<Option<NamespaceDoc>> {
        let docs = self.all().await?;
        Ok(docs
            .into_iter()
            .find(|d| d.package == package && d.version == version))
    }

    /// List all registered namespace docs (for the API surface).
    pub async fn list_namespaces(&self) -> anyhow::Result<Vec<NamespaceDoc>> {
        self.all().await
    }

    pub async fn latest_for_package(&self, package: &str) -> anyhow::Result<Option<NamespaceDoc>> {
        let docs = self.all().await?;
        Ok(docs
            .into_iter()
            .filter(|d| d.package == package)
            .max_by_key(|d| d.version.parse::<u64>().unwrap_or(0)))
    }

    async fn all(&self) -> anyhow::Result<Vec<NamespaceDoc>> {
        let docs = self
            .client
            .get_documents(
                vec![],
                &self.spec,
                terminusdb_client::GetOpts {
                    unfold: true,
                    type_filter: Some("NamespaceDoc".to_string()),
                    ..Default::default()
                },
            )
            .await?;
        docs.into_iter()
            .map(|d| serde_json::from_value(d).map_err(anyhow::Error::from))
            .collect()
    }

    /// Validate an `additionalType` IRI against registered types.
    pub async fn validate_additional_type(&self, iri: &str) -> Result<(), RegistryError> {
        let rest = iri
            .strip_prefix(ADDITIONAL_TYPE_SCHEME)
            .ok_or_else(|| RegistryError::Parse(format!("invalid additionalType IRI: {iri}")))?;
        let (package, type_name) = rest
            .rsplit_once('/')
            .ok_or_else(|| RegistryError::Parse(format!("invalid additionalType IRI: {iri}")))?;
        let doc = self
            .latest_for_package(package)
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))?
            .ok_or_else(|| RegistryError::NotFound(format!("package {package}")))?;
        if doc.types.iter().any(|t| t == type_name) {
            Ok(())
        } else {
            Err(RegistryError::TypeNotFound(type_name.to_string()))
        }
    }
}
