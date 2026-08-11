//! Import checkpoint state (SPEC-027 REQ-002/REQ-003).
//!
//! The importer owns a `hexsha -> Uuid` index. This is REQUIRED state, not a
//! hint: EntityDoc ids are Uuids, so hash-idempotency depends on this map
//! surviving across runs.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Skipped {
    pub hexsha: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Counts {
    pub seen: u64,
    pub imported: u64,
    pub skipped: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportState {
    pub schema_version: u32,
    pub repo: String,
    pub r#ref: String,
    pub since: String,
    pub hexsha_index: HashMap<String, String>,
    /// Normalized email → author entity Uuid (REQ-002: re-runs must not
    /// duplicate author entities either).
    pub authors_index: HashMap<String, String>,
    pub skipped: Vec<Skipped>,
    pub counts: Counts,
}

impl ImportState {
    /// Create a fresh checkpoint. The cutoff is a fixed date string
    /// (SPEC-027 REQ-001) for reproducible imports.
    pub fn new(repo: &str, r#ref: &str, since: &str) -> Self {
        Self {
            schema_version: 1,
            repo: repo.to_string(),
            r#ref: r#ref.to_string(),
            since: since.to_string(),
            hexsha_index: HashMap::new(),
            authors_index: HashMap::new(),
            skipped: Vec::new(),
            counts: Counts::default(),
        }
    }

    pub fn is_imported(&self, hexsha: &str) -> bool {
        self.hexsha_index.contains_key(hexsha)
    }

    pub fn record_imported(&mut self, hexsha: &str, entity_uuid: uuid::Uuid) {
        self.hexsha_index
            .insert(hexsha.to_string(), entity_uuid.to_string());
        self.counts.imported += 1;
    }

    pub fn record_skipped(&mut self, hexsha: &str, reason: &str) {
        self.skipped.push(Skipped {
            hexsha: hexsha.to_string(),
            reason: reason.to_string(),
        });
        self.counts.skipped += 1;
    }

    /// Persist atomically: write a temp file in the same directory, fsync,
    /// then rename over the target (crash-safe — SPEC-027 REQ-003).
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let dir = path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("import-state.json");
        let tmp = dir.join(format!(".{file_name}.tmp"));
        let json = serde_json::to_vec_pretty(self)?;
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(&json)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn load(path: &Path) -> anyhow::Result<Option<Self>> {
        match fs::read_to_string(path) {
            Ok(raw) => Ok(Some(serde_json::from_str(&raw)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("git-importer-{name}"))
    }

    #[test]
    fn new_state_has_defaults() {
        let s = ImportState::new("terminusdb/terminusdb", "main", "2025-08-11");
        assert_eq!(s.schema_version, 1);
        assert_eq!(s.repo, "terminusdb/terminusdb");
        assert_eq!(s.r#ref, "main");
        assert_eq!(s.since, "2025-08-11");
        assert!(s.hexsha_index.is_empty());
        assert!(s.skipped.is_empty());
        assert_eq!(s.counts, Counts::default());
    }

    #[test]
    fn index_roundtrip_survives_save_load() {
        let path = tmp_path("roundtrip.json");
        let _ = fs::remove_file(&path);
        let mut s = ImportState::new("repo", "main", "2025-08-11");
        s.record_imported("abc123", uuid::Uuid::nil());
        s.record_skipped("def456", "poison");
        s.save(&path).unwrap();

        let loaded = ImportState::load(&path).unwrap().expect("file exists");
        assert_eq!(
            loaded.hexsha_index.get("abc123"),
            Some(&uuid::Uuid::nil().to_string())
        );
        assert_eq!(loaded.skipped.len(), 1);
        assert_eq!(loaded.skipped[0].reason, "poison");
        assert_eq!(loaded.counts.imported, 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn is_imported_reflects_records() {
        let mut s = ImportState::new("repo", "main", "2025-08-11");
        assert!(!s.is_imported("abc123"));
        s.record_imported("abc123", uuid::Uuid::nil());
        assert!(s.is_imported("abc123"));
    }

    #[test]
    fn load_missing_returns_none() {
        let path = tmp_path("missing.json");
        let _ = fs::remove_file(&path);
        assert!(ImportState::load(&path).unwrap().is_none());
    }

    #[test]
    fn save_replaces_existing_content() {
        let path = tmp_path("replace.json");
        let _ = fs::remove_file(&path);
        let mut a = ImportState::new("repo", "main", "2025-08-11");
        a.record_imported("one", uuid::Uuid::nil());
        a.save(&path).unwrap();

        let mut b = ImportState::new("repo", "main", "2025-08-11");
        b.record_imported("two", uuid::Uuid::nil());
        b.save(&path).unwrap();

        let loaded = ImportState::load(&path).unwrap().expect("file exists");
        assert!(loaded.is_imported("two"));
        assert!(!loaded.is_imported("one"));
        let _ = fs::remove_file(&path);
    }
}
