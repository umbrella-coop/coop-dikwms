//! git2-backed history walk (SPEC-027 REQ-001/REQ-004).

use std::collections::BTreeSet;
use std::path::Path;

use chrono::{DateTime, FixedOffset};

use crate::in_window;

#[derive(Debug, Clone, PartialEq)]
pub struct GitAuthor {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GitCommit {
    pub hexsha: String,
    pub message: String,
    pub authored_at: DateTime<FixedOffset>,
    pub author: GitAuthor,
    pub parents: Vec<String>,
    /// Paths touched by this commit (tree diff vs first parent; root commits
    /// diff against the empty tree). No file nodes are created — paths feed
    /// dir-level analytics (SPEC-027 REQ-004).
    pub paths: Vec<String>,
}

/// Walk all commits reachable from HEAD, newest-first, keeping only commits
/// at/after `since` (fixed cutoff — reproducible imports).
pub fn walk(repo_path: &Path, since: DateTime<FixedOffset>) -> anyhow::Result<Vec<GitCommit>> {
    let repo = git2::Repository::open(repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
    revwalk.push_head()?;

    let mut out = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let authored_at = commit_time(commit.time());
        if !in_window(authored_at, since) {
            continue;
        }
        out.push(GitCommit {
            hexsha: oid.to_string(),
            message: commit.message().unwrap_or("").trim().to_string(),
            authored_at,
            author: GitAuthor {
                name: commit.author().name().unwrap_or("").to_string(),
                email: commit.author().email().unwrap_or("").to_string(),
            },
            parents: commit.parent_ids().map(|id| id.to_string()).collect(),
            paths: touched_paths(&repo, &commit)?,
        });
    }
    Ok(out)
}

fn commit_time(time: git2::Time) -> DateTime<FixedOffset> {
    let offset_minutes = time.offset_minutes();
    let offset = FixedOffset::east_opt(offset_minutes * 60)
        .unwrap_or_else(|| FixedOffset::east_opt(0).expect("zero offset"));
    DateTime::from_timestamp(time.seconds(), 0)
        .expect("commit timestamp out of range")
        .with_timezone(&offset)
}

fn touched_paths(repo: &git2::Repository, commit: &git2::Commit) -> anyhow::Result<Vec<String>> {
    let tree = commit.tree()?;
    let parent_tree = commit.parents().next().map(|p| p.tree()).transpose()?;
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let mut paths = BTreeSet::new();
    for delta in diff.deltas() {
        match delta.new_file().path() {
            Some(p) => {
                paths.insert(p.to_string_lossy().to_string());
            }
            None => {
                if let Some(p) = delta.old_file().path() {
                    paths.insert(p.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(paths.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn days_ago(days: i64) -> i64 {
        chrono::Utc::now().timestamp() - days * 86_400
    }

    fn sig(name: &str, email: &str, days: i64) -> git2::Signature<'static> {
        git2::Signature::new(name, email, &git2::Time::new(days_ago(days), 0)).unwrap()
    }

    /// Linear fixture: commit i at `days_ago[i]`, touching `file{i}.txt`.
    fn fixture(dir: &Path, days_ago: &[i64]) -> anyhow::Result<git2::Repository> {
        let repo = git2::Repository::init(dir)?;
        let mut index = repo.index()?;
        for (i, days) in days_ago.iter().enumerate() {
            let filename = format!("file{i}.txt");
            std::fs::write(dir.join(&filename), format!("content {i}\n"))?;
            index.add_path(Path::new(&filename))?;
            index.write()?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            let parent_commits: Vec<git2::Commit> = if i == 0 {
                Vec::new()
            } else {
                vec![repo.head()?.peel_to_commit()?]
            };
            let parent_refs: Vec<&git2::Commit> = parent_commits.iter().collect();
            repo.commit(
                Some("HEAD"),
                &sig("Alice", "alice@example.com", *days),
                &sig("Alice", "alice@example.com", *days),
                &format!("commit {i}"),
                &tree,
                &parent_refs,
            )?;
        }
        Ok(repo)
    }

    fn tmp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("git-importer-fixture-{name}"))
    }

    #[test]
    fn walk_filters_by_cutoff_newest_first() {
        let dir = tmp("window");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        fixture(&dir, &[400, 300, 200, 100]).unwrap();

        let since = chrono::Utc
            .timestamp_opt(days_ago(250), 0)
            .unwrap()
            .fixed_offset();
        let commits = walk(&dir, since).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].message, "commit 3");
        assert_eq!(commits[1].message, "commit 2");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn walk_populates_fields_and_paths() {
        let dir = tmp("fields");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        fixture(&dir, &[10, 5]).unwrap();

        let since = chrono::Utc
            .timestamp_opt(days_ago(1_000), 0)
            .unwrap()
            .fixed_offset();
        let commits = walk(&dir, since).unwrap();
        assert_eq!(commits.len(), 2);

        let newest = &commits[0];
        assert_eq!(newest.author.email, "alice@example.com");
        assert_eq!(newest.parents.len(), 1);
        assert_eq!(newest.parents[0], commits[1].hexsha);
        assert!(newest.paths.contains(&"file1.txt".to_string()));

        let root = &commits[1];
        assert!(root.parents.is_empty());
        assert!(root.paths.contains(&"file0.txt".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn walk_empty_when_cutoff_in_future() {
        let dir = tmp("future");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        fixture(&dir, &[10]).unwrap();

        let since = chrono::Utc
            .timestamp_opt(days_ago(-1), 0)
            .unwrap()
            .fixed_offset();
        assert!(walk(&dir, since).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
