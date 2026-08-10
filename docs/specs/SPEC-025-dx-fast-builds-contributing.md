# SPEC-025 Feature: Developer Experience — Fast Local Builds/Tests + CONTRIBUTING.md

<!-- status: Draft -->
<!-- progress: raw material = fork dev commit 13cb418 (excluded from PR-1..3 as dev-tooling) -->

## Overview

PR-4 to `ParapluOU/terminusdb-rs`: improve contributor DX by shipping the **non-imposing** parts of the fork's fast-build work (dev-profile defaults) plus a **CONTRIBUTING.md** with per-OS guidelines (Linux, macOS, Windows) for a fast build/test loop. Explicitly **out of scope**: `.cargo/config.toml`, `.mise.toml`, and any forced toolchain/linker/dependency installs — guidelines only, nothing imposed.

**Repo wiring (as SPEC-008 precedent):** this spec governs **fork-side work** — implementation happens in `third_party/terminusdb-rs` (submodule → github.com/gustavorps/terminusdb-rs), specifically **`third_party/terminusdb-rs/CONTRIBUTING.md`** and its `Cargo.toml`. The spec document lives in the platform's `docs/specs/` as the governing artifact; the PR targets upstream `ParapluOU/terminusdb-rs` from the fork.

## Motivation

The upstream workspace has only `[profile.release]` (lto, codegen-units=1 — slow release builds) and no dev-profile tweaks, no CONTRIBUTING.md, and no per-OS setup guidance. First-time contributors face: nightly-only `rust-toolchain.toml`, heavy system deps (clang/libclang, protoc, libgmp, openssl, SWI-Prolog via the embedded server), and multi-minute first builds with no guidance on caching (sccache), faster linkers (lld/mold), or test-run patterns. The fork's `13cb418` solves this locally but in an imposing way (`.cargo/config.toml` forces lld + nightly `-Z share-generics`; `.mise.toml` forces mise). Source: user request 2026-08-10 (backlog SPEC-025).

## Requirements

### Requirement: Fast dev-profile defaults

The workspace SHALL ship compile-time-only dev-profile defaults that speed incremental builds without imposing toolchain/linker requirements: `codegen-units = 256`, `opt-level = 0`, `debug = 1` under `[profile.dev]`, and `opt-level = 2` under `[profile.dev.package."*"]` (optimized dependencies → much faster test runtime, notably the embedded server).

#### Scenario: Dev builds use fast defaults
- **GIVEN** the workspace Cargo.toml with the dev-profile defaults
- **WHEN** `cargo build -p terminusdb-client` runs on the default (nightly) toolchain without extra flags
- **THEN** the build succeeds and uses the fast profile

#### Scenario: Stable toolchain unaffected
- **GIVEN** the same Cargo.toml
- **WHEN** a stable toolchain builds the client crate
- **THEN** the build succeeds — no `-Z` flags or linker overrides are referenced anywhere

### Requirement: CONTRIBUTING.md with per-OS guidelines

The repository SHALL provide CONTRIBUTING.md covering: system dependencies per OS (Linux: clang/libclang, protoc, libgmp, openssl; macOS: brew equivalents; Windows: client/schema crates only — the embedded-server tests require SWI-Prolog, Linux/macOS-first or WSL), first-build expectations, and **optional** speed-ups (documented as opt-in, via env vars or toolchain flags, never config files): sccache (`CARGO_BUILD_RUSTC_WRAPPER`), lld/mold linkers (`CARGO_TARGET_*_LINKER`/`RUSTFLAGS`), nightly `-Z share-generics`, and `RUST_TEST_THREADS=1` for test-serialization when servers contend.

#### Scenario: Optional speed-ups documented, not imposed
- **GIVEN** CONTRIBUTING.md
- **WHEN** a contributor reads the build-speed section
- **THEN** every speed-up is opt-in and the default path needs no extra tooling

#### Scenario: Windows guidance present
- **GIVEN** CONTRIBUTING.md
- **WHEN** a Windows contributor reads the platform section
- **THEN** they learn which crates build/test on Windows natively and the WSL path for embedded-server tests

### Requirement: No imposition of tooling

The PR SHALL NOT add `.cargo/config.toml`, `.mise.toml`, or any file that forces a linker, toolchain, or dependency install on contributors.

#### Scenario: PR diff contains no imposed config
- **GIVEN** the PR branch
- **WHEN** `git diff upstream/main...branch --stat` is inspected
- **THEN** no `.cargo/config.toml` or `.mise.toml` appears and no file sets `rustflags` or `rustc-wrapper`

### Requirement: Test-run guidance

CONTRIBUTING.md SHALL document the test pattern: integration tests spawn real per-process embedded servers (`TerminusDBServer::test_instance()`, unique ports, parallel-safe) — no docker required; and the contention escape hatch `RUST_TEST_THREADS=1`.

#### Scenario: Test pattern documented
- **GIVEN** CONTRIBUTING.md
- **WHEN** the testing section is read
- **THEN** it explains the embedded-server pattern and the single-threaded escape hatch

## Acceptance Criteria

- AC-1: Given the workspace, when `cargo build -p terminusdb-client` runs on nightly without extra flags, then it succeeds with the fast dev-profile defaults.
- AC-2: Given the workspace, when a stable toolchain builds the client crate, then it succeeds (no `-Z`/linker references in the diff).
- AC-3: Given the PR branch, when its diff vs upstream `main` is inspected, then no `.cargo/config.toml`, `.mise.toml`, or forced-config file appears.
- AC-4: Given CONTRIBUTING.md, when read, then it has Linux/macOS/Windows dependency sections and an optional speed-ups section (sccache, lld/mold, share-generics, test threads).
- AC-5: Given CONTRIBUTING.md, when the testing section is read, then the embedded-server pattern and `RUST_TEST_THREADS=1` escape hatch are documented.
- AC-6: Given the PR, when CI runs, then existing tests remain green (profiles are compile-time-only).

## Technical Design

### Diff shape (upstream `main`-based branch `pr/dx`)

1. `Cargo.toml` — append the dev profiles (port of `13cb418`'s Cargo.toml hunk only):
   ```toml
   [profile.dev]
   opt-level = 0
   debug = 1
   codegen-units = 256

   [profile.dev.package."*"]
   opt-level = 2
   ```
2. `CONTRIBUTING.md` (new) — sections: prerequisites per OS, first build, daily loop (check → test crate → targeted test), optional speed-ups (env-var based), testing patterns, troubleshooting.

### Excluded content (explicit non-goals)
- `.cargo/config.toml` contents from `13cb418` (lld, `-Z share-generics`, `RUST_TEST_THREADS=1`, sccache wrapper) → CONTRIBUTING.md guidance only
- `.mise.toml` → mentioned as optional tooling
- `580b840` rustfmt sweep — unrelated
- CI/workflow changes — separate axis (SPEC-024)

### Trade-off note for CONTRIBUTING
`[profile.dev.package."*"] opt-level = 2` trades slightly longer first dependency build for much faster test runtime (the embedded server runs optimized). Documented so contributors aren't surprised.

## Test Plan

- [ ] `cargo build -p terminusdb-client` (nightly, no flags) — succeeds with new profiles (AC-1)
- [ ] Stable-toolchain build of the client crate (AC-2) — verify no `-Z` leakage
- [ ] Diff inspection vs upstream `main` (AC-3)
- [ ] CONTRIBUTING.md content review against AC-4/AC-5 checklist
- [ ] `cargo test -p terminusdb-client --lib` — existing tests green (AC-6)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-30 | Reviewers reject profile defaults as opinionated | Medium | Compile-time-only justification; matches common community practice; offer to drop if contested |
| R-31 | CONTRIBUTING.md drifts stale after future dep changes | Low | Keep it close to `tests.yml`'s documented deps; refresh on CI changes (SPEC-024) |
| R-32 | `opt-level = 2` on deps surprises contributors (slower first dep build) | Low | Trade-off documented in CONTRIBUTING (Technical Design) |
| R-33 | Windows embedded-server guidance can't be validated in CI (no Windows runner upstream) | Low | Label the WSL path as community-verified; keep native guidance to client/schema crates |

## Relationship to Other Specs

- SPEC-023: same upstream target, independent concept — PR-4 is a separate branch/PR, no merge-order dependency
- SPEC-024: CI speed/matrix is the CI axis; SPEC-025 is the local-DX axis — explicit split
- Backlog SPEC-025 (planned): this spec's intake entry
