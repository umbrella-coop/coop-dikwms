//! Live knowledge stream over the TerminusDB commit stream (SPEC-004).
//!
//! Commit messages carry typed tokens:
//! - `create-entity|ent:{entity_id}:{kind}`  -> EntityCreated
//! - `{msg}|ps:{entity}:{instance}:{scope}:v{version}` -> PropertySetSaved
//!
//! `poll_events` walks the commit log from a cursor (genesis = empty) and
//! emits events strictly newer than the cursor, in chronological order.

use uuid::Uuid;

use crate::Repository;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    EntityCreated {
        entity_id: Uuid,
        kind: String,
        commit: String,
    },
    PropertySetSaved {
        entity_id: Uuid,
        instance_id: Uuid,
        scope: String,
        version: u64,
        commit: String,
    },
}

impl DomainEvent {
    /// Commit identifier the event was observed at.
    pub fn commit(&self) -> &str {
        match self {
            DomainEvent::EntityCreated { commit, .. }
            | DomainEvent::PropertySetSaved { commit, .. } => commit,
        }
    }
}

/// Commit identifier marking the last consumed event ("" = genesis).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitCursor(pub String);

/// Parse `ps:{entity}:{instance}:{scope}:v{version}` (4-part, current format).
/// Legacy 3-part tokens (no scope) yield None — they are not stream events.
fn parse_ps_token(token: &str) -> Option<(String, String, String, u64)> {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() < 4 {
        return None;
    }
    Some((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
        parts[3].trim_start_matches('v').parse().ok()?,
    ))
}

fn parse_ent_token(token: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() < 2 {
        return None;
    }
    Some((parts[0].to_string(), parts[1].to_string()))
}

fn decode_entry(identifier: &str, message: &str) -> Option<DomainEvent> {
    if let Some((_, ps)) = message.rsplit_once("|ps:") {
        if let Some((entity, instance, scope, version)) = parse_ps_token(ps) {
            return Some(DomainEvent::PropertySetSaved {
                entity_id: entity.parse().ok()?,
                instance_id: instance.parse().ok()?,
                scope,
                version,
                commit: identifier.to_string(),
            });
        }
        return None;
    }
    if let Some((_, ent)) = message.rsplit_once("|ent:")
        && let Some((id, kind)) = parse_ent_token(ent)
    {
        return Some(DomainEvent::EntityCreated {
            entity_id: id.parse().ok()?,
            kind,
            commit: identifier.to_string(),
        });
    }
    None
}

/// Emit events strictly newer than `cursor`, in chronological order, and
/// advance the cursor to the newest commit seen.
pub async fn poll_events(
    repo: &Repository,
    cursor: &CommitCursor,
) -> anyhow::Result<(Vec<DomainEvent>, CommitCursor)> {
    let log = repo.log().await?;
    let mut collected: Vec<DomainEvent> = Vec::new();
    for entry in &log {
        if !cursor.0.is_empty() && entry.identifier == cursor.0 {
            break; // cursor commit and everything older are already consumed
        }
        if let Some(ev) = decode_entry(&entry.identifier, &entry.message) {
            collected.push(ev);
        }
    }
    collected.reverse(); // log is newest-first; emit chronologically
    let new_cursor = log
        .first()
        .map(|e| CommitCursor(e.identifier.clone()))
        .unwrap_or_else(|| cursor.clone());
    Ok((collected, new_cursor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_ps_token_with_scope() {
        let ev = decode_entry(
            "c1",
            "request-42|ps:11111111-1111-1111-1111-111111111111:22222222-2222-2222-2222-222222222222:Org:v3",
        )
        .unwrap();
        assert_eq!(
            ev,
            DomainEvent::PropertySetSaved {
                entity_id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
                instance_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
                scope: "Org".into(),
                version: 3,
                commit: "c1".into(),
            }
        );
    }

    #[test]
    fn decodes_ent_token() {
        let ev = decode_entry(
            "c2",
            "create-entity|ent:11111111-1111-1111-1111-111111111111:Node",
        )
        .unwrap();
        assert_eq!(
            ev,
            DomainEvent::EntityCreated {
                entity_id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
                kind: "Node".into(),
                commit: "c2".into(),
            }
        );
    }

    #[test]
    fn unknown_messages_are_skipped() {
        assert!(decode_entry("c3", "some other commit").is_none());
        assert!(decode_entry("c4", "legacy|ps:a:b:v1").is_none()); // 3-part, no scope
    }
}
