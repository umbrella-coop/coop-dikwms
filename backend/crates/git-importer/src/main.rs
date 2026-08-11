//! git-importer CLI (SPEC-027 Phase A).
//!
//! Usage:
//!   git-importer --repo <path> [--since 2025-08-11] [--resume] [--verify]

use std::env;
use std::path::PathBuf;

const DEFAULT_SINCE: &str = "2025-08-11";

struct Config {
    repo: PathBuf,
    since: String,
    resume: bool,
    verify: bool,
}

fn parse_args(args: &[String]) -> anyhow::Result<Config> {
    let mut repo: Option<PathBuf> = None;
    let mut since = DEFAULT_SINCE.to_string();
    let mut resume = false;
    let mut verify = false;

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
    })
}

fn main() -> anyhow::Result<()> {
    let config = parse_args(&env::args().skip(1).collect::<Vec<_>>())?;
    println!(
        "git-importer plan: repo={} since={} resume={} verify={} (pipeline wiring: SPEC-027 Phase A)",
        config.repo.display(),
        config.since,
        config.resume,
        config.verify
    );
    Ok(())
}
