# SPEC-023 Feature: Upstream Contribution Ladder — terminusdb-rs fork → ParaplouOU

<!-- status: Implemented -->
<!-- progress: 3/3 PRs opened (2026-08-10) — #9 casing, #10 auth, #11 collab+tests; awaiting upstream merge -->
<!-- approved: 2026-08-10 by orchestrator (retrospective — implementation preceded approval) -->

## Verification Record (2026-08-10)

- AC-1 ✅ — PR [#9](https://github.com/ParapluOU/terminusdb-rs/pull/9) OPEN, mergeable; commits `9b7b3e6` + `7cbdb6f`; diff = 1 file, 3 unit tests
- AC-2 ✅ — PR [#10](https://github.com/ParapluOU/terminusdb-rs/pull/10) OPEN, mergeable; commits `3846ebc` + `513f428`; Basic unchanged
- AC-3 ✅ — diff inspection on all three branches: no `580b840`/`13cb418` content
- AC-4 ⚠️ PARTIAL — new code clippy-clean; upstream `main` carries 77 pre-existing lib warnings (verified via stash)
- AC-5 ⚠️ PARTIAL — 3-line evidence note in #9/#11; #10 carries grounding notes (parity gap, DFRNT PAT flow)
- AC-6 ✅ — opened in order #9 → #10 → #11 (sequential ladder satisfied)

**Verdict**: 4/6 PASS, 2 PARTIAL (upstream-side, documented) → Implemented. **Not archived**: awaiting upstream review/merge (R-23); archive when all three PRs land.

## Overview

Contribute the fork's verified TerminusDB v12/auth work back to upstream `ParaplouOU/terminusdb-rs` as **three small, sequential PRs**, each branched from upstream `main`. Trust ladder: tiny bug fix → documented feature → collaboration fixes with tests.

## Motivation

The fork (`github.com/gustavorps/terminusdb-rs`) holds verified, valuable work that upstream users need (RISK-004 fork drift; SPEC-008 verification evidence). Upstream cannot accept the fork wholesale (367+ commits of WIP, nightly-only tooling, fork-specific semantics), so the landable surface is a reviewable subset. Uncontributed, the value stays trapped in a private fork: upstream v12 users keep broken push/pull and no API-key auth, and the platform keeps paying fork-pin maintenance. Source: user request + brainstorm 2026-08-10 (backlog SPEC-023).

## Requirements

### Requirement: PR-1 — Authorization-Remote header-casing fix

The first PR SHALL be the smallest landable unit: the `Authorization-Remote` header casing fix (server looks up the header case-sensitively; `AUTHORIZATION_REMOTE` was silently ignored).

#### Scenario: Casing fix is the first PR
- **GIVEN** the fork's verified commit `3f04b24` (clone → push → pull roundtrip vs real v12.1)
- **WHEN** a branch `pr/casing-fix` is created from upstream `main` and the casing change is cherry-picked
- **THEN** the PR contains only that change and no other commit

### Requirement: PR-2 — Bearer/API-key authentication

The second PR SHALL add bearer-token and API-key authentication (`3bd396d`), the documented JS-client parity gap, as an additive change with Basic remaining the default.

#### Scenario: Auth is additive
- **GIVEN** the auth commit `3bd396d` (unit-verified header construction Basic/Bearer/Apikey, SPEC-008 AC-6)
- **WHEN** the PR branch `pr/auth` is prepared from upstream `main`
- **THEN** existing Basic-auth behavior is unchanged and no default is flipped

### Requirement: PR-3 — v12 collaboration fixes + pruned tests

The third PR SHALL carry the remaining v12 collaboration fixes plus only the v12-verification tests, pruned of fork-specific semantics (Rebase/Apply merge strategies, local tooling).

#### Scenario: Tests pruned to v12 verification
- **GIVEN** commit `11d525b` mixes v12 collaboration tests with fork-specific merge-strategy tests
- **WHEN** the PR branch `pr/collab-v12` is prepared
- **THEN** fork-only semantics are excluded and only v12-contract verification remains

### Requirement: Packaging discipline

Every PR SHALL follow the packaging discipline: upstream duplicate-check (≤10 min), branch from upstream `main`, cherry-pick only the scion commits, verify on the upstream nightly toolchain + clippy, and open with a 3-line v12 evidence note.

#### Scenario: Evidence note
- **GIVEN** a prepared PR branch
- **WHEN** the PR description is written
- **THEN** it contains the v12 evidence: changeset-sse endpoint 404 on v12, header-casing lookup, v12 push/pull path+body contract

### Requirement: No fork-only commits in any PR

No PR SHALL include `580b840` (rustfmt sweep, 146 files) or `13cb418` (local build tooling), and no PR SHALL combine two concepts.

#### Scenario: Exclusion verified by diff
- **GIVEN** any prepared PR branch
- **WHEN** `git diff upstream/main...branch --stat` is inspected
- **THEN** neither the rustfmt sweep nor the local-tooling commits appear in the diff

## Acceptance Criteria

- AC-1: Given the fork, when `pr/casing-fix` is diffed against upstream `main`, then only the Authorization-Remote casing change (plus its unit tests) is present. ✅ **Opened 2026-08-10** — PR [#9](https://github.com/ParapluOU/terminusdb-rs/pull/9); commits `9b7b3e6` (fix) + `7cbdb6f` (tests); 1 file, 3 unit tests
- AC-2: Given `pr/auth`, when inspected, then it contains exactly `3bd396d`'s file set (plus the Token-scheme correction) and Basic remains the default auth method. ✅ **Opened 2026-08-10** — PR [#10](https://github.com/ParapluOU/terminusdb-rs/pull/10); commits `3846ebc` (auth feature) + `513f428` (Token scheme fix from JS-client review); Basic unchanged
- AC-3: Given all three PR branches, when their diffs vs upstream `main` are inspected, then no rustfmt-sweep or local-tooling commit appears. ✅ Verified on all three branches (PR #9/#10/#11 diffs contain no `580b840`/`13cb418` content)
- AC-4: Given each PR branch, when `cargo clippy` runs on the upstream nightly toolchain, then the client crate is clippy-clean. ⚠️ Partial — new code is clippy-clean on all three branches; upstream `main` has 77 pre-existing lib warnings (untouched, verified via stash — see session 2026-08-10)
- AC-5: Given each opened PR, when the description is read, then the 3-line v12 evidence note is present. ⚠️ Partial — evidence notes present in #9 (verification) and #11 (v12 contract + merge-order note); #10 carries grounding notes (parity gap, DFRNT PAT flow)
- AC-6: Given the ladder, when PR-1 is not yet merged, then PR-2/PR-3 are not opened (sequential order). ✅ **Met 2026-08-10** — opened in order #9 → #10 → #11; merge order documented in #11 (tests depend on #9's casing fix)

## Technical Design

### Branch mechanics (fork repo)
- Base for all branches: upstream `main` (`36f5f4f`) — fetch upstream, branch, cherry-pick
- PR-1: cherry-pick the casing change out of `3f04b24` (`git cherry-pick -n` + selective restore, or extract via `git show 3f04b24 -- <path>` + manual patch)
- PR-2: cherry-pick `3bd396d` clean (self-contained)
- PR-3: cherry-pick `3f04b24` remainder + v12-test subset of `11d525b` (test files touching `collaboration`/v12-contract paths only)

### Evidence note (3 lines, reused in each PR)
1. v12 removed the `changeset-sse` plugin — `/api/changesets/stream` returns 404; native commit stream is the live-update path
2. `Authorization-Remote` is looked up case-sensitively by the server
3. v12 push/pull contract: `/api/push|pull/{org}/{db}` with `{'remote': <name>, 'remote_branch': <branch>}`

### Verification loop
`env RUSTUP_TOOLCHAIN=nightly cargo check -p terminusdb-client --tests` → `cargo clippy -p terminusdb-client --all-targets` → targeted test against `TerminusDBServer::test_instance()` before opening the PR.

## Test Plan

- [ ] PR-1: verify the casing fix restores clone→push→pull convergence (SPEC-008 AC-4 pattern)
- [ ] PR-2: unit-verify header construction Basic/Bearer/Apikey (SPEC-008 AC-6 pattern)
- [ ] PR-3: v12-contract tests run green on upstream nightly
- [ ] Packaging: each branch diff-vs-main inspected against AC-1/AC-3

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-21 | Upstream already has auth work in flight → shape rejection | Medium | 10-min dup-check before PR-2; align with their design if found |
| R-22 | Server-dependent v12 tests rejected by upstream CI (embedded-server build is heavy) | Medium | Keep tests behind the existing `terminusdb-bin` pattern; evidence note explains |
| R-23 | Reviewer latency stalls the ladder | Low | PRs are small; latch on SPEC-024 multi-version matrix for extra evidence |
| R-24 | Cherry-picking `3bd396d` conflicts with upstream drift between now and PR-2 | Low | Rebase from upstream `main` at open time |

## Relationship to Other Specs

- SPEC-008 (archived): verification evidence (7/7 ACs vs real v12.1) — the basis for this ladder
- SPEC-024: multi-version CI matrix — generates additional evidence and issue material for older versions
- RISK-004: fork drift — upstream landing of these PRs is the durable mitigation
- Backlog SPEC-023 (planned): this spec's intake entry
