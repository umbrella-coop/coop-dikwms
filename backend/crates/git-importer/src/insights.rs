//! Wisdom-layer insight post-pass (SPEC-027 REQ-010).
//!
//! Pure computation over the walked commits: per top-level directory —
//! ownership concentration, stale-knowledge zones, bus-factor risk. Stored as
//! property sets on a per-repo insight entity (Wisdom = data, queryable).

use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};

use crate::git::GitCommit;

/// A dir is stale when its most recent activity predates the last 90 days of
/// the import window (`window_end` = import time, an external anchor — not the
/// newest commit); bus-factor risk additionally requires >=80% of commits
/// from a single author.
const STALE_DAYS: i64 = 90;
const BUS_FACTOR_SHARE: f64 = 0.8;

#[derive(Debug, Clone, PartialEq)]
pub struct DirInsight {
    pub dir: String,
    pub commits: usize,
    pub authors: usize,
    pub dominant_author: Option<String>,
    pub dominant_share: f64,
    pub last_active: Option<DateTime<FixedOffset>>,
    pub stale: bool,
    pub bus_factor_risk: bool,
}

/// Top-level directory of a touched path (first segment); empty/root paths
/// group under "(root)".
pub fn top_dir(path: &str) -> String {
    path.split('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("(root)")
        .to_string()
}

/// Normalized Shannon entropy over author shares (0 = single author, 1 = fully
/// distributed).
pub fn author_entropy(authors: &[String]) -> f64 {
    let total = authors.len() as f64;
    if total == 0.0 {
        return 0.0;
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for a in authors {
        *counts.entry(a).or_insert(0) += 1;
    }
    let mut h = 0.0;
    for count in counts.values() {
        let p = *count as f64 / total;
        h -= p * p.ln();
    }
    let n = counts.len() as f64;
    if n <= 1.0 {
        return 0.0;
    }
    h / n.ln()
}

pub fn compute_dir_insights(
    commits: &[GitCommit],
    window_end: DateTime<FixedOffset>,
) -> Vec<DirInsight> {
    let mut by_dir: HashMap<String, Vec<&GitCommit>> = HashMap::new();
    for commit in commits {
        let mut seen: Vec<String> = commit.paths.iter().map(|p| top_dir(p)).collect();
        if seen.is_empty() {
            seen.push("(root)".to_string());
        }
        seen.sort();
        seen.dedup();
        for dir in seen {
            by_dir.entry(dir).or_default().push(commit);
        }
    }

    let stale_cutoff = window_end - chrono::Duration::days(STALE_DAYS);

    let mut out: Vec<DirInsight> = by_dir
        .into_iter()
        .map(|(dir, commits)| {
            let authors: Vec<String> = commits
                .iter()
                .map(|c| crate::pipeline::normalize_email(&c.author.email))
                .collect();
            let mut counts: HashMap<&str, usize> = HashMap::new();
            for a in &authors {
                *counts.entry(a).or_insert(0) += 1;
            }
            let total = commits.len() as f64;
            // Deterministic tie-break: equal counts resolve to the
            // lexicographically smallest email — HashMap iteration order must
            // never leak into stored insights (deterministic imports).
            let (dominant_author, dominant_share) = counts
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
                .map(|(a, c)| (Some(a.to_string()), *c as f64 / total))
                .unwrap_or((None, 0.0));
            let last_active = commits.iter().map(|c| c.authored_at).max();
            let stale = last_active.is_some_and(|t| t < stale_cutoff);
            let bus_factor_risk = dominant_share >= BUS_FACTOR_SHARE && stale;
            DirInsight {
                dir,
                commits: commits.len(),
                authors: counts.len(),
                dominant_author,
                dominant_share,
                last_active,
                stale,
                bus_factor_risk,
            }
        })
        .collect();
    out.sort_by_key(|i| std::cmp::Reverse(i.commits));
    out
}

/// Serialized form stored on the insight entity property set.
pub fn insights_properties(insights: &[DirInsight]) -> serde_json::Value {
    let map: HashMap<String, serde_json::Value> = insights
        .iter()
        .map(|i| {
            (
                i.dir.clone(),
                serde_json::json!({
                    "commits": i.commits,
                    "authors": i.authors,
                    "dominant_author": i.dominant_author,
                    "dominant_share": i.dominant_share,
                    "last_active": i.last_active.map(|t| t.to_rfc3339()),
                    "stale": i.stale,
                    "bus_factor_risk": i.bus_factor_risk,
                }),
            )
        })
        .collect();
    serde_json::json!(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::GitAuthor;

    fn commit(hexsha: &str, email: &str, days: i64, paths: &[&str]) -> GitCommit {
        GitCommit {
            hexsha: hexsha.to_string(),
            message: "m".to_string(),
            authored_at: DateTime::parse_from_rfc3339(&format!(
                "2026-01-{:02}T00:00:00+00:00",
                (days % 28).max(1)
            ))
            .unwrap(),
            author: GitAuthor {
                name: email.to_string(),
                email: email.to_string(),
            },
            parents: vec![],
            paths: paths.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn top_dir_first_segment() {
        assert_eq!(top_dir("src/lib.rs"), "src");
        assert_eq!(top_dir("docs/specs/a.md"), "docs");
        assert_eq!(top_dir("README.md"), "README.md");
        assert_eq!(top_dir(""), "(root)");
    }

    #[test]
    fn entropy_single_author_zero() {
        assert_eq!(author_entropy(&["a".to_string(), "a".to_string()]), 0.0);
    }

    #[test]
    fn entropy_two_balanced() {
        let h = author_entropy(&["a".to_string(), "b".to_string()]);
        assert!((h - 1.0).abs() < 1e-9, "balanced = 1, got {h}");
    }

    #[test]
    fn bus_factor_risk_flags_dominant_dormant() {
        // 10 commits in "src": 9 by alice, 1 by bob — all in early January,
        // window ends 2026-05-31 (cutoff 2026-03-02) → nothing in the last
        // 90d → stale + risk.
        let window_end = DateTime::parse_from_rfc3339("2026-05-31T00:00:00+00:00").unwrap();
        let commits: Vec<GitCommit> = (0..9)
            .map(|i| commit(&format!("a{i}"), "alice@example.com", i + 1, &["src/x.rs"]))
            .chain(std::iter::once(commit(
                "b0",
                "bob@example.com",
                2,
                &["src/y.rs"],
            )))
            .collect();
        let insights = compute_dir_insights(&commits, window_end);
        assert_eq!(insights.len(), 1);
        let src = &insights[0];
        assert_eq!(src.commits, 10);
        assert_eq!(src.dominant_author.as_deref(), Some("alice@example.com"));
        assert!((src.dominant_share - 0.9).abs() < 1e-9);
        assert!(src.stale, "no activity in last 90d of window");
        assert!(src.bus_factor_risk, "90% share + stale");
    }

    #[test]
    fn active_dir_not_stale() {
        let window_end = DateTime::parse_from_rfc3339("2026-02-01T00:00:00+00:00").unwrap();
        let commits = vec![commit("a0", "alice@example.com", 28, &["src/x.rs"])];
        let insights = compute_dir_insights(&commits, window_end);
        assert!(!insights[0].stale);
        assert!(!insights[0].bus_factor_risk);
    }

    #[test]
    fn multi_dir_grouping_and_sort() {
        let window_end = DateTime::parse_from_rfc3339("2026-02-01T00:00:00+00:00").unwrap();
        let commits = vec![
            commit("a0", "alice@example.com", 5, &["src/a.rs", "docs/d.md"]),
            commit("a1", "alice@example.com", 4, &["docs/e.md"]),
        ];
        let insights = compute_dir_insights(&commits, window_end);
        assert_eq!(insights.len(), 2);
        assert_eq!(insights[0].dir, "docs", "sorted by commit count desc");
        assert_eq!(insights[0].commits, 2);
        assert_eq!(insights[1].dir, "src");
    }
}
