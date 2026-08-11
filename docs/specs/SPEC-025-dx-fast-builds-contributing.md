# SPEC-025 Feature: Developer Experience — Fast Local Builds/Tests + CONTRIBUTING.md

<!-- status: Implemented -->
<!-- progress: fork PR-12 = 8 commits, 6 files, opened 2026-08-10; backend adoption committed (5c4d35a/f395461/415410d); awaiting upstream merge -->
<!-- approved: 2026-08-10 by orchestrator (retrospective — implementation preceded approval; see Delta Record) -->

## Verification Record (2026-08-10)

- AC-1 ✅ — `cargo config get` resolves the example; `cargo check -p data-graph` passes with the copy applied (dev profile from config)
- AC-2 ✅ — PR #12 diff: no `-Z` flags; linker overrides exist only in the example; tracked `.cargo/config.toml` deleted
- AC-3 ✅ — PR #12 diff vs `main` = exactly `.cargo/config.toml` (del), `.cargo/config.toml.example`, `.gitignore`, `CONTRIBUTING.md`, `scripts/dx-benchmark-{no-dep,hyperfine}.sh`; `Cargo.toml` byte-identical
- AC-4 ✅ — CONTRIBUTING: Linux/macOS/Windows deps table + opt-in speed-ups section (sccache, mold/lld, per-developer config, profile overrides, CARGO_TARGET_DIR, RUST_TEST_THREADS)
- AC-5 ✅ — CONTRIBUTING testing section documents `TerminusDBServer::test_instance()` + `RUST_TEST_THREADS=1` escape hatch
- AC-6 ⚠️ PENDING — no CI checks reported on `pr/dx` (fork PR on upstream CI hasn't triggered); re-verify post-merge
- AC-7 ✅ — linker-less PATH run: `WARNING: lld not found…` and the run continues (suggestion printed)

**Verdict**: 6/7 PASS, 1 PENDING (upstream CI) → Implemented. **Not archived**: PR #12 awaiting upstream review/merge; re-verify AC-6 and archive on landing.

## Delta Record (2026-08-10)

- **MODIFIED AC-2 (review-phase catch, 2026-08-10)**: AC-2 claimed the PR diff contains "no linker overrides" — false since the `.example` ships per-OS linker blocks by design. Reworded: no `-Z` flags and no *active* config in the PR; the linker overrides are opt-in via the example copy (without the copy, rustc defaults apply). See AC-2 below.
- **ADDED Requirement "CI config generation (ci-gen)"** (user requirement 2026-08-10): new `scripts/ci-gen-cargo-config.sh` — a **generic, auto-discovering** generator (no runner/vendor knowledge; container-aware via cgroup v2/v1) used by `tests.yml` to generate the gitignored `.cargo/config.toml` (optional `[build]`/`[env]`/`[target]` sections) from the machine's cores/RAM/OS/linkers. This narrows the SPEC-024 axis split: the single minimal tests.yml step is in scope here; the multi-version matrix remains SPEC-024. See the requirement + AC-8 below.

## Delta Record (2026-08-10)

- **MODIFIED Requirement "Fast dev-profile defaults"**: profiles move from the workspace `Cargo.toml` into **`.cargo/config.toml.example`** — `Cargo.toml` stays byte-identical to upstream. Rationale (user constraint, 2026-08-10): `debug = 1` strips local-variable info and `[profile.dev.package."*"] opt-level = 2` changes test-build/runtime behavior for every contributor (incl. CI) — both are per-developer preferences, so they are opt-in, like the linker blocks.
- **MODIFIED AC-1/AC-3/AC-6, Technical Design, Test Plan, R-30 mitigation**: see inline deltas. PR-12 branch history re-grouped to 3 commits (example+gitignore / CONTRIBUTING / bench script).
- **ADDED Requirement "Hyperfine sweet-spot benchmark"** (user requirement 2026-08-10): `scripts/bench-dx.sh` renamed to `scripts/dx-benchmark-no-dep.sh`; new `scripts/dx-benchmark-hyperfine.sh` suggests `.cargo/config.toml` sweet-spot values from detected hardware (RAM/cores/OS) and validates with hyperfine; missing linking deps (mold/lld) emit WARNINGs only, never block. See the requirement + delta sections below.
- **ADDED Scope note: platform `backend/` adoption** (user requirement 2026-08-10): the same DX surface is applied to the platform's own `backend/` Rust workspace — `backend/CONTRIBUTING.md`, `backend/scripts/dx-benchmark-no-dep.sh` + `backend/scripts/dx-benchmark-hyperfine.sh` (adapted targets: `-p api`, `crates/api/src/lib.rs`), `backend/.cargo/config.toml.example` (gitignored local copy; previously tracked active `config.toml` untracked). `backend/Cargo.toml`'s `[profile.dev]`/`[profile.dev.package."*"]` are **removed** — profiles live only in the example (parity with the fork). Backend CI does not exist yet; the upstream-CI note applies when it lands (SPEC-024 axis).
- **CORRECTION (2026-08-10)**: `RUSTUP_TOOLCHAIN` is **not** added to the example's `[env]` — **cargo rejects toolchain variables in the `[env]` config table** ("setting the RUSTUP_TOOLCHAIN environment variable is not supported in the `[env]` configuration table", verified locally). Nightly forcing stays environment-only: `RUSTUP_TOOLCHAIN=nightly cargo ...` (CONTRIBUTING troubleshooting + benchmark scripts export it). The example carries a comment documenting the limitation; CONTRIBUTING notes the same.

## Overview

PR-4 to `ParapluOU/terminusdb-rs`: improve contributor DX by shipping the **non-imposing** parts of the fork's fast-build work (dev-profile defaults) plus a **CONTRIBUTING.md** with per-OS guidelines (Linux, macOS, Windows) for a fast build/test loop. Explicitly **out of scope**: `.cargo/config.toml`, `.mise.toml`, and any forced toolchain/linker/dependency installs — guidelines only, nothing imposed.

**Repo wiring (as SPEC-008 precedent):** this spec governs **fork-side work** — implementation happens in `third-party/terminusdb-client-rs` (submodule → github.com/gustavorps/terminusdb-client-rs), specifically **`third-party/terminusdb-client-rs/CONTRIBUTING.md`**, its `.cargo/config.toml.example` (dev profiles + per-OS baseline) and `scripts/bench-dx.sh` — `Cargo.toml` itself is NOT touched (byte-identical to upstream). The spec document lives in the platform's `docs/specs/` as the governing artifact; the PR targets upstream `ParapluOU/terminusdb-rs` from the fork.

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

The PR SHALL ship **`.cargo/config.toml.example`** (committed baseline: dev-profile speed-ups — `opt-level = 0`, `debug = 1`, `codegen-units = 256`, deps at `opt-level = 2` — plus per-OS linker blocks — mold on Linux, lld on macOS/Windows — `split-debuginfo`, `jobs`, `RUST_TEST_*`) and SHALL add `/.cargo/config.toml` to `.gitignore` so each developer copies the example to their own gitignored local file. **No active `.cargo/config.toml` is committed** — the previously tracked one (`RUST_TEST_THREADS = "1"`) is removed from the tree; without the copy, the workspace builds with defaults (non-imposing). `RUSTUP_TOOLCHAIN` is deliberately **not** in the `[env]` block — cargo refuses toolchain variables there; nightly forcing is environment-only (see Overview).

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

### Requirement: CI config generation (ci-gen) *(ADDED 2026-08-10)*

The PR SHALL ship **`scripts/ci-gen-cargo-config.sh`** — a **generic generator** that auto-discovers the machine's configuration at runtime (logical cores, RAM, OS, installed linkers) with **no knowledge of the runner or CI vendor** (GitHub-hosted, act containers, self-hosted), and SHALL be invoked by `tests.yml` before the first cargo command to write the gitignored `.cargo/config.toml`. Generated sections are optional and emitted only when discoverable/requested: `[build] jobs = min(cores, RAM_GB / 2.5)` (RAM-bound rustc), `[env] RUST_TEST_THREADS = cores` (override per invocation; `1` for shared-resource tests), per-OS `[target]` linker/rustflags **only if the linker is present** (mold/lld on Linux, lld on macOS, lld-link on Windows). On Linux the script SHALL be container-aware: cgroup v2 (`cpu.max`, `memory.max`) / v1 limits take precedence over host-reported `/proc` values (containers report host hardware otherwise). Flags: `--build|--env|--target` section selection (default all), `--jobs=`/`--threads=`/`--cores=`/`--ram=`/`--os=` overrides, `--out=`, `--dry-run`, `--force` (refuses to overwrite an existing config otherwise).

#### Scenario: Any runner, no configuration
- **GIVEN** a CI machine (or act container) with unknown topology
- **WHEN** `scripts/ci-gen-cargo-config.sh` runs
- **THEN** cores/RAM/OS/linkers are discovered at runtime and the generated `.cargo/config.toml` matches that machine — no runner name, image, or vendor assumption anywhere

#### Scenario: Containerized runner respects limits
- **GIVEN** a containerized runner (act/docker) whose cgroup allows fewer cores/RAM than the host
- **WHEN** the script runs on Linux
- **THEN** the cgroup limits (not `/proc/meminfo`/`nproc`) drive the generated sections

#### Scenario: Missing linker → no target section
- **GIVEN** a machine without mold/lld
- **WHEN** the script runs
- **THEN** no `[target]` section is emitted and the run succeeds (build/env still generated)

### Requirement: Hyperfine sweet-spot benchmark *(ADDED 2026-08-10)*

The PR SHALL ship **`scripts/dx-benchmark-hyperfine.sh`** (hyperfine-based; `scripts/bench-dx.sh` renamed to `scripts/dx-benchmark-no-dep.sh`, behavior unchanged) that: (a) detects logical/physical cores and RAM per OS (macOS `sysctl`, Linux `nproc`/`lscpu`/`/proc/meminfo`, Windows best-effort) and suggests `.cargo/config.toml` starting values — `CARGO_BUILD_JOBS = min(logical cores, floor(RAM_GB / 2.5))` (rustc is RAM-bound: 1.5–4 GB per parallel compiler during LLVM codegen) and `RUST_TEST_THREADS` guidance (physical cores for CPU-bound unit tests; 1.5–2× logical for I/O-bound integration tests; `1` for shared-resource tests); (b) validates the candidates with hyperfine (jobs scaling — incremental by default, clean builds with `--full`; test-thread scaling — unit by default, integration with `--full`); (c) **if the linking dependencies (mold/lld) are not installed, emits a WARNING but does NOT block the benchmark**; (d) `--no-bench` prints the suggestion without hyperfine.

#### Scenario: Missing linker warns but doesn't block
- **GIVEN** a machine without mold (Linux) / lld (macOS) / lld-link (Windows) installed
- **WHEN** `scripts/dx-benchmark-hyperfine.sh` runs
- **THEN** a WARNING names the missing tool and the benchmark proceeds on the system linker

#### Scenario: Suggestion reflects hardware
- **GIVEN** 8 logical cores and 16 GB RAM
- **WHEN** the script runs (or `--no-bench`)
- **THEN** the suggested `CARGO_BUILD_JOBS` is `min(8, floor(16/2.5)) = 6` and the thread guidance shows physical-core / 1.5–2× / 1 options

## Acceptance Criteria

- AC-1: Given a developer who copied `.cargo/config.toml.example` to a local gitignored `.cargo/config.toml`, when `cargo build -p terminusdb-client` runs on nightly without extra flags, then it succeeds with the fast dev profile. *(MODIFIED: profiles are in the example, not `Cargo.toml`; without the copy, stock rustc defaults apply.)*
- AC-2: Given the PR diff, when inspected, then it contains no `-Z` flags and no **active** config: linker overrides appear only inside `.cargo/config.toml.example` (opt-in — without the copy, rustc defaults apply and no linker override is active). Nightly remains required workspace-wide — `terminusdb-schema` uses `#![feature(specialization)]`; verified 2026-08-10 that stable fails there, pre-existing, not caused by this PR. *(MODIFIED 2026-08-10: previously claimed no linker overrides at all — contradicted the example.)*
- AC-3: Given the PR branch, when its diff vs upstream `main` is inspected, then only `.cargo/config.toml.example`, `.gitignore`, `CONTRIBUTING.md`, `scripts/bench-dx.sh` appear, and **`Cargo.toml` is byte-identical to upstream** (no active `.cargo/config.toml`, no `.mise.toml`); `/.cargo/config.toml` is gitignored.
- AC-4: Given CONTRIBUTING.md, when read, then it has Linux/macOS/Windows dependency sections and an optional speed-ups section (sccache, lld/mold, share-generics, test threads).
- AC-5: Given CONTRIBUTING.md, when the testing section is read, then the embedded-server pattern and `RUST_TEST_THREADS=1` escape hatch are documented.
- AC-6: Given the PR, when CI runs, then existing tests remain green (profiles are opt-in; CI does not copy the example). *(MODIFIED: previously "profiles are compile-time-only".)*
- AC-7: Given the PR, when `scripts/dx-benchmark-hyperfine.sh --no-bench` runs on a machine without mold/lld, then a WARNING names the missing linker and the suggestion still prints (no block). *(ADDED 2026-08-10.)*
- AC-8: Given any machine, when `scripts/ci-gen-cargo-config.sh` runs, then it auto-discovers cores/RAM/OS/linkers (no runner/vendor knowledge), emits only discoverable sections, and `tests.yml` invokes it before the first cargo command. *(ADDED 2026-08-10.)*

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
4. `scripts/dx-benchmark-no-dep.sh` (new, executable — renamed from `bench-dx.sh`) — rustc-native benchmark (full/incr-unchanged/incr-patched × wall/timings/self-profile). *(MODIFIED 2026-08-10: renamed; hyperfine script added as step 5.)*
5. `scripts/dx-benchmark-hyperfine.sh` (new, executable) — hardware-guided sweet spot + hyperfine validation; linker-missing → WARNING only. *(ADDED 2026-08-10.)*
6. `scripts/ci-gen-cargo-config.sh` (new, executable) — generic auto-discovering config generator for CI (build/env/target sections; container-aware). *(ADDED 2026-08-10.)*
7. `.github/workflows/tests.yml` — one step invoking the generator before the first cargo command. *(ADDED 2026-08-10; the broader CI matrix stays SPEC-024.)*

`Cargo.toml` is **untouched** (byte-identical to upstream). *(MODIFIED 2026-08-10: previously step 1 appended the profiles to `Cargo.toml`.)*

### Excluded content (explicit non-goals)
- Active `.cargo/config.toml` — never committed; the gitignored local copy comes from `.cargo/config.toml.example`, and CI generates its own via `ci-gen-cargo-config.sh` (the upstream-tracked `RUST_TEST_THREADS = "1"` config is removed by this PR)
- `[profile.dev]` / `[profile.dev.package."*"]` in `Cargo.toml` — per-developer preferences (debugger UX; test-build behavior for everyone incl. CI), so they live in the example only
- `.mise.toml` → mentioned as optional tooling
- `580b840` rustfmt sweep — unrelated
- CI/workflow changes beyond the single `ci-gen` step — separate axis (SPEC-024); NOTE: upstream CI loses the tracked single-thread config — the generated config sets threads from discovered cores; if contention surfaces, invoke `ci-gen-cargo-config.sh --threads 1` in `tests.yml` or move to SPEC-024

### Trade-off note for CONTRIBUTING
`[profile.dev.package."*"] opt-level = 2` trades slightly longer first dependency build for much faster test runtime (the embedded server runs optimized). Documented so contributors aren't surprised.

## Test Plan

- [ ] `cargo build -p terminusdb-client` (nightly, no flags, after copying the example) — succeeds with the fast dev profile (AC-1)
- [ ] Fresh-clone build without the example copy — stock rustc defaults, succeeds (AC-1 delta)
- [ ] Stable-toolchain build of the client crate (AC-2) — verify no `-Z` leakage
- [ ] Diff inspection vs upstream `main` — only `.cargo/config.toml.example`, `.gitignore`, `CONTRIBUTING.md`, `scripts/dx-benchmark-no-dep.sh`, `scripts/dx-benchmark-hyperfine.sh`; `Cargo.toml` byte-identical (AC-3)
- [ ] CONTRIBUTING.md content review against AC-4/AC-5 checklist
- [ ] `cargo test -p terminusdb-client --lib` — existing tests green (AC-6)
- [ ] `scripts/dx-benchmark-hyperfine.sh --no-bench` on a linker-less PATH — WARNING + suggestion, no block (AC-7)
- [ ] `scripts/ci-gen-cargo-config.sh --dry-run` on macOS + Linux-sim (`--os=linux`) — sections reflect discovery; linker-less run emits no `[target]` (AC-8)
- [ ] tests.yml diff review — ci-gen step before first cargo invocation (AC-8)

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
