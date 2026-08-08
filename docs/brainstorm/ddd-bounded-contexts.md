# Brainstorm Report: DDD Bounded Contexts & Generated DDD Artefacts

> Generated: 2026-08-07 · BQS v1 · Rebuttal: all accepted (a) — APPROVED v1
> Input: user prompt on SPEC-017/SPEC-018 (frontend organization, bit generator gap)

## Problem Statement
Organization follows *component type* (ui/, hooks/, apps/) but humans and AI agents navigate by *what they are doing* (bounded context). The platform needs a **context map**, a **ubiquitous language**, and **context-first directory/naming rules** that generated artefacts (`bit create` output) conform to.

## Approved v1 (the contract)

### 1. Context Map
| Bounded context | Backend container | Frontend namespace | Purpose |
|---|---|---|---|
| Graph Knowledge | `knowledge-domain` | `canvas/` | entities, property sets, resolve, G6 rendering |
| Governance | moderation ledger + audit (repository) | `governance/` (future moderation UI) | change requests, decisions, audit |
| Access | `knowledge-domain` Policy | `access/` (future) | scope instances, ACL, principals |
| Registry | `schema-registry` | `registry/` (future console) | namespaces, additionalType |
| Stream | `stream-api` | `stream/` | live events, SSE |
| Workspace Shell | `api` (transport only) | `workspace/` | app shells composing contexts |

### 2. Directory Rule (context-first)
- `frontend/network-graph/<context>/<component>` — e.g. `canvas/graph`, `canvas/select-layout`, `editor/entity-drawer`, `stream/use-event-stream`, `workspace/coop-graph-app`
- Backend crates remain the context containers (no crate re-split in v1)
- Anti-corruption: the API is a translation layer; components communicate via interfaces, never direct cross-context imports (documented rule; CI import check deferred)

### 3. Naming Rule
- Component ids: `<context>/<component>` kebab-case (e.g. `canvas/select-layout`)
- `data-testid`: context-prefixed kebab-case, descriptive of intent (e.g. `graph-select-layout`, `entity-drawer-form`)
- Conventions apply to NEW components; incremental migration list for existing (testids are a stable contract — no mass churn)

### 4. Generator Policy
- **Post-create lint** over generator wrappers (v1) — a script validates a component conforms (testids, layout, spec presence); revisit generators at 10+ components

### 5. Artefacts to codify
- UDS options: `frontend-component-conventions` + `ddd-bounded-context-org`
- ADRs: bounded-context map, frontend component layout, testid naming
- Glossary stub (ubiquitous language): node, edge, combo, property set, scope instance, change request, promotion, namespace, context

## Deferred
- CI import-boundary checks (C2)
- Generator customization (revisit at 10+ components)
- Governance/Access/Registry frontend namespaces (until those UIs exist)

## Next Steps
- [x] Approval gate passed
- [x] Codify UDS options + ADRs (Actions 2–3)
- [ ] Restructure `frontend/network-graph/` to context-first layout (`bit move` batch)
