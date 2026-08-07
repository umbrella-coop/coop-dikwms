# Events (SPEC-004)

Live knowledge events over the commit stream (`GET /events?cursor=`):

- `entity_created` — `{ entity_id, kind, commit }`
- `property_set_saved` — `{ entity_id, instance_id, scope, version, commit }`

Commit-message tokens: `ps:` (property set), `ent:` (entity), `corr:` (correlation), `rev:` (revert target). Tokens are immutable — see AGENTS.md token discipline.
