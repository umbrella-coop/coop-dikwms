# Brainstorm Report: Core Schema Set & Namespace Registry (SPEC-009 input)

> Generated: 2026-08-07 · BQS v1 · Panel `[degraded]` (single-context) · Rebuttal: all accepted (a)

## Problem Statement
`knowledge-domain` ships only the **core schemas**; everything domain-specific lives in a **multi-namespace versioned schema registry** stored in TerminusDB, added via API/web console in **Protobuf** (package-directive namespaces, immutable field tags). **schema.org is the first namespace package**; core kinds are node/edge/combo + creative work, media object, action.

## HMW Questions
1. Minimal core yet sufficient for all platform features?
2. Node/edge/combo derived from schema.org/Thing without domain leakage?
3. Action types powering SPEC-002 moderation instead of a bespoke enum?
4. Core evolution without breaking wire compat (immutable proto tags)?

## Recommendations (Agg ≥ 3.5, all ✓ passed rebuttal — accepted (a))

**1. Thing as universal base + `additionalType` seam** (4.8) ✓
- Every node/edge/combo IS a schema.org `Thing`; namespace semantics attach via `additionalType` (registry-validated reference at write time).
- Persona: Domain expert · Rebuttal: accepted — additionalType validated against registry.

**2. 5 core kinds with Thing's minimal properties** (4.8) ✓
- Discriminated `kind` (Node/Edge/Combo/CreativeWork/MediaObject/Action) + `name`/`identifier`/`url` only; no more in code.
- Persona: Domain expert · Rebuttal: accepted (kept thin).

**3. Edge/Combo as Things with link semantics** (4.3) ✓
- Edge = Thing + `subject`/`object`/`relationship`; Combo = Thing + members. Core-only extension types (schema.org lacks them).
- Persona: Domain expert · Rebuttal: accepted.

**4. Action records as moderation events** (4.7) ✓ **REVISED per user**
- schema.org `Action`/`AssessAction` tree (ChooseAction→VoteAction, IgnoreAction, ReactAction→Like/Disagree/Endorse..., ReviewAction) records **user intent** for SPEC-002.
- **User amendment:** record actions in **PostgreSQL**; **PG19 native SQL/PGQ** enables graph queries over AssessAction→moderation graphs (e.g. endorsement networks). Ledger status (SPEC-002) remains the workflow state — two projections of one event.
- Persona: End-user · Rebuttal: accepted + amended.

**5. v1 = exactly two namespaces** (4.3) ✓
- `core` + `org.schema.v1` (schema.org subset). Multi-namespace tooling designed-in, console v1-scoped.
- Persona: Cost · Rebuttal: accepted.

**6. Registry drives UI generation** (4.3, contested) ✓
- Property metadata (type/label) renders generic antd drawer forms for any namespace type.
- Persona: End-user · Rebuttal: accepted — metadata-driven generic form renderer.

**7. Curated, pinned schema.org subset** (4.1) ✓
- `org.schema.v1` pins Thing/CreativeWork/MediaObject/Action property subsets — not upstream HEAD (schema.org is large & unstable).
- Persona: Skeptic · Rebuttal: accepted — pinned + import script + conformance checks.

**8. Tag immutability lint** (4.1) ✓
- protoc descriptor sets + lightweight tag-reuse check (buf optional).
- Persona: Cost · Rebuttal: accepted.

**9. Record vs state separation** (4.0) ✓
- Action = intent record; ledger = workflow state. SPEC-002 state machine unchanged.
- Persona: Skeptic · Rebuttal: accepted.

**10. Namespace explorer** (4.0) ✓
- Console browse/search surface for `additionalType`; ships with console.
- Persona: End-user · Rebuttal: accepted.

**11. Core tag budget 1–15** (3.8) ✓
- Policy + lint rule; varint-friendly; avoids future migration.
- Persona: Skeptic · Rebuttal: accepted.

**12. Properties as resolvable IRIs** (3.8) ✓
- Property IRIs = registry lookup keys (enables #6/#7).
- Persona: Analogist · Rebuttal: accepted.

## Deferred (user ruling)
- **D3 derived-artifact cache** (3.8) → later spec
- **C1 structural conformance** (3.7) → later spec
- **Multi-namespace console/tooling** → after schema.org proves out
- **Cross-domain mapping** (OCSF, healthcare, FiBO) → long-term goal; schema.org is the seed standard for alignment

## Seeds (killed ideas)
- **C2 NCBI accession analogy** → real problem: stable term identity → covered by #12 (IRIs) + tag immutability

## User Additions (accepted into recommendations)
1. **E1 amendment:** AssessAction moderation data in PostgreSQL (PG19 SQL/PGQ for graph queries over moderation/assess graphs) — `[user-provided fact; verify PG19 SQL/PGQ native support]`
2. **Registry UX:** organizations define their supported schema namespaces; selectable **per workspace and project** (fits SPEC-003 scope instances)
3. **Metadata property:** each entity carries a metadata property storing the **native domain datastructure + schema versioning** (registry version pointer)
4. **Risk note:** multi-schema-namespace complexity is an explicit RISK — start schema.org-only

## Next Steps
- [ ] Update SPEC-009 backlog entry with these decisions
- [ ] Draft SPEC-009: registry data model (FileDescriptorSet + derived schemas in TerminusDB), core package (proto), schema.org seed namespace, metadata property, per-workspace/project namespace selection
- [ ] `/sdd` create for SPEC-009 when ready
