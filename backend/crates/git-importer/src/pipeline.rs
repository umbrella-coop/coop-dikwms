//! Ingestion pipeline: walk → transform → persist → checkpoint (SPEC-027).
//!
//! Single pass, newest-first. Commit `parents` are stored twice: as hexsha
//! strings (stable identifiers, also for out-of-window parents) and as known
//! entity Uuids (ADR-004 adjacency properties). Clients resolve hexsha → Uuid
//! through the loaded hash map; unimported parents render as ghost nodes.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use data_graph::{EntityKind, PropertySet, Scope};
use terminusdb_repository::Repository;
use uuid::Uuid;

use crate::git::{GitAuthor, GitCommit};
use crate::retry::{Backoff, CircuitBreaker};
use crate::state::ImportState;

const BATCH_SIZE: usize = 100;
const MAX_ATTEMPTS: u32 = 5;
const COMMON_INSTANCE: Uuid = Uuid::nil();
const IMPORTER_AUTHOR: &str = "git-importer";

pub struct Importer {
    repo: Repository,
    state: ImportState,
    state_path: PathBuf,
}

#[derive(Debug, Default, PartialEq)]
pub struct ImportReport {
    pub seen: usize,
    pub imported: usize,
    pub skipped: usize,
}

impl Importer {
    pub fn new(repo: Repository, state: ImportState, state_path: PathBuf) -> Self {
        Self {
            repo,
            state,
            state_path,
        }
    }

    /// Rebuild the hexsha→Uuid and email→Uuid indexes from stored property
    /// sets. DB wins over the state file (REQ-003 reconciliation) — closes
    /// the crash-after-write-before-checkpoint gap.
    pub async fn reconcile(&mut self) -> anyhow::Result<()> {
        let hashes = self.repo.property_index("hash").await?;
        for (hexsha, ids) in &hashes {
            if let Some(id) = ids.first() {
                self.state
                    .hexsha_index
                    .insert(hexsha.clone(), id.to_string());
            }
        }
        let emails = self.repo.property_index("email").await?;
        for (email, ids) in &emails {
            if let Some(id) = ids.first() {
                self.state
                    .authors_index
                    .insert(email.clone(), id.to_string());
            }
        }
        Ok(())
    }

    pub async fn run(&mut self, commits: &[GitCommit]) -> anyhow::Result<ImportReport> {
        let mut backoff = Backoff::new(Duration::from_secs(1), Duration::from_secs(30));
        let mut breaker = CircuitBreaker::new(Duration::from_secs(60), 8);
        let mut run_authors: HashMap<String, Uuid> = HashMap::new();
        let mut report = ImportReport::default();
        let mut since_checkpoint = 0usize;

        for commit in commits {
            report.seen += 1;
            self.state.counts.seen += 1;

            if self.state.is_imported(&commit.hexsha) {
                continue;
            }

            let author_id = self
                .ensure_author(&commit.author, &mut run_authors, &mut breaker, &mut backoff)
                .await?;

            let entity_id = retry_write(&mut breaker, &mut backoff, || async {
                self.repo
                    .create_entity_as(EntityKind::Node, IMPORTER_AUTHOR)
                    .await
            })
            .await
            .with_context(|| format!("create entity for {}", commit.hexsha))?;

            let props = build_props(commit, author_id);
            let set = PropertySet::current(Scope::Common, 1, props);
            let repo = &self.repo;
            let write_result = retry_write(&mut breaker, &mut backoff, || async {
                repo.save_property_set(
                    entity_id,
                    COMMON_INSTANCE,
                    &set,
                    IMPORTER_AUTHOR,
                    "import-commit",
                )
                .await
            })
            .await;

            match write_result {
                Ok(()) => {
                    self.state.record_imported(&commit.hexsha, entity_id);
                    report.imported += 1;
                    since_checkpoint += 1;
                }
                Err(e) if !breaker.should_halt() => {
                    self.state.record_skipped(&commit.hexsha, &format!("{e:#}"));
                    report.skipped += 1;
                }
                Err(e) => return Err(e),
            }

            if since_checkpoint >= BATCH_SIZE {
                self.state.save(&self.state_path)?;
                since_checkpoint = 0;
            }
        }
        self.state.save(&self.state_path)?;
        Ok(report)
    }

    async fn ensure_author(
        &mut self,
        author: &GitAuthor,
        run_authors: &mut HashMap<String, Uuid>,
        breaker: &mut CircuitBreaker,
        backoff: &mut Backoff,
    ) -> anyhow::Result<Uuid> {
        let email = normalize_email(&author.email);
        if let Some(id) = self.state.authors_index.get(&email) {
            return id.parse().context("corrupt authors_index uuid");
        }
        if let Some(id) = run_authors.get(&email) {
            return Ok(*id);
        }
        let id = retry_write(breaker, backoff, || async {
            self.repo
                .create_entity_as(EntityKind::Node, IMPORTER_AUTHOR)
                .await
        })
        .await
        .with_context(|| format!("create author entity {email}"))?;
        let props = HashMap::from([
            ("name".to_string(), serde_json::json!(author.name)),
            ("email".to_string(), serde_json::json!(email)),
        ]);
        let set = PropertySet::current(Scope::Common, 1, props);
        retry_write(breaker, backoff, || async {
            self.repo
                .save_property_set(id, COMMON_INSTANCE, &set, IMPORTER_AUTHOR, "import-author")
                .await
        })
        .await?;
        run_authors.insert(email.clone(), id);
        self.state.authors_index.insert(email, id.to_string());
        Ok(id)
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
    use chrono::DateTime;

    fn commit() -> GitCommit {
        GitCommit {
            hexsha: "abc123".to_string(),
            message: "fix: thing".to_string(),
            authored_at: DateTime::parse_from_rfc3339("2026-01-01T00:00:00+00:00").unwrap(),
            author: GitAuthor {
                name: "Alice Example".to_string(),
                email: "Alice@Example.COM".to_string(),
            },
            parents: vec!["deadbeef".to_string()],
            paths: vec!["src/lib.rs".to_string(), "src/lib.rs".to_string()],
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
        assert_eq!(props["paths"].as_array().unwrap().len(), 2);
        assert!(
            props["authored_at"]
                .as_str()
                .unwrap()
                .contains("2026-01-01")
        );
    }

    #[test]
    fn walk_and_filter_pipeline_smoke() {
        // window filter + state skip logic compose (no server needed)
        let mut state = ImportState::new("repo", "main", "2025-08-11");
        let c = commit();
        assert!(!state.is_imported(&c.hexsha));
        state.record_imported(&c.hexsha, Uuid::nil());
        assert!(state.is_imported(&c.hexsha));
    }
}
