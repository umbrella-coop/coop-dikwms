//! git-importer: SPE-027 Phase A ingestion pipeline (durable primitives).

pub mod git;
pub mod retry;
pub mod state;

use chrono::{DateTime, FixedOffset};

/// True when `authored_at` is on or after the cutoff (SPEC-027 REQ-001).
pub fn in_window(authored_at: DateTime<FixedOffset>, since: DateTime<FixedOffset>) -> bool {
    authored_at >= since
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(s: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(s).unwrap()
    }

    #[test]
    fn window_includes_cutoff_and_after() {
        let since = dt("2025-08-11T00:00:00+00:00");
        assert!(in_window(dt("2025-08-11T00:00:00+00:00"), since));
        assert!(in_window(dt("2026-01-01T12:00:00+00:00"), since));
    }

    #[test]
    fn window_excludes_before_cutoff() {
        let since = dt("2025-08-11T00:00:00+00:00");
        assert!(!in_window(dt("2025-08-10T23:59:59+00:00"), since));
        assert!(!in_window(dt("2020-01-01T00:00:00+00:00"), since));
    }

    #[test]
    fn window_is_timezone_aware() {
        let since = dt("2025-08-11T00:00:00+00:00");
        let same_instant_other_zone =
            DateTime::parse_from_rfc3339("2025-08-10T17:00:00-07:00").unwrap();
        assert!(in_window(same_instant_other_zone, since));
    }
}
