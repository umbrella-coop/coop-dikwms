//! Ingestion pipeline: walk → transform → bulk-import via the platform API
//! (SPEC-027 REQ-001/006). The importer is an HTTP client of the dikwms API
//! (`POST /bulk-entities`) — docker runs the services that expose the APIs;
//! this binary consumes them.
//!
//! Server-side idempotency: every entity carries a `dedupe_key` value (commit
//! hexsha / normalized email); the API skips existing keys, so re-runs and
//! crash-resumes can never duplicate entities (no client-side reconcile
//! needed).

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use data_graph::{EntityKind, PropertySet, Scope};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::git::GitCommit;
use crate::insights;
use crate::retry::{Backoff, CircuitBreaker};
use crate::state::ImportState;

const BATCH_SIZE: usize = 100;
const MAX_ATTEMPTS: u32 = 5;

#[derive(Clone)]
pub struct PlatformClient {
    base: String,
    http: reqwest::Client,
}

impl PlatformClient {
    pub fn new(base: &str) -> anyhow::Result<Self> {
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        })
    }

    pub async fn bulk_entities(
        &self,
        dedupe_key: &str,
        entities: &[BulkEntity],
    ) -> anyhow::Result<BulkResponse> {
        let resp = self
            .http
            .post(format!("{}/bulk-entities", self.base))
            .json(&serde_json::json!({ "dedupe_key": dedupe_key, "entities": entities }))
            .send()
            .await?
            .error_for_status()?
            .json::<BulkResponse>()
            .await?;
        Ok(resp)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkEntity {
    pub kind: EntityKind,
    pub idempotency_key: String,
    /// data-graph domain model as the wire contract (SPEC-027 REQ-006):
    /// scope/version/status/properties serialize 1:1 with the API.
    pub set: PropertySet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkResultEntry {
    pub idempotency_key: String,
    pub entity_id: Option<String>,
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkResponse {
    pub results: Vec<BulkResultEntry>,
    #[allow(dead_code)]
    pub warnings: Vec<serde_json::Value>,
}

pub struct Importer {
    client: PlatformClient,
    state: ImportState,
    state_path: PathBuf,
}

#[derive(Debug, Default, PartialEq)]
pub struct ImportReport {
    pub seen: usize,
    pub imported: usize,
    pub skipped: usize,
    pub warnings: usize,
}

impl Importer {
    pub fn new(client: PlatformClient, state: ImportState, state_path: PathBuf) -> Self {
        Self {
            client,
            state,
            state_path,
        }
    }

    pub async fn run(&mut self, commits: &[GitCommit]) -> anyhow::Result<ImportReport> {
        let mut backoff = Backoff::new(Duration::from_secs(1), Duration::from_secs(30));
        let mut breaker = CircuitBreaker::new(Duration::from_secs(60), 8);
        let mut report = ImportReport::default();
        let mut author_ids: HashMap<String, Uuid> = HashMap::new();

        // Phase A — author entities (dedupe by normalized email).
        let mut author_emails: Vec<String> = Vec::new();
        for commit in commits {
            let email = normalize_email(&commit.author.email);
            if !author_emails.contains(&email) {
                author_emails.push(email);
            }
        }
        for email in &author_emails {
            if let Some(id) = self
                .state
                .authors_index
                .get(email)
                .and_then(|s| s.parse().ok())
            {
                author_ids.insert(email.clone(), id);
                continue;
            }
            let author = &commits
                .iter()
                .find(|c| normalize_email(&c.author.email) == *email)
                .expect("author email derived from commits")
                .author;
            let entity = BulkEntity {
                kind: EntityKind::Node,
                idempotency_key: email.clone(),
                set: PropertySet::current(
                    Scope::Common,
                    1,
                    HashMap::from([
                        ("name".to_string(), serde_json::json!(author.name)),
                        ("email".to_string(), serde_json::json!(email)),
                    ]),
                ),
            };
            let batch = vec![entity.clone()];
            let resp = retry_write(&mut breaker, &mut backoff, || {
                self.client.bulk_entities("email", &batch)
            })
            .await
            .with_context(|| format!("bulk author {email}"))?;
            report.warnings += resp.warnings.len();
            for entry in resp.results {
                let id = entry
                    .entity_id
                    .as_deref()
                    .map(|s| s.parse())
                    .transpose()
                    .context("api returned no entity_id for author")?
                    .ok_or_else(|| anyhow::anyhow!("author {email} skipped without entity_id"))?;
                author_ids.insert(email.clone(), id);
                self.state
                    .authors_index
                    .insert(email.clone(), id.to_string());
            }
        }

        // Phase B — commit entities (dedupe by hexsha), batched.
        let mut pending: Vec<BulkEntity> = Vec::new();
        let mut pending_hexshas: Vec<String> = Vec::new();
        for commit in commits {
            report.seen += 1;
            self.state.counts.seen += 1;

            if self.state.is_imported(&commit.hexsha) {
                continue;
            }
            let author_id = author_ids
                .get(&normalize_email(&commit.author.email))
                .copied()
                .ok_or_else(|| anyhow::anyhow!("no author entity for {}", commit.hexsha))?;
            pending.push(BulkEntity {
                kind: EntityKind::Node,
                idempotency_key: commit.hexsha.clone(),
                set: PropertySet::current(Scope::Common, 1, build_props(commit, author_id)),
            });
            pending_hexshas.push(commit.hexsha.clone());

            if pending.len() >= BATCH_SIZE {
                self.apply_batch(
                    &pending,
                    &pending_hexshas,
                    &mut report,
                    &mut breaker,
                    &mut backoff,
                )
                .await?;
                self.state.save(&self.state_path)?;
                pending.clear();
                pending_hexshas.clear();
            }
        }
        if !pending.is_empty() {
            self.apply_batch(
                &pending,
                &pending_hexshas,
                &mut report,
                &mut breaker,
                &mut backoff,
            )
            .await?;
        }
        self.state.save(&self.state_path)?;
        Ok(report)
    }

    /// Wisdom post-pass (REQ-010): compute dir-level insights and persist them
    /// as a property set on a per-repo insight entity (idempotency key
    /// `git-insight-<repo>-<since>`). `window_end` is the external staleness
    /// anchor (import time). Returns the number of dirs analyzed.
    pub async fn run_insights(
        &self,
        commits: &[GitCommit],
        window_end: chrono::DateTime<chrono::FixedOffset>,
    ) -> anyhow::Result<usize> {
        let dir_insights = insights::compute_dir_insights(commits, window_end);
        if dir_insights.is_empty() {
            return Ok(0);
        }
        let key = format!("git-insight-{}-{}", self.state.repo, self.state.since);
        let entity = BulkEntity {
            kind: EntityKind::Node,
            idempotency_key: key.clone(),
            set: PropertySet::current(
                Scope::Common,
                1,
                HashMap::from([
                    ("hash".to_string(), serde_json::json!(key)),
                    (
                        "insights".to_string(),
                        insights::insights_properties(&dir_insights),
                    ),
                ]),
            ),
        };
        let batch = vec![entity];
        self.client.bulk_entities("hash", &batch).await?;
        Ok(dir_insights.len())
    }

    async fn apply_batch(
        &mut self,
        entities: &[BulkEntity],
        hexshas: &[String],
        report: &mut ImportReport,
        breaker: &mut CircuitBreaker,
        backoff: &mut Backoff,
    ) -> anyhow::Result<()> {
        let resp = retry_write(breaker, backoff, || {
            self.client.bulk_entities("hash", entities)
        })
        .await?;
        report.warnings += resp.warnings.len();
        for (hexsha, entry) in hexshas.iter().zip(resp.results) {
            match (entry.status.as_str(), entry.entity_id) {
                ("imported", Some(id)) => {
                    let uuid = id.parse().context("api returned invalid entity_id")?;
                    self.state.record_imported(hexsha, uuid);
                    report.imported += 1;
                }
                ("skipped", _) => {
                    let reason = entry.reason.unwrap_or_default();
                    if !reason.is_empty() {
                        self.state.record_skipped(hexsha, &reason);
                        report.skipped += 1;
                    }
                    // skipped-already-imported: idempotent replay, not an error.
                }
                (other, _) => {
                    self.state
                        .record_skipped(hexsha, &format!("unexpected status {other}"));
                    report.skipped += 1;
                }
            }
        }
        Ok(())
    }
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub fn build_props(commit: &GitCommit, author_id: Uuid) -> HashMap<String, serde_json::Value> {
    HashMap::from([
        ("hash".to_string(), serde_json::json!(commit.hexsha)),
        ("message".to_string(), serde_json::json!(commit.message)),
        (
            "authored_at".to_string(),
            serde_json::json!(commit.authored_at.to_rfc3339()),
        ),
        (
            "author_name".to_string(),
            serde_json::json!(commit.author.name),
        ),
        (
            "author_email".to_string(),
            serde_json::json!(commit.author.email),
        ),
        (
            "author".to_string(),
            serde_json::json!(author_id.to_string()),
        ),
        (
            "parent_hexshas".to_string(),
            serde_json::json!(&commit.parents),
        ),
        (
            "parent_uuids".to_string(),
            serde_json::json!(Vec::<String>::new()),
        ),
        ("paths".to_string(), serde_json::json!(&commit.paths)),
    ])
}

async fn retry_write<T, F, Fut>(
    breaker: &mut CircuitBreaker,
    backoff: &mut Backoff,
    mut f: F,
) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(value) => {
                breaker.record_success();
                backoff.reset();
                return Ok(value);
            }
            Err(e) => {
                breaker.record_failure();
                if breaker.should_halt() {
                    return Err(e).context("circuit breaker halted");
                }
                attempt += 1;
                if attempt >= MAX_ATTEMPTS {
                    return Err(e).context("exhausted retries");
                }
                tokio::time::sleep(backoff.next_delay()).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime as ChronoDateTime;

    fn commit() -> GitCommit {
        GitCommit {
            hexsha: "abc123".to_string(),
            message: "fix: thing".to_string(),
            authored_at: ChronoDateTime::parse_from_rfc3339("2026-01-01T00:00:00+00:00").unwrap(),
            author: crate::git::GitAuthor {
                name: "Alice Example".to_string(),
                email: "Alice@Example.COM".to_string(),
            },
            parents: vec!["deadbeef".to_string()],
            paths: vec!["src/lib.rs".to_string()],
        }
    }

    #[test]
    fn normalize_email_lowercases_and_trims() {
        assert_eq!(normalize_email("  Alice@Example.COM "), "alice@example.com");
    }

    #[test]
    fn build_props_covers_mapping() {
        let author_id = Uuid::now_v7();
        let props = build_props(&commit(), author_id);
        assert_eq!(props["hash"], "abc123");
        assert_eq!(props["message"], "fix: thing");
        assert_eq!(props["author"], author_id.to_string());
        assert_eq!(props["parent_hexshas"][0], "deadbeef");
        assert!(props["parent_uuids"].as_array().unwrap().is_empty());
        assert!(
            props["authored_at"]
                .as_str()
                .unwrap()
                .contains("2026-01-01")
        );
    }
}
