# SPEC-024 Feature: Multi-Version TerminusDB CI Matrix (docker + act)

<!-- status: Draft -->

## Overview

Add a docker-based integration-test matrix to the fork's GitHub workflows covering multiple `terminusdb/terminusdb-server` versions: **`latest` (blocking, priority)**, v12.x pins, and v11.x pins (report-only). Runnable locally via `act` (installed: act 0.2.89). Failures on non-latest versions are **never fixed** — the workflow generates a markdown report artifact to open an issue manually.

## Motivation

The existing integration surface (`tests.yml`) covers exactly one version — the embedded `v12.1-rc-paraplu.1` server built from source (90-min job). Docker tags (`latest`, v12.x, v11.x) are untested, so regressions against released server images go undetected. `latest` is the priority because it is what users pull; v11.x is a legacy surface the client deliberately dropped (v12-only per fork CLAUDE.md) — running it as report-only surfaces breakage without blocking the pipeline or demanding compatibility fixes. Source: user request 2026-08-10 (backlog SPEC-024).

## Requirements

### Requirement: Docker matrix job

The workflow SHALL add a `multi-version-integration` job, running **alongside** the existing embedded-server job, whose matrix covers `terminusdb/terminusdb-server:latest`, at least one v12.x tag, and at least one v11.x tag.

#### Scenario: Matrix spans latest, v12.x, v11.x
- **GIVEN** the workflow triggering on push/PR/workflow_dispatch
- **WHEN** the matrix job runs
- **THEN** each matrix cell spins a real server from the pinned docker tag and runs the client integration tests against it

### Requirement: latest tag is priority and blocking

The `latest` matrix cell SHALL be blocking (failure fails the job). Non-latest cells SHALL be non-blocking (failure is recorded, job continues).

#### Scenario: latest failure blocks
- **GIVEN** a `latest` cell failure
- **WHEN** the job completes
- **THEN** the overall job is red

#### Scenario: v11.x failure does not block
- **GIVEN** a v11.x cell failure
- **WHEN** the job completes
- **THEN** the overall job stays green (report-only)

### Requirement: Report-only for non-latest versions

The workflow SHALL NOT modify source code to accommodate non-latest versions. On failure it SHALL generate a markdown report artifact (version, failing tests, reproduction hints) sufficient to open an issue manually — no auto-issue.

#### Scenario: Report artifact generated
- **GIVEN** a non-latest cell failure
- **WHEN** the job finishes
- **THEN** a markdown report artifact is uploaded with version, failure list, and reproduction hints

### Requirement: act compatibility

The job SHALL be runnable locally with `act` (container-based services, no `sudo`-dependent setup beyond documented prerequisites), and the workflow file SHALL document the local invocation.

#### Scenario: Local act run
- **GIVEN** `act` 0.2.89 installed locally
- **WHEN** the documented act invocation runs the matrix job
- **THEN** the matrix executes against the same docker tags and produces the same report behavior

## Acceptance Criteria

- AC-1: Given a workflow run, when the matrix executes, then `latest` runs blocking while v12.x/v11.x cells run non-blocking.
- AC-2: Given a v11.x cell failure, when the job finishes, then a markdown report artifact is uploaded and the job is not failed.
- AC-3: Given `act` installed, when the documented `act -j <job>` runs locally, then the matrix executes against the docker tags.
- AC-4: Given any non-latest failure, when the report is generated, then no source file was modified for compatibility.
- AC-5: Given the generated report, when read, then it contains server version, failing tests, and reproduction hints sufficient to draft an issue.
- AC-6: Given the workflow file, when inspected, then the embedded-server job (`tests.yml`) is unchanged and the matrix job is additive.

## Technical Design

### Workflow sketch (`tests.yml` addition or new `matrix.yml`)

```yaml
name: Multi-version integration matrix

on:
  workflow_dispatch:
  push:
    branches: [main]
  pull_request:

jobs:
  matrix:
    strategy:
      fail-fast: false
      matrix:
        include:
          - version: latest          # blocking — priority
            blocking: true
          - version: v12.1.0         # v12.x pin
            blocking: false
          - version: v11.0.0         # v11.x legacy — report-only
            blocking: false
    steps:
      - run: docker run -d -p 6363:6363 terminusdb/terminusdb-server:${{ matrix.version }}
      - run: cargo test -p terminusdb-client --tests --no-fail-fast   # per cell
      - if: failure() && !matrix.blocking
        run: scripts/matrix-report.sh ${{ matrix.version }} > report.md
      - if: failure() && !matrix.blocking
        uses: actions/upload-artifact@v4
        with: { name: "matrix-report-${{ matrix.version }}", path: report.md }
```

### act notes
- Local run: `act -j matrix -s GITHUB_TOKEN=...` (or `act pull_request -j matrix`)
- Docker-tag availability: `docker pull terminusdb/terminusdb-server:<tag>` first; pins chosen from Docker Hub tags at spec implementation time
- `fail-fast: false` is required so every cell produces its report
- Client is v12-only — v11.x failures are expected; the report is the deliverable, not green tests

### Report script (`scripts/matrix-report.sh` in the fork)
- Input: server version
- Output: markdown with version, `cargo test` failure summary (test names), server tag, date, reproduction steps (docker run + test command)

## Test Plan

- [ ] `act -j matrix` locally: all three cells run, `latest` green against current `latest` tag (or blocks red)
- [ ] Simulated v11.x failure: report artifact uploaded, job green (AC-2)
- [ ] Full workflow run on fork `main` (or PR branch): embedded job untouched (AC-6)

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-25 | Docker Hub tag availability drift (pins disappear/rename) | Medium | Pin at implementation time; document tag provenance in workflow comment |
| R-26 | act service/network quirks differ from GH-hosted runners | Low | Document `--container-architecture`/docker flags in the workflow comment |
| R-27 | v11 server startup incompatible with v12 client calls (expected failures) | Low | That is the point — report-only path already handles it |
| R-28 | Matrix runtime cost (3 full server spins) on CI | Medium | Keep cells to the client crate only; reuse build cache from main job |

## Relationship to Other Specs

- SPEC-023: contribution ladder — this matrix provides additional acceptance evidence and issue material for older versions
- SPEC-008 (archived): embedded v12.1 verification remains the primary loop (unchanged, AC-6)
- RISK-004: fork drift — version-breakage visibility reduces surprise on future upgrades
- Backlog SPEC-024 (planned): this spec's intake entry
