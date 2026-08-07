# Brainstorm Report: Evolving Knowledge Graph Platform (G6 · Rust · TerminusDB · Bit)

> Generated: Aug 2026 · BQS v1 (Layer 0-3) · Panel `[degraded]` (baseline single-context, D4 not passed)

## Problem Statement

Governed, multi-scope evolving knowledge graph: graph primitives (node/edge/combo) are **unique in identity**, **shared across orgs**, and **scoped in content** (common/org/workspace/project). Each scope holds its own truth that can *adopt parent versions* or *promote its own* — every layer change flows through a **request-to-change moderation process**. Live graph updates render in G6, with antd/antv widgets composed into nodes/combos.

Root cause (5 Whys): *shared truth must be stable, local truth must be adaptable* — the hard problem is the **identity + governance model**, not the visualization.

## HMW Questions

1. One entity, unique identity, scoped content across org boundaries?
2. Child scopes adopt parent truth OR promote their own — safely & auditably?
3. Stream live graph changes to G6 without redraw storms?
4. Compose antd drawer widgets into nodes/combos without coupling bounded contexts?

## Recommendations (Agg ≥ 3.5, uncapped)

Rebuttal: all accepted (a) → modified versions. Scores: Eng/User/Strat critics, Agg = mean.

| # | Idea | Eng | User | Strat | Agg |
|---|------|-----|------|-------|-----|
| 1 | Identity/Content Separation | 3.5 | 5.0 | 5.0 | 4.5 |
| 2 | Safe-Edit Sandbox + Fast-Track Tiers | 3.5 | 5.0 | 5.0 | 4.5 |
| 3 | Git-for-Knowledge-Layers | 2.5 | 5.0 | 5.0 | 4.2 |
| 4 | In-Canvas Propose+Diff | 2.5 | 5.0 | 5.0 | 4.2 |
| 5 | CDC + Cursor Streaming | 3.5 | 4.3 | 4.6 | 4.1 |
| 6 | Hierarchy-as-Graph | 4.0 | 4.0 | 4.0 | 4.0 |
| 7 | Activity Stream | 4.0 | 4.0 | 4.0 | 4.0 |
| 8 | Single-Store v1 | 5.0 | 3.3 | 3.6 | 4.0 |
| 9 | Event-Sourced Writes (deferred v2) | 2.5 | 4.3 | 4.6 | 3.8 |
| 10 | Provenance-as-Edges | 3.0 | 4.0 | 4.0 | 3.7 |
| 11 | Idempotent Moderation Apply | 4.0 | 3.3 | 3.6 | 3.6 |
| 12 | Schema-First Codegen | 4.0 | 3.3 | 3.6 | 3.6 |

### Per-idea detail (D5–D8)

**1. Identity/Content Separation** ✓
- Identity = pure global UUID; scope-resolved access is a `resolve(scope) → propertySet` function with cached per-scope projections (RustForge-composable, no inheritance). Promotion = versioned copy of a scoped property set upward.
- D5: `[hypothesis]` resolution-as-function; TerminusDB multi-doc capabilities `[to-verify, site HTTP 522]`
- D6: solves cross-org sharing without identity collisions; not doing it ⇒ entity forked per org. Lagging: cross-org shared-node adoption
- D7: `[need to spike resolve() first]`
- D8: spike `resolve(scope)` on 2 scopes + schema.org model

**2. Safe-Edit Sandbox + Fast-Track Tiers** ✓
- Editing a synced node auto-creates a workspace proposal; small/auto-safe edits route to fast-track auto-approval.
- D6: kills "someone changed my shared graph" fear + keeps review queues sane. Lagging: proposal rejection rate
- D7: `[falsifiable now]` rejection rate measurable
- D8: define fast-track policy rules

**3. Git-for-Knowledge-Layers** ✓
- Property-set diff on the node drawer (before/after) in v1, not topology diff; conflict = two children promoting incompatible versions of the same property.
- D6: promotion ladder is the product's heart. Lagging: promote/adopt rate per scope
- D7: `[need to spike property-diff first]`
- D8: spike attribute-set diff + merge

**4. In-Canvas Propose+Diff** ✓
- v1 = side-by-side property table + "Compare with parent" chip in the antd drawer; mini-graph diff deferred.
- D6: moderation as workflow, not bureaucracy. Lagging: proposal completion rate
- D7: `[falsifiable now]`
- D8: wire propose action into drawer component

**5. CDC + Cursor Streaming** ✓
- Postgres LISTEN/NOTIFY (or Debezium) + domain-event cursor (node created, property promoted, combo regrouped); polling fallback if TerminusDB subscriptions unavailable.
- D6: live streaming is a stated requirement. Lagging: end-to-end event latency
- D7: `[falsifiable now]` latency under load
- D8: define event schema + cursor protocol

**6. Hierarchy-as-Graph** ✓
- org/ws/project as scope nodes with `memberOf` edges; ACL attached to scope nodes (reified); "shared across orgs" = path query.
- D6: cross-org sharing as a query, not a join. Lagging: ACL leak incidents (target 0)
- D7: `[falsifiable now]` ACL-leak test
- D8: model scope nodes + ACL matrix

**7. Activity Stream** ✓
- Moderation lifecycle events only (created/approved/rejected/adopted); doubles as audit feed.
- D6: trust loop visibility. Lagging: notification click-through
- D7: `[falsifiable now]`
- D8: subscribe stream to moderation events

**8. Single-Store v1** ✓
- TerminusDB = everything graph+moderation; Postgres = auth/sessions only. Event-sourcing deferred to v2.
- D6: fastest coherent path to first render. Lagging: time-to-first-node-rendered
- D7: `[falsifiable now]` 1 feature (promotion) ships on single store
- D8: verify TerminusDB live-update capability before committing

**9. Event-Sourced Writes** (contested) ✓ → **deferred v2**
- v2 target: Postgres event log doubles as audit trail; TerminusDB becomes materialized query projection.
- D6: moderation is an audit problem. Lagging: audit-reconstruction time
- D7: `[need to ship v1 first]`
- D8: defer; revisit after v1 promotion ships

**10. Provenance-as-Edges** ✓
- Materialized in `meta:` side namespace, excluded from render queries; same store, separate index.
- D6: full lineage for governance. Lagging: audit query completeness
- D7: `[falsifiable now]` lineage query returns full chain
- D8: side namespace + index config

**11. Idempotent Moderation Apply** ✓
- Version-checked apply as stored procedure; insurance against double-apply/rejection races.
- D6: correctness of governance path. Lagging: double-apply bug count (target 0)
- D7: `[falsifiable now]` double-apply test
- D8: include in moderation command handler

**12. Schema-First Codegen** ✓
- Single source schema (schema.org); generate TerminusDB `@to` schema + Rust DTOs as build artifacts.
- D6: kills hand-written serialization drift. Lagging: schema-drift breakage count
- D7: `[falsifiable now]` regenerate on schema change, build passes
- D8: pick schema.org subset + codegen toolchain

## Contested Zone

- Idea 3 Git-for-Layers → kept (property-diff v1)
- Idea 9 Event-Sourcing → kept as deferred v2 target
- Idea 4 In-Canvas Propose → kept (table-diff v1)

## Seeds (from killed ideas)

- **Idea 8 (CRDT):** over-engineered for low-concurrency writes → real problem: multi-org conflicts need a **policy** (last-writer + approval gate), not a math solution.
- **Idea 9 (Genome model):** subsumed by #1 → real problem: the promotion ladder needs shared **vocabulary** (promote/adopt/fork) — define in CONTEXT.md early.
- **Idea 12 (ProComponents):** low impact to plan around → real problem: keep moderation UI cheap with antd primitives as implementation detail.

## Judgment Override

*None claimed.*

## Diversity Note

Recommended Set spans 5 personas × 2 lenses — no single-cluster collapse.

## Discarded Ideas

| Idea | Reason |
|------|--------|
| 8. CRDT scoped registers | 3.3 — cost/complexity >> need; policy suffices |
| 9. "One genome, transcriptomes" | 3.3 — subsumed by #1 |
| 12. antd ProComponents moderation UI | 3.3 — implementation detail, not a feature |

## Grounding footnote

- Bit commands (`bit init`, `bit create react-app`, `bit tag`, `bit export`, module-federation/UMD micro-frontends): verified against https://bit.dev/docs/intro (Accessed Aug 2026)
- TerminusDB claims (versioning, `@to`, WOQL, subscriptions): **unverified — terminusdb.com returned HTTP 522 at fetch time**; confirm before committing to single-store v1.

## Next Steps

- [ ] Run `/sdd` to turn the spine (#1 identity/scope · #2 sandbox · #3 git-layers · #8 single-store) into a spec
- [ ] Verify TerminusDB capabilities (site was down) → gates #8/#10
- [ ] **Calibration (lagging):** record after 3 sessions — Adoption Rate / Diversity / Cognitive Load
