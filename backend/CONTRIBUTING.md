# Contributing to dikwms backend

Thanks for contributing! The backend is a Rust workspace (API server, stream
API, knowledge domain, schema registry, TerminusDB repository adapter) that
depends on the `terminusdb-rs` fork in `third-party/terminusdb-client-rs` — including
the embedded-server test harness (`terminusdb-bin`). This guide keeps your
first build fast and your daily loop pleasant — everything here is **optional
guidance**, not imposed configuration.

## Prerequisites

- **Rust (nightly)** — the workspace requires nightly because the
  `terminusdb-rs` fork dependency uses nightly features (`specialization`).
  Install with `rustup toolchain install nightly` (or use `mise` if you
  prefer).
  > **Troubleshooting:** if plain `cargo` fails with `#![feature(...)]` errors
  > (e.g. `specialization`), an environment pin is overriding the toolchain —
  > e.g. mise exports `RUSTUP_TOOLCHAIN=1.97.1` (stable), which silently beats
  > the nightly requirement. Run builds with `RUSTUP_TOOLCHAIN=nightly cargo
  > ...` (the benchmark scripts do this automatically). Note: cargo refuses to
  > set `RUSTUP_TOOLCHAIN` from `.cargo/config.toml`'s `[env]` — it must be an
  > environment variable.
- **System dependencies** (for the embedded TerminusDB server used by
  integration tests — the `terminusdb-bin` dev-dependency builds it from
  source):

  | OS | Packages |
  |----|----------|
  | Linux (Debian/Ubuntu) | `build-essential make git curl unzip ca-certificates openssl libssl-dev libgmp-dev clang libclang-dev protobuf-compiler` |
  | macOS | `brew install llvm protobuf gmp openssl` (Xcode Command Line Tools required) |
  | Windows | The client/schema crates build and test natively. The embedded-server tests need SWI-Prolog + a POSIX toolchain — use **WSL2** for those. |

  The embedded server builds TerminusDB from source on first test run and
  provisions its own SWI-Prolog runtime — this is the slowest step in the
  repo, and it's cached afterward.

## First build (expect this)

```bash
RUSTUP_TOOLCHAIN=nightly cargo check -p api --tests   # fast — seconds
RUSTUP_TOOLCHAIN=nightly cargo build -p api           # compiles api + deps
RUSTUP_TOOLCHAIN=nightly cargo test -p api --tests    # first run: builds the
                                                      # embedded server (minutes)
```

The first `terminusdb-bin` build clones and compiles the pinned TerminusDB
server (with SWI-Prolog). It is cached under `target/` — subsequent builds are
fast. Prefer `cargo check` over `cargo build` for iteration.

## Daily loop

```bash
RUSTUP_TOOLCHAIN=nightly cargo check -p <crate> --tests   # type-check first
RUSTUP_TOOLCHAIN=nightly cargo test -p <crate> --lib      # unit tests (fast)
RUSTUP_TOOLCHAIN=nightly cargo test -p <crate> --tests    # integration tests
```

- **Target only the crate you changed** (`-p <crate>`) — never the whole
  workspace: the `terminusdb-bin` dev-dependency compiles TerminusDB from
  source and full builds are slow.
- `cargo check` before `cargo test` — catches compile errors in seconds
  without the full codegen/link pass.
- Heavy concurrent tests against the shared server may hit transaction
  contention — serialize with a global mutex (see `api/tests/spec_013_api.rs`)
  or run `RUST_TEST_THREADS=1`.

## Testing patterns

Integration tests use **real per-process TerminusDB servers** via
`TerminusDBServer::test_instance()` (v12.1, `--memory` mode) — never mocks for
persistence. Focused fixtures: `with_tmp_db`, `with_db_schema`, `with_db_seed`.
No `#[ignore]`, no external service, no Docker required.

If orphaned servers pile up after aborted test runs, clean them with
`pkill -f "serve --memory root"` before rebuilding.

## Optional speed-ups (all opt-in)

| Speed-up | Effect | How (your choice — no repo config changes) |
|----------|--------|---------------------------------------------|
| **sccache** | Caches compiled artifacts across workspaces/rebuilds | Install per OS — [macOS](https://www.google.com/search?q=how+install+sccache+on+macos), [Linux](https://www.google.com/search?q=how+install+sccache+on+linux), [Windows](https://www.google.com/search?q=how+install+sccache+on+windows) — then `CARGO_BUILD_RUSTC_WRAPPER=sccache cargo build` |
| **lld / mold linker** | Faster linking (Linux/macOS/Windows) | **Linux:** `sudo apt install mold` (or `lld`) · **macOS:** `brew install llvm` (lld included) · **Windows:** lld ships with recent LLVM / Visual Studio — then copy `.cargo/config.toml.example` to `.cargo/config.toml` (see below) |

### Per-developer cargo config

The repo ships **`.cargo/config.toml.example`** — a ready-made, per-OS-optimized baseline
(dev profile: `opt-level = 0`, `debug = 1`, `codegen-units = 256`, deps at
`opt-level = 2`; linkers: mold on Linux, lld on macOS/Windows; `split-debuginfo`;
`jobs`; `RUST_TEST_THREADS`/`RUST_TEST_NOCAPTURE`). Your **own** `.cargo/config.toml`
is **gitignored**, so each developer keeps their local settings without affecting
anyone else:

```bash
cp .cargo/config.toml.example .cargo/config.toml   # one-time setup
# edit to taste — jobs, threads, linker, debuginfo, sccache wrapper
```

Nothing imposes anything: without the copy, the workspace builds with defaults.
| **mise** | Pins the nightly toolchain per-workspace (no toolchain flips → no full rebuilds) | `mise install` after adding `.mise.toml` locally |
| **Profile overrides** | Tune dev builds without editing the repo | `CARGO_PROFILE_DEV_OPT_LEVEL`, `CARGO_PROFILE_DEV_CODEGEN_UNITS`, … |
| **Out-of-repo target dir** | Keeps the repo light and shares the cache across clones | `export CARGO_TARGET_DIR="$HOME/Library/Caches/Cargo/target"` (macOS) / `"$HOME/.cache/cargo/target"` (Linux) / `%LOCALAPPDATA%\Cargo\target` (Windows) |
| **Single-threaded tests** | Escape hatch under contention | `RUST_TEST_THREADS=1` (env var, per-invocation) |

The example's dev profile (`codegen-units = 256`, deps at `opt-level = 2`) trades
a slightly longer first dependency build for much faster test runtime, notably the
embedded server. Without the copy, rustc's defaults apply — nothing is imposed.

## Benchmarking build/test speed

Two scripts, same methodology (nightly forced automatically, `CARGO_TARGET_DIR`
aware):

```bash
scripts/dx-benchmark-no-dep.sh                # rustc-native, zero installs
scripts/dx-benchmark-no-dep.sh wall 5         # wall-clock only, 5 runs
scripts/dx-benchmark-hyperfine.sh             # hardware-guided sweet spot (RAM/cores)
scripts/dx-benchmark-hyperfine.sh --full      # + clean-build & integration-test scaling
scripts/dx-benchmark-hyperfine.sh --no-bench  # suggestion only, no hyperfine needed
```

**`dx-benchmark-hyperfine.sh`** detects logical/physical cores and RAM, suggests
a `.cargo/config.toml` starting point, then uses
[hyperfine](https://github.com/sharkdp/hyperfine) to find the real sweet spot:

- `CARGO_BUILD_JOBS = min(logical cores, RAM_GB / 2.5)` — rustc is RAM-bound
  (each parallel compiler can use 1.5–4 GB during LLVM codegen); low-RAM
  machines must cut jobs or they swap and lose 90% of compile speed
- `RUST_TEST_THREADS` — **physical cores** for CPU-bound unit tests,
  **1.5–2× logical cores** for I/O-bound integration tests (network/disk/async
  wait), **1** for shared-resource tests (the embedded-server harness, local
  ports, global state)

Missing linking dependencies (mold/lld) only emit **WARNING**s — the benchmark
still runs on the system linker.

Methodology (compare the same target on the base branch and the candidate
branch, same machine):

1. **Warm the cache first** — one build/test before measuring, so caches are
   populated and you measure incremental reality, not cold misses
2. **Use medians of ≥ 3 runs** — machine variance is real; single runs mislead
3. **Interpret correctly**:
   - *incremental build* and *unit-test runtime* are the **wins** the
     example's dev profile targets (256 codegen units; deps at `opt-level 2`
     make the embedded server and tests run much faster)
   - a **clean build is intentionally slower** on the optimized-deps profile —
     that is the documented trade-off for faster test runtime, not a regression

## Before opening a PR

- `cargo fmt` and `cargo clippy --all-targets` via nightly — fix warnings in
  the crates you touched; pre-existing fork warnings are not yours
- Add tests with the embedded-server pattern; integration tests must not
  require an external service
- Keep the diff focused — one logical change per PR; every change maps to a
  spec/AC (see the root `AGENTS.md` spec-governance rules)
