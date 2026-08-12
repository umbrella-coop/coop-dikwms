//! SPEC-027 integration tests (AC-1, AC-2, AC-3, author dedupe).
//! Full path, no mocks: importer (HTTP) → in-process axum API → repository →
//! compose-hosted official TerminusDB v12.0.7.
//!   docker compose up -d terminusdb
//! Env: TERMINUSDB_URL (default http://localhost:6363), TERMINUSDB_ADMIN_PASS (default root).

#![recursion_limit = "512"]

use std::env;
use std::path::Path;

use api::router;
use chrono::{TimeZone, Utc};
use git_importer::git::walk;
use git_importer::pipeline::{Importer, PlatformClient, normalize_email};
use git_importer::state::ImportState;
use schema_registry::SchemaRegistry;
use terminusdb_repository::Repository;
use url::Url;
use uuid::Uuid;

/// Shared compose server — serialize the async tests (AGENTS.md transaction
/// contention pattern, mirrors api/tests/spec_013_api.rs).
static SERVER_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    SERVER_LOCK.lock().await
}

/// Connect to the docker TerminusDB and boot the real API in-process on an
/// ephemeral port. Returns (api base url, repository handle for assertions).
async fn start_platform() -> anyhow::Result<(String, Repository)> {
    let endpoint = env::var("TERMINUSDB_URL").unwrap_or_else(|_| "http://localhost:6363".into());
    let pass = env::var("TERMINUSDB_ADMIN_PASS").unwrap_or_else(|_| "root".into());
    let client = terminusdb_client::TerminusDBHttpClient::new(
        Url::parse(&endpoint)?,
        "admin",
        &pass,
        "admin",
    )
    .await?;
    let db = format!("git_importer_it_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client.clone(), db.clone()).await?;
    let registry = SchemaRegistry::init(client, format!("{db}_registry")).await?;
    let app = router(repo.clone(), registry);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("api server");
    });
    Ok((format!("http://{addr}"), repo))
}

async fn drop_db(repo: &Repository) {
    let _ = repo.client().delete_database(repo.db()).await;
}

fn sig(name: &str, email: &str, days: i64) -> git2::Signature<'static> {
    git2::Signature::new(name, email, &git2::Time::new(days_ago(days), 0)).unwrap()
}

fn days_ago(days: i64) -> i64 {
    Utc::now().timestamp() - days * 86_400
}

fn cutoff_days_ago(days: i64) -> chrono::DateTime<chrono::FixedOffset> {
    Utc.timestamp_opt(days_ago(days), 0).unwrap().fixed_offset()
}

/// 6-commit fixture with a merge (2 parents):
///   c0 (Alice, 400d) ← c1 (Alice, 300d) ← c2 (Alice, 200d) ← c3 (Bob, 150d) ← c4 (Bob, 100d)
///                                        └─ c2b (Bob, 120d) ←┐
///                                                          merge c5 (Alice, 90d)
/// Authors use email-case variants to exercise dedupe.
fn fixture(dir: &Path) -> anyhow::Result<git2::Repository> {
    let repo = git2::Repository::init(dir)?;
    let commit = |repo: &git2::Repository,
                  index: &mut git2::Index,
                  file: &str,
                  days: i64,
                  parents: &[String]|
     -> anyhow::Result<String> {
        std::fs::write(dir.join(file), format!("content {file}\n"))?;
        index.add_path(Path::new(file))?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let author = match file.contains("bob") {
            true => ("Bob", "BOB@example.com"),
            false => ("Alice", "Alice@Example.COM"),
        };
        let parents: Vec<git2::Commit> = parents
            .iter()
            .map(|hex| repo.find_commit(git2::Oid::from_str(hex).unwrap()))
            .collect::<Result<_, _>>()?;
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(
            Some("HEAD"),
            &sig(author.0, author.1, days),
            &sig(author.0, author.1, days),
            file,
            &tree,
            &parent_refs,
        )
        .map(|oid| oid.to_string())
        .map_err(Into::into)
    };

    let mut index = repo.index()?;
    let c0 = commit(&repo, &mut index, "c0.txt", 400, &[])?;
    let c1 = commit(&repo, &mut index, "c1.txt", 300, &[c0])?;
    let c2 = commit(&repo, &mut index, "c2.txt", 200, &[c1])?;
    {
        let c2_commit = repo.find_commit(git2::Oid::from_str(&c2)?)?;
        let _side = repo.branch("side", &c2_commit, false)?;
    }
    repo.set_head("refs/heads/side")?;
    let c2b = commit(
        &repo,
        &mut index,
        "c2b-bob.txt",
        120,
        std::slice::from_ref(&c2),
    )?;
    repo.set_head("refs/heads/main")?;
    let c3 = commit(&repo, &mut index, "c3.txt", 150, &[c2])?;
    let c4 = commit(&repo, &mut index, "c4.txt", 100, &[c3])?;
    let _c5 = commit(&repo, &mut index, "c5-merge.txt", 90, &[c4, c2b])?;
    Ok(repo)
}

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("git-importer-it-{name}"))
}

// ------------------------------------------------------------------
// AC-1: imported count == window walk count; property sets populated.
// ------------------------------------------------------------------
#[tokio::test]
async fn import_matches_window_and_populates_property_sets() -> anyhow::Result<()> {
    let _g = guard().await;
    let dir = tmp("ac1");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    fixture(&dir)?;

    let (api_base, repo) = start_platform().await?;

    let since = cutoff_days_ago(250);
    let commits = walk(&dir, since)?;
    assert_eq!(commits.len(), 5, "window = c2,c2b,c3,c4,c5");

    let state_path = tmp("ac1-state.json");
    let _ = std::fs::remove_file(&state_path);
    let state = ImportState::new("fixture", "main", "2025-08-11");
    let mut importer = Importer::new(PlatformClient::new(&api_base)?, state, state_path.clone());
    let report = importer.run(&commits).await?;
    assert_eq!(report.imported, 5);
    assert_eq!(report.skipped, 0);

    let hashes = repo.property_index("hash").await?;
    assert_eq!(hashes.len(), 5, "every window commit persisted");

    let newest = &commits[0]; // c5 merge — 2 parents
    let ids = hashes.get(&newest.hexsha).expect("c5 indexed");
    let ps = repo.load_property_sets(ids[0]).await?;
    assert_eq!(ps.len(), 1);
    assert_eq!(ps[0].properties["hash"].as_str().unwrap(), newest.hexsha);
    assert_eq!(
        ps[0].properties["message"].as_str().unwrap(),
        "c5-merge.txt"
    );
    let parent_hexshas = ps[0].properties["parent_hexshas"].as_array().unwrap();
    assert_eq!(parent_hexshas.len(), 2, "merge parents stored");

    let emails = repo.property_index("email").await?;
    assert_eq!(emails.len(), 2, "authors deduped by normalized email");
    assert!(emails.contains_key("alice@example.com"));
    assert!(emails.contains_key("bob@example.com"));

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&state_path);
    drop_db(&repo).await;
    Ok(())
}

// ------------------------------------------------------------------
// AC-2: re-run creates zero new entities (server-side idempotency).
// ------------------------------------------------------------------
#[tokio::test]
async fn rerun_is_idempotent() -> anyhow::Result<()> {
    let _g = guard().await;
    let dir = tmp("ac2");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    fixture(&dir)?;

    let (api_base, repo) = start_platform().await?;

    let since = cutoff_days_ago(250);
    let commits = walk(&dir, since)?;
    let state_path = tmp("ac2-state.json");
    let _ = std::fs::remove_file(&state_path);

    let state = ImportState::new("fixture", "main", "2025-08-11");
    let mut importer = Importer::new(PlatformClient::new(&api_base)?, state, state_path.clone());
    let first = importer.run(&commits).await?;
    assert_eq!(first.imported, 5);

    let before = repo.property_index("hash").await?;
    assert_eq!(before.len(), 5);

    let loaded = ImportState::load(&state_path)?.expect("checkpoint persisted");
    let mut importer2 = Importer::new(PlatformClient::new(&api_base)?, loaded, state_path.clone());
    let second = importer2.run(&commits).await?;
    assert_eq!(second.imported, 0, "re-run must import nothing");

    let after = repo.property_index("hash").await?;
    assert_eq!(after.len(), 5, "no duplicate entities");

    let emails_after = repo.property_index("email").await?;
    assert_eq!(emails_after.len(), 2);

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&state_path);
    drop_db(&repo).await;
    Ok(())
}

// ------------------------------------------------------------------
// AC-3: crash before checkpoint — resume re-sends; server dedupes.
// ------------------------------------------------------------------
#[tokio::test]
async fn crash_before_checkpoint_resumes_without_duplicates() -> anyhow::Result<()> {
    let _g = guard().await;
    let dir = tmp("ac3");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    fixture(&dir)?;

    let (api_base, repo) = start_platform().await?;

    let since = cutoff_days_ago(250);
    let commits = walk(&dir, since)?;
    let state_path = tmp("ac3-state.json");
    let _ = std::fs::remove_file(&state_path);

    // Crash simulation: import only the first 2 commits with a FRESH state.
    let state = ImportState::new("fixture", "main", "2025-08-11");
    let mut crashed = Importer::new(PlatformClient::new(&api_base)?, state, state_path.clone());
    let partial = crashed.run(&commits[..2]).await?;
    assert_eq!(partial.imported, 2);
    let partial_state = ImportState::load(&state_path)?.expect("final checkpoint saved");
    assert_eq!(partial_state.counts.imported, 2);

    // Resume with a FRESH state: no client-side index — the API dedupes by
    // hash/email, so only the remaining 3 commits import.
    let fresh = ImportState::new("fixture", "main", "2025-08-11");
    let mut resumed = Importer::new(PlatformClient::new(&api_base)?, fresh, state_path.clone());
    let report = resumed.run(&commits).await?;
    assert_eq!(report.imported, 3, "remaining commits imported");

    let hashes = repo.property_index("hash").await?;
    assert_eq!(hashes.len(), 5, "no duplicates after crash-resume");

    let emails = repo.property_index("email").await?;
    assert_eq!(emails.len(), 2, "authors not duplicated by server dedupe");

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&state_path);
    drop_db(&repo).await;
    Ok(())
}

// ------------------------------------------------------------------
// AC-7: git.v1 registered; organic violations warn, never reject.
// ------------------------------------------------------------------
#[tokio::test]
async fn bulk_endpoint_warns_on_git_v1_violations() -> anyhow::Result<()> {
    let (api_base, repo) = start_platform().await?;

    let client = git_importer::pipeline::PlatformClient::new(&api_base)?;
    let entity = git_importer::pipeline::BulkEntity {
        kind: data_graph::EntityKind::Node,
        idempotency_key: "warn-1".to_string(),
        set: data_graph::PropertySet::current(
            data_graph::Scope::Common,
            1,
            std::collections::HashMap::from([
                ("hash".to_string(), serde_json::json!("warn-1")),
                ("message".to_string(), serde_json::json!("")),
                ("bogus".to_string(), serde_json::json!("x")),
            ]),
        ),
    };
    let resp = client.bulk_entities("hash", &[entity]).await?;
    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].status, "imported", "warn-not-fail");
    assert!(
        !resp.warnings.is_empty(),
        "git.v1 lint must warn on unknown property + empty message"
    );

    let hashes = repo.property_index("hash").await?;
    assert!(
        hashes.contains_key("warn-1"),
        "entity persisted despite warnings"
    );

    drop_db(&repo).await;
    Ok(())
}

fn json_eq(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    match (a, b) {
        (serde_json::Value::Number(x), serde_json::Value::Number(y)) => x.as_f64() == y.as_f64(),
        (serde_json::Value::Object(x), serde_json::Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| json_eq(v, w)))
        }
        (serde_json::Value::Array(x), serde_json::Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(v, w)| json_eq(v, w))
        }
        _ => a == b,
    }
}

// ------------------------------------------------------------------
// AC-10: insight post-pass stored; independently recomputed values match.
// ------------------------------------------------------------------
#[tokio::test]
async fn insight_post_pass_matches_recomputation() -> anyhow::Result<()> {
    let _g = guard().await;
    let dir = tmp("ac10");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    fixture(&dir)?;

    let (api_base, repo) = start_platform().await?;

    let since = cutoff_days_ago(250);
    let commits = walk(&dir, since)?;
    let state_path = tmp("ac10-state.json");
    let _ = std::fs::remove_file(&state_path);

    let state = ImportState::new("fixture", "main", "2025-08-11");
    let mut importer = git_importer::pipeline::Importer::new(
        git_importer::pipeline::PlatformClient::new(&api_base)?,
        state,
        state_path.clone(),
    );
    let report = importer.run(&commits).await?;
    assert_eq!(report.imported, 5);
    let window_end = chrono::DateTime::parse_from_rfc3339("2026-02-01T00:00:00+00:00").unwrap();
    let insight_dirs = importer.run_insights(&commits, window_end).await?;
    assert!(insight_dirs >= 2, "c2/c3/c4/c5/c2b touch distinct top dirs");

    let key = format!("git-insight-fixture-{}", "2025-08-11");
    let ids = repo.property_index("hash").await?;
    let insight_ids = ids.get(&key).expect("insight entity persisted");
    let ps = repo.load_property_sets(insight_ids[0]).await?;
    let stored = ps[0].properties["insights"].clone();

    let expected = git_importer::insights::insights_properties(
        &git_importer::insights::compute_dir_insights(&commits, window_end),
    );
    assert!(
        json_eq(&stored, &expected),
        "stored insights == recomputed insights\nstored: {stored}\nexpected: {expected}"
    );

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&state_path);
    drop_db(&repo).await;
    Ok(())
}

// ------------------------------------------------------------------
// REQ-008 bootstrap: GET /graph/snapshot returns the full projection.
// ------------------------------------------------------------------
#[tokio::test]
async fn graph_snapshot_lists_imported_entities() -> anyhow::Result<()> {
    let dir = tmp("snap");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    fixture(&dir)?;

    let (api_base, repo) = start_platform().await?;

    let since = cutoff_days_ago(250);
    let commits = walk(&dir, since)?;
    let state_path = tmp("snap-state.json");
    let _ = std::fs::remove_file(&state_path);

    let state = ImportState::new("fixture", "main", "2025-08-11");
    let mut importer = git_importer::pipeline::Importer::new(
        git_importer::pipeline::PlatformClient::new(&api_base)?,
        state,
        state_path.clone(),
    );
    importer.run(&commits).await?;
    let window_end = chrono::DateTime::parse_from_rfc3339("2026-02-01T00:00:00+00:00").unwrap();
    importer.run_insights(&commits, window_end).await?;

    let resp: serde_json::Value = reqwest::Client::new()
        .get(format!("{api_base}/graph/snapshot"))
        .send()
        .await?
        .json()
        .await?;
    let entities = resp["entities"].as_array().expect("entities array");
    // 5 commits + 2 authors + 1 insight entity
    assert_eq!(entities.len(), 8, "full projection incl. insight entity");
    let commit_entities = entities
        .iter()
        .filter(|e| e["shape"] == "git.v1/Commit")
        .count();
    assert_eq!(commit_entities, 5, "shapes distinguish git entities");
    let author_shapes = entities
        .iter()
        .filter(|e| e["shape"] == "git.v1/Author")
        .count();
    assert_eq!(author_shapes, 2, "authors carry the Author shape");
    let insight_shapes = entities
        .iter()
        .filter(|e| e["shape"] == "git.v1/Insight")
        .count();
    assert_eq!(
        insight_shapes, 1,
        "insight entity carries the Insight shape"
    );

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&state_path);
    drop_db(&repo).await;
    Ok(())
}

#[test]
fn normalize_email_handles_variants() {
    assert_eq!(normalize_email("Alice@Example.COM "), "alice@example.com");
    assert_eq!(normalize_email("bob@example.com"), "bob@example.com");
}
