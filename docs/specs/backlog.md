# Spec Backlog

Planned specs not yet created. Add requirements captured during development here so they are not lost.

## SPEC-005 (planned): Graph Primitive Merge

**Requirement (captured 2026-08-06):** Nodes, edges, and combos can be **merged** — two or more graph primitives collapse into one entity.

**Key design questions to resolve in the spec:**
- Identity: merged entity takes which UUID? (new UUID + provenance, or survivor's UUID?)
- Content: how scoped property sets (SPEC-001) combine — union/conflict resolution per scope layer
- Moderation: does a merge require a change request (SPEC-002) and record provenance?
- Attachments: entity scope attachments (SPEC-003) union?
- History: prior versions/entities retained for audit?
- Relationship to soft-delete and promotion ladder
- Downstream impact: edges referencing merged nodes must be rewired

**Dependencies:** SPEC-001 (identity), SPEC-002 (moderation), SPEC-003 (attachments)

## SPEC-006 (planned): TerminusDB Git-Like Version Control & Temporal Architecture

**Requirement (captured 2026-08-06):** Leverage TerminusDB's Git-like version control — combining **JSON** (document) and **RDF** (graph) data models with high-precision **temporal reasoning** — to improve the platform's architecture and design patterns.

**Candidate architectural gains to evaluate in the spec:**
- **History (SPEC-002 R-3):** replace in-memory per-scope `history` vectors with TerminusDB commits/branches (time-travel, diff, rollback for free)
- **Promotion ladder (SPEC-002):** map promote/adopt onto branch → merge → commit semantics (brainstorm idea #3 "Git-for-Knowledge-Layers")
- **Provenance (SPEC-002):** commit metadata as the audit trail instead of separate `Provenance` records
- **Temporal reasoning:** version-as-of queries for `resolve(entity, scope, at: DateTime)` — time-travel resolution across scope layers
- **JSON+RDF dual model:** scoped property sets as JSON documents; edges/combo topology as RDF triples — schema decisions for SPEC-001 R-2 and cross-org sharing queries (SPEC-003)
- **Conflict detection:** TerminusDB merge conflict handling as the basis for competing-promotion resolution

**Gate:** ~~blocked on R-1~~ — **R-1 partially verified** (2026-08-06, github.com/terminusdb/terminusdb README): commits/diff/push-pull ✓, time-travel ✓, Allen-interval temporal reasoning ✓, JSON Git-for-Data ✓, WOQL/GraphQL/REST ✓, Rust client ✓; subscriptions ✗. Remaining unknowns: `@to` schema specifics, Rust client maturity, subscription availability — verify hands-on via local Docker server before spec creation.

**Dependencies:** SPEC-001 (R-2 schema pipeline), SPEC-002 (R-3 history, promotion), SPEC-003 (scope queries)

## SPEC-007 (planned): AI Code Quality Gauntlet

**Requirement (captured 2026-08-06):** Operationalize Robert C. Martin's ("Uncle Bob") approach to AI-generated code: instead of line-by-line human review, subject AI output to an automated "gauntlet" of strict static analysis, behavioral constraints, and verification gates.

**Candidate gates to define in the spec:**
- **Static quality & structure:**
  - Cyclomatic complexity ceilings (max conditional branching per function)
  - Dependency & coupling analysis (no circular deps, no architectural boundary violations)
  - Module/function limits (file length, function line count, argument count — SRP enforcement)
- **Testing & verification:**
  - Mutation score threshold (inject mutants; tests must catch them) — aligns with existing `.standards/mutation-testing.ai.yaml`
  - Code & branch coverage thresholds (line + conditional path coverage) — aligns with `.standards/full-coverage-testing.ai.yaml`
  - Executable behavioral acceptance tests (Gherkin/BDD scenarios run in CI) — aligns with `features/*.feature` + `.standards/behavior-driven-development.ai.yaml`
- **Philosophy shift:** humans set structural constraints + acceptance suites; AI code passes gates automatically — humans act as system architects, not line readers

**Tooling questions for the spec:** cargo-geiger / cargo-cyclomatic-complexity / rustfmt/clippy-as-gauntlet, cargo-mutants (mutation), tarpaulin/llvm-cov (coverage), cucumber-rs (Gherkin execution), CI wiring (UDS `test`/`lint`/`security` commands in `uds.project.yaml`).

**Dependencies:** none blocking (independent of R-1/R-2); requires CI wiring

## SPEC-008 (planned): terminusdb-rs Fork — JS-Client Feature Parity

**Requirement (captured 2026-08-06):** Implement the missing TerminusDB Rust-client features that the TypeScript/JavaScript client already has, in a **fork at https://github.com/gustavorps/terminusdb-rs** (fork of ParapluOU/terminusdb-rs), added as a **git submodule**.

**Features to port from the JS client (per ParapluOU repo README "Future Development"):**
- Branch management operations
- Push/pull/clone (remote database operations)
- Streaming operations
- Patch/diff operations
- Schema migration tools
- Advanced authentication methods

**Also blocking/supporting:**
- Unblocks SPEC-006 R-9 (branch-per-scope promotion mapping) and SPEC-004 (streaming)
- Reduces SPEC-006 R-7 (Rust client maturity risk) — forked dependency, in-repo

**Workflow:** add submodule (e.g. `third_party/terminusdb-rs`) → spec per feature (SDD) → TDD against the real server (Docker) or recorded fixtures
