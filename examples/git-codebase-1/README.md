# Git-Codebase-1 — git-domain speed-run reproduction (SPEC-027)

End-to-end reproduction for the first use case: **12 months of
`terminusdb/terminusdb` commit history** imported into the dikwms platform.

Deterministic by design: the import window uses a **fixed cutoff**
(`--since 2025-08-11`) — same inputs, same outputs, every run.

## Prerequisites

- TerminusDB (official image, v12.0.7) in docker: `docker compose up -d terminusdb` (repo root)
- The **data-graph API server**: `cargo run -p api` (backend/; `TERMINUSDB_URL` defaults to the docker server) — architecture: `git-import → data-graph API → terminusdb`
- Nightly Rust toolchain (`RUSTUP_TOOLCHAIN=nightly`)

## Steps

```bash
./scripts/01-clone.sh      # blobless clone of terminusdb/terminusdb into .cache/
./scripts/02-import.sh     # build + run the importer (first run: full import)
./scripts/02-import.sh     # re-run — must import 0 new commits (server-side idempotency)
./scripts/03-verify.sh     # assertions: state consistency + idempotent re-run
```

State file: `.import-state.json` (hexsha→Uuid index, skipped commits, counts).
Crash-safe: atomic writes; `--resume` rebuilds the index from the DB (DB wins).

## Layout

```
examples/git-codebase-1/
├── README.md
├── scripts/
│   ├── 01-clone.sh
│   ├── 02-import.sh
│   └── 03-verify.sh
└── .import-state.json     (generated; gitignored)
```
