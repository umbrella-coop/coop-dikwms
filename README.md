# Branch: `main`

**Integration / documentation backbone of the DIKW Management System (dikwms).**

## Context

`main` is the stable integration line. It carries the **architecture, governance, and backend foundation** of the DIKW hierarchy platform (Data → Information → Knowledge → Wisdom, after Rowley 2007), plus the spec-driven-development process that governs every feature branch:

- **Docs governance** — `docs/specs/` (SPEC-NNN, with completed specs archived), `docs/adr/`, `docs/risks/`, `docs/ddd/`, `docs/brainstorm/`, `docs/openapi/`. Single intake queue `docs/specs/backlog.md`; specs are promoted from backlog, never created on demand.
- **Backend workspace** — Rust (edition 2024, nightly toolchain required): `data-graph` (core domain model / wire contract), `terminusdb-repository`, `schema-registry`, `stream-api` (SSE), `api` (axum REST, :8080), `git-importer`.
- **TerminusDB integration** — vendored fork `third-party/terminusdb-client-rs` (submodule), `compose.yml` for real per-process TerminusDB servers (:6363 / :6364), integration tests use `TerminusDBServer::test_instance()` — never mocks for persistence.

## What is NOT here

Feature work lives on dedicated branches (e.g. `speed-run-1/git-domain` — SPEC-027 git explorer). This branch tracks implementation **records** (spec deltas, ADRs, risk resolutions) but not the feature code itself.

## Development

See `AGENTS.md` for the full ruleset: schemaless-by-default (warn-not-fail), `.wip/` is off-limits scratch space, spec intake gate, and the Rust workflow (`env RUSTUP_TOOLCHAIN=nightly cargo ...` — the fork requires nightly).
