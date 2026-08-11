# ADR-004: Git Commit Graph Edge Representation (speed-run v1)

- **Status:** Accepted (2026-08-11)
- **Source:** SPEC-027 REQ-004 (S2' verification gate), commit 1
- **Standard:** SPEC-027

## Context

The git-domain speed run must represent `Commit → parent Commit` and
`Commit → Author` relationships. Two candidate representations exist:

1. **Edge entities** — persist relationships as `EntityKind::Edge` entities.
2. **Adjacency-list properties** — relationship targets referenced as property
   values on the owning entity (e.g. `parents: ["<uuid>", ...]`).

Verification of the current platform surface:

- `EntityKind::{Node, Edge, Combo}` exists in the domain model
  [Source: Code] backend/crates/data-graph/src/lib.rs:23
- Persistence stores `EntityDoc { id, kind: String }` + JSON property sets —
  an edge is representable only as a `kind = "Edge"` entity whose
  subject/object/relationship live in its JSON properties
  [Source: Code] backend/crates/terminusdb-repository/src/lib.rs:26-44
- The API exposes entity + property-set CRUD only — no edge endpoints, no edge
  semantics in `resolve` [Source: Code] backend/crates/api/src/routes.rs

## Decision

**Adjacency-list properties (v1) for the speed run.** Commit relationships
(`parents`, `author`) are property values referencing entity Uuids. No edge
entities are created by the importer.

Rationale:

- Matches the reference extraction pipeline shape (parent ids as reference
  lists) and the G6 UI's need — edges are built client-side from properties.
- Requires zero platform changes; edge entities would add an unverified
  surface (no edge CRUD, no edge resolution) to a speed run whose goal is
  proving the existing stack.
- `EntityKind::Edge` remains available; nothing in v1 forecloses it.

## Consequences

- G6 graph edges are synthesized from `parents`/`author` properties at render
  time (client-side concern, not platform).
- Graph-traversal queries (e.g. ancestor walks) must follow property
  references — O(depth × refs) reads; acceptable at the 2-3k-commit scale.
- Full edge-entity semantics (dedicated CRUD, edge resolution, edge versioning)
  deferred to P2 — tracked in SPEC-027 delivery constraints.
- Verdict recorded in SPEC-027 REQ-004.

## Alternatives Considered

- **Edge entities now** — rejected: no edge API surface, unverified
  persistence semantics, scope creep for the speed run.
- **Branch/tree-based encoding** — rejected: opaque, breaks the generic
  property-set model.
