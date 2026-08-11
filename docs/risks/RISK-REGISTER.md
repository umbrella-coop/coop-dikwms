# Risk Register — dikwms (coop-graph-network)

**Last Updated**: 2026-08-11
**Source**: Phase-0 project discovery (`/discover`, 2026-08-10)
**Conventions**: risk IDs cross-reference spec Open Risks where they exist (R-1/R-2 in SPEC-001, R-18/R-20 in SPEC-013). NEW risks get sequential IDs.

| ID | Category | Description | Likelihood | Impact | Level | Owner | Mitigation | Status |
|----|----------|-------------|-----------|--------|-------|-------|------------|--------|
| RISK-001 | Security | Unverified `X-Principal` header — spoofable authors, audit + ACL (SPEC-003) integrity at risk (SPEC-013 R-18) | High | High | Critical | @backend | SPEC-014 (planned) mandatory before production; tracked in backlog | Open |
| RISK-002 | Architecture | TerminusDB single-store persistence unverified end-to-end (SPEC-001 R-1) — gates SPEC-001 AC-6 | Medium | High | High | @backend | SPEC-006/008 parity work + real-server integration tests (`spec_001_persistence.rs`) | Mitigating |
| RISK-003 | Architecture | Schema pipeline codegen partially unverified (SPEC-001 R-2) — gates SPEC-001 AC-5 | Medium | Medium | Medium | @backend | `#[derive(TerminusDBModel)]` derive-macro chosen (2026-08-06); AC-5 closed for storage schema | Mitigating |
| RISK-004 | Dependencies | Fork drift: `third_party/terminusdb-rs` 367 commits past `pre-generics-merge` tag; nightly-only toolchain required | High | Medium | High | @orchestrator | Pin fork to `dev` branch (2026-08-10, pushed to origin); drift = commits past `origin/dev`; tag when a release point stabilizes | Mitigating |
| RISK-005 | Quality | Sparse unit tests: `stream-api` (1 SSE test) and `knowledge-domain` (0) — integration-only coverage | Medium | Low | Medium | @qa | Add unit tests before extending those crates (AGENTS.md TDD order) | Open |
| RISK-006 | Security | `TERMINUSDB_ADMIN_PASS: root` plaintext in `compose.yml` | Low | Medium | Medium | @devops | Dev-only fixture; rotate credentials and externalize secrets before any prod deployment | Accepted |
| RISK-007 | Process | Uncommitted work at discovery time (`frontend/dikwms/app/app.tsx` layout wiring) | Low | Low | Low | @orchestrator | Committed 2026-08-10 | Resolved |
| RISK-008 | Security | No rate limiting/quotas on API (SPEC-013 R-20) | Low | Medium | Medium | @backend | Deferred to SPEC-014 or SPEC-007 gates | Open |
| RISK-009 | Dependencies | Frontend deps Bit-managed per scope — version drift risk across `ui/` components | Medium | Low | Medium | @frontend | `bit install --add-missing-deps` + pnpm-lock discipline | Open |
| RISK-010 | Dependencies | Server-fork drift: embedded-server pin `ParapluOU/terminusdb` `v12.1-rc-paraplu.1` was **189 commits behind** upstream `12.1-rc` (measured 2026-08-11, last sync 2026-07-13) — silent divergence from released images (Docker `latest` = v12.0.7) | High | Medium | High | @backend | **Resolved 2026-08-11**: embedded server now defaults to official `terminusdb/terminusdb` with `TERMINUSDB_VERSION=auto-rc` (highest `-rc` branch, build-time) — no fork to drift; `server-release-watch` weekly workflow alerts on rc-line moves / pin-worthy rc tags (SPEC-024 delta) | Resolved |

## Status Lifecycle

```
Identified ──► Mitigating ──► Resolved ──► Closed
     │
     └──► Accepted (with justification)
```

## Update Protocol

- Append new rows; never rewrite history of closed risks.
- Take quarterly snapshots as `RISK-REGISTER-<YYYY-Qn>.md` when the register grows past ~15 active rows.
- Re-run `/discover --risks` after major milestones to re-evaluate likelihood/impact.
