# SPEC-025 Feature: Developer Experience — Fast Local Builds/Tests + CONTRIBUTING.md

<!-- status: Draft -->
<!-- progress: raw material = fork dev commit 13cb418 (excluded from PR-1..3 as dev-tooling) -->

## Delta Record (2026-08-10)

- **MODIFIED Requirement "Fast dev-profile defaults"**: profiles move from the workspace `Cargo.toml` into **`.cargo/config.toml.example`** — `Cargo.toml` stays byte-identical to upstream. Rationale (user constraint, 2026-08-10): `debug = 1` strips local-variable info and `[profile.dev.package."*"] opt-level = 2` changes test-build/runtime behavior for every contributor (incl. CI) — both are per-developer preferences, so they are opt-in, like the linker blocks.
- **MODIFIED AC-1/AC-3/AC-6, Technical Design, Test Plan, R-30 mitigation**: see inline deltas. PR-12 branch history re-grouped to 3 commits (example+gitignore / CONTRIBUTING / bench script).

## Overview

PR-4 to `ParapluOU/terminusdb-rs`: improve contributor DX by shipping the **non-imposing** parts of the fork's fast-build work (dev-profile defaults) plus a **CONTRIBUTING.md** with per-OS guidelines (Linux, macOS, Windows) for a fast build/test loop. Explicitly **out of scope**: `.cargo/config.toml`, `.mise.toml`, and any forced toolchain/linker/dependency installs — guidelines only, nothing imposed.

**Repo wiring (as SPEC-008 precedent):** this spec governs **fork-side work** — implementation happens in `third_party/terminusdb-rs` (submodule → github.com/gustavorps/terminusdb-rs), specifically **`third_party/terminusdb-rs/CONTRIBUTING.md`**, its `.cargo/config.toml.example` (dev profiles + per-OS baseline) and `scripts/bench-dx.sh` — `Cargo.toml` itself is NOT touched (byte-identical to upstream). The spec document lives in the platform's `docs/specs/` as the governing artifact; the PR targets upstream `ParapluOU/terminusdb-rs` from the fork.

## Motivation

The upstream workspace has only `[profile.release]` (lto, codegen-units=1 — slow release builds) and no dev-profile tweaks, no CONTRIBUTING.md, and no per-OS setup guidance. First-time contributors face: nightly-only `rust-toolchain.toml`, heavy system deps (clang/libclang, protoc, libgmp, openssl, SWI-Prolog via the embedded server), and multi-minute first builds with no guidance on caching (sccache), faster linkers (lld/mold), or test-run patterns. The fork's `13cb418` solves this locally but in an imposing way (`.cargo/config.toml` forces lld + nightly `-Z share-generics`; `.mise.toml` forces mise). Source: user request 2026-08-10 (backlog SPEC-025).

## Requirements

### Requirement: Fast dev-profile defaults

The PR SHALL ship the dev-profile defaults (`codegen-units = 256`, `opt-level = 0`, `debug = 1` under `[profile.dev]`, and `opt-level = 2` under `[profile.dev.package."*"]`) **inside `.cargo/config.toml.example`** — NOT in the workspace `Cargo.toml`, which stays byte-identical to upstream. The profiles are compile-time-only and impose no toolchain/linker requirements, but they are per-developer preferences (debugger UX; test-build behavior for everyone incl. CI), so they are opt-in via the example copy, same as the linker blocks. *(MODIFIED 2026-08-10: previously required appending the profiles to `Cargo.toml`.)*

#### Scenario: Dev builds use fast defaults (after opt-in)
- **GIVEN** a developer who copied `.cargo/config.toml.example` to their local `.cargo/config.toml`
- **WHEN** `cargo build -p terminusdb-client` runs on the default (nightly) toolchain without extra flags
- **THEN** the build succeeds and uses the fast dev profile

#### Scenario: No-copy builds use stock defaults
- **GIVEN** a fresh clone WITHOUT a local `.cargo/config.toml`
- **WHEN** `cargo build -p terminusdb-client` runs
- **THEN** the build succeeds with rustc's default dev profile — nothing imposed

#### Scenario: Stable toolchain unaffected
- **GIVEN** the example config or no config
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

### Requirement: Per-developer cargo config (gitignored, `.example` committed)

The PR SHALL ship **`.cargo/config.toml.example`** (committed baseline: dev-profile speed-ups — `opt-level = 0`, `debug = 1`, `codegen-units = 256`, deps at `opt-level = 2` — plus per-OS linker blocks — mold on Linux, lld on macOS/Windows — `split-debuginfo`, `jobs`, `RUST_TEST_*`) and SHALL add `/.cargo/config.toml` to `.gitignore` so each developer copies the example to their own gitignored local file. **No active `.cargo/config.toml` is committed** — the previously tracked one (`RUST_TEST_THREADS = "1"`) is removed from the tree; without the copy, the workspace builds with defaults (non-imposing).

#### Scenario: Local config is per-developer
- **GIVEN** the repo with `.cargo/config.toml.example` and the gitignore entry
- **WHEN** a developer runs `cp .cargo/config.toml.example .cargo/config.toml`
- **THEN** their local build uses the optimizations, the file stays untracked, and other developers' builds are unaffected

### Requirement: Test-run guidance

CONTRIBUTING.md SHALL document the test pattern: integration tests spawn real per-process embedded servers (`TerminusDBServer::test_instance()`, unique ports, parallel-safe) — no docker required; and the contention escape hatch `RUST_TEST_THREADS=1`.

#### Scenario: Test pattern documented
- **GIVEN** CONTRIBUTING.md
- **WHEN** the testing section is read
- **THEN** it explains the embedded-server pattern and the single-threaded escape hatch

## Acceptance Criteria

- AC-1: Given a developer who copied `.cargo/config.toml.example` to a local gitignored `.cargo/config.toml`, when `cargo build -p terminusdb-client` runs on nightly without extra flags, then it succeeds with the fast dev profile. *(MODIFIED: profiles are in the example, not `Cargo.toml`; without the copy, stock rustc defaults apply.)*
- AC-2: Given the PR diff, when inspected, then it contains no `-Z` flags or linker overrides (nightly remains required workspace-wide — `terminusdb-schema` uses `#![feature(specialization)]`; verified 2026-08-10 that stable fails there, pre-existing, not caused by this PR).
- AC-3: Given the PR branch, when its diff vs upstream `main` is inspected, then only `.cargo/config.toml.example`, `.gitignore`, `CONTRIBUTING.md`, `scripts/bench-dx.sh` appear, and **`Cargo.toml` is byte-identical to upstream** (no active `.cargo/config.toml`, no `.mise.toml`); `/.cargo/config.toml` is gitignored.
- AC-4: Given CONTRIBUTING.md, when read, then it has Linux/macOS/Windows dependency sections and an optional speed-ups section (sccache, lld/mold, share-generics, test threads).
- AC-5: Given CONTRIBUTING.md, when the testing section is read, then the embedded-server pattern and `RUST_TEST_THREADS=1` escape hatch are documented.
- AC-6: Given the PR, when CI runs, then existing tests remain green (profiles are opt-in; CI does not copy the example). *(MODIFIED: previously "profiles are compile-time-only".)*

## Technical Design

### Diff shape (upstream `main`-based branch `pr/dx`)

1. `.cargo/config.toml.example` (new) — dev profiles (port of `13cb418`'s Cargo.toml hunk only) + per-OS linker/env baseline:
   ```toml
   [profile.dev]
   opt-level = 0
   debug = 1
   codegen-units = 256

   [profile.dev.package."*"]
   opt-level = 2
   ```
2. `.gitignore` — add `/.cargo/config.toml`; remove the tracked `.cargo/config.toml` (`RUST_TEST_THREADS = "1"`) from the tree — the single-thread escape hatch stays documented in CONTRIBUTING (`RUST_TEST_THREADS=1`, per-invocation).
3. `CONTRIBUTING.md` (new) — sections: prerequisites per OS, first build, daily loop (check → test crate → targeted test), optional speed-ups (env-var based), per-developer cargo config workflow, testing patterns, troubleshooting, benchmarking.
4. `scripts/bench-dx.sh` (new, executable) — rustc-native benchmark (full/incr-unchanged/incr-patched × wall/timings/self-profile).

`Cargo.toml` is **untouched** (byte-identical to upstream). *(MODIFIED 2026-08-10: previously step 1 appended the profiles to `Cargo.toml`.)*

### Excluded content (explicit non-goals)
- Active `.cargo/config.toml` — never committed; the gitignored local copy comes from `.cargo/config.toml.example` (the upstream-tracked `RUST_TEST_THREADS = "1"` config is removed by this PR)
- `[profile.dev]` / `[profile.dev.package."*"]` in `Cargo.toml` — per-developer preferences (debugger UX; test-build behavior for everyone incl. CI), so they live in the example only
- `.mise.toml` → mentioned as optional tooling
- `580b840` rustfmt sweep — unrelated
- CI/workflow changes — separate axis (SPEC-024); NOTE: upstream CI loses the tracked single-thread config — if contention surfaces, add `RUST_TEST_THREADS: "1"` to `tests.yml` under SPEC-024

### Trade-off note for CONTRIBUTING
`[profile.dev.package."*"] opt-level = 2` trades slightly longer first dependency build for much faster test runtime (the embedded server runs optimized). Documented so contributors aren't surprised.

## Test Plan

- [ ] `cargo build -p terminusdb-client` (nightly, no flags, after copying the example) — succeeds with the fast dev profile (AC-1)
- [ ] Fresh-clone build without the example copy — stock rustc defaults, succeeds (AC-1 delta)
- [ ] Stable-toolchain build of the client crate (AC-2) — verify no `-Z` leakage
- [ ] Diff inspection vs upstream `main` — only `.cargo/config.toml.example`, `.gitignore`, `CONTRIBUTING.md`, `scripts/bench-dx.sh`; `Cargo.toml` byte-identical (AC-3)
- [ ] CONTRIBUTING.md content review against AC-4/AC-5 checklist
- [ ] `cargo test -p terminusdb-client --lib` — existing tests green (AC-6)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-30 | Reviewers reject profile defaults as opinionated | Medium | Profiles are opt-in via the example (per-developer preferences), so the objection surface is gone; matches common community practice |
| R-31 | CONTRIBUTING.md drifts stale after future dep changes | Low | Keep it close to `tests.yml`'s documented deps; refresh on CI changes (SPEC-024) |
| R-32 | `opt-level = 2` on deps surprises contributors (slower first dep build) | Low | Trade-off documented in CONTRIBUTING (Technical Design) |
| R-33 | Windows embedded-server guidance can't be validated in CI (no Windows runner upstream) | Low | Label the WSL path as community-verified; keep native guidance to client/schema crates |

## Relationship to Other Specs

- SPEC-023: same upstream target, independent concept — PR-4 is a separate branch/PR, no merge-order dependency
- SPEC-024: CI speed/matrix is the CI axis; SPEC-025 is the local-DX axis — explicit split
- Backlog SPEC-025 (planned): this spec's intake entry
