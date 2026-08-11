//! git-importer CLI (SPEC-027 Phase A).
//!
//! Usage:
//!   git-importer --repo <path> [--since 2025-08-11] [--resume] [--verify]
//!   --state <path>   override checkpoint path (default examples/git-codebase-1/.import-state.json)

#![recursion_limit = "512"]

use std::env;
use std::path::PathBuf;

use anyhow::Context;
use chrono::DateTime;
use git_importer::git::walk;
use git_importer::pipeline::{ImportReport, Importer};
use git_importer::state::ImportState;

const DEFAULT_SINCE: &str = "2025-08-11";
const DEFAULT_STATE: &str = "examples/git-codebase-1/.import-state.json";

struct Config {
    repo: PathBuf,
    since: String,
    resume: bool,
    verify: bool,
    state: PathBuf,
}

fn parse_args(args: &[String]) -> anyhow::Result<Config> {
    let mut repo: Option<PathBuf> = None;
    let mut since = DEFAULT_SINCE.to_string();
    let mut resume = false;
    let mut verify = false;
    let mut state = PathBuf::from(DEFAULT_STATE);

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo" => {
                i += 1;
                repo = Some(PathBuf::from(
                    args.get(i)
                        .ok_or_else(|| anyhow::anyhow!("--repo requires a path"))?,
                ));
            }
            "--since" => {
                i += 1;
                since = args
                    .get(i)
                    .ok_or_else(|| anyhow::anyhow!("--since requires a date"))?
                    .clone();
            }
            "--state" => {
                i += 1;
                state = PathBuf::from(
                    args.get(i)
                        .ok_or_else(|| anyhow::anyhow!("--state requires a path"))?,
                );
            }
            "--resume" => resume = true,
            "--verify" => verify = true,
            other => anyhow::bail!("unknown argument: {other}"),
        }
        i += 1;
    }

    let repo = repo.ok_or_else(|| anyhow::anyhow!("--repo <path> is required"))?;
    Ok(Config {
        repo,
        since,
        resume,
        verify,
        state,
    })
}

fn report_summary(report: &ImportReport) {
    println!(
        "import done: seen={} imported={} skipped={}",
        report.seen, report.imported, report.skipped
    );
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = parse_args(&env::args().skip(1).collect::<Vec<_>>())?;

    let since = DateTime::parse_from_rfc3339(&format!("{}T00:00:00+00:00", config.since))
        .with_context(|| format!("invalid --since date: {}", config.since))?;

    let endpoint = env::var("TERMINUSDB_URL").unwrap_or_else(|_| "http://localhost:6363".into());
    let db = env::var("TERMINUSDB_DB").unwrap_or_else(|_| "git_network".into());
    let pass = env::var("TERMINUSDB_ADMIN_PASS").unwrap_or_else(|_| "root".into());

    let client = terminusdb_client::TerminusDBHttpClient::new(
        url::Url::parse(&endpoint)?,
        "admin",
        &pass,
        "admin",
    )
    .await?;
    let repository = terminusdb_repository::Repository::new(client, db).await?;

    let state = match ImportState::load(&config.state)? {
        Some(s) => s,
        None => ImportState::new("terminusdb/terminusdb", "main", &config.since),
    };

    let mut importer = Importer::new(repository, state, config.state);
    if config.resume {
        importer.reconcile().await?;
        println!("reconcile: index rebuilt from stored property sets");
    }

    let commits = walk(&config.repo, since)?;
    println!(
        "walk: {} commits in window since {} (resume={} verify={})",
        commits.len(),
        config.since,
        config.resume,
        config.verify
    );

    let report = importer.run(&commits).await?;
    report_summary(&report);
    if config.verify {
        println!("verify: runbook assertion step (examples/git-codebase-1/verify.sh)");
    }
    Ok(())
}
