# SPEC-018 Feature: Frontend Platform — AI-Agent-Friendly E2E Testability

<!-- status: Archived -->

## Overview

Ratify the frontend E2E conventions from `.wip/todo-friently-e2e-to-code-agents.md` (applied in the spike) as binding requirements for all frontend components: stable `data-testid` selectors, DOM state reflection, semantic/ARIA markup, `window.__APP_READY__`, structured error events, and Playwright-ready Page Object Models. Formalizes the composable component structure (apps/ui/hooks/services) and the API-consumption contract.

**Backlog source:** SPEC-018 (promoted per the Spec Intake Rule; user request + .wip guide).

## Motivation

AI agents and E2E tooling fail on volatile selectors (CSS classes, UI copy) and timing-based waits. The conventions were applied ad-hoc in the spike; without a spec they drift as new components land. This spec makes them the contract — mirroring SPEC-017's BDD/E2E organization on the backend.

## Requirements

### Requirement: Stable Selectors

Every interactive component SHALL expose persistent `data-testid` attributes; tests and agents MUST NOT depend on CSS class names or user-visible copy.

#### Scenario: Button has a stable selector
- **GIVEN** an interactive element
- **WHEN** rendered
- **THEN** it carries a `data-testid` that does not change across refactors or copy edits

### Requirement: DOM State Reflection

Components SHALL reflect async/lifecycle state on elements via `data-loading`, `data-state`, and `data-error` attributes.

#### Scenario: Loading state is observable
- **GIVEN** an in-flight async operation
- **WHEN** the element is queried
- **THEN** `data-loading="true"` is present without visual inspection

### Requirement: Accessible DOM

Components SHALL use semantic elements and explicit ARIA (`role="dialog"`, `aria-expanded`, `aria-label`) for custom widgets.

### Requirement: Readiness Flag

The app SHALL set `window.__APP_READY__ = true` after its first rendered frame; E2E agents SHALL assert on it rather than sleeping.

### Requirement: Structured Errors

Error boundaries and network failures SHALL log `{"@type":"app:Error", context, message, timestamp}` and dispatch `app:error` events; bare console logs are prohibited.

### Requirement: Composable Structure

Frontend components SHALL follow the layout `network-graph/{apps,ui,hooks,services}/<name>` with app shells composing reusable components (no monoliths).

### Requirement: API Consumption Contract

Consumers SHALL bootstrap state from the SSE genesis replay (`/events?cursor=`), reconnect with the `id:` cursor (lossless), treat `api:warnings` as expected (schemaless), and prefer OpenAPI-generated clients (SPEC-016).

## Acceptance Criteria

- AC-1: Given any interactive component, when rendered, then interactive elements carry stable `data-testid`s.
- AC-2: Given an in-flight async operation, when queried, then `data-loading`/`data-state` reflect it.
- AC-3: Given the app boot, when the first frame renders, then `window.__APP_READY__` is set.
- AC-4: Given a network failure, then a structured `app:error` event fires.
- AC-5: Given the workspace, when components are listed, then they follow the apps/ui/hooks/services layout.
- AC-6: Given a component consuming the API, then it uses the SSE genesis replay + cursor reconnect and tolerates `api:warnings`.

## Technical Design

- **Conventions file:** `frontend/AGENTS.md` is the normative reference (already written); the spec binds it
- **Enforcement:** a lint check for `data-testid` presence on interactive elements (component specs in Bit's test task); Playwright smoke suite asserting `__APP_READY__` + canvas/drawer selectors (SPEC-017 integration later)
- **Component inventory:** `apps/coop-graph-app` (shell), `ui/graph` (G6), `ui/entity-drawer` (antd form), `hooks/use-event-stream` (SSE) — all already conformant

## Test Plan

- [x] Component specs: `data-testid` presence + state reflection on ui/graph + ui/entity-drawer (AC-1, AC-2) — 6 tests green
- [ ] E2E (Playwright, later): `__APP_READY__` wait, canvas render, drawer open, live-update lands (AC-3, AC-6)
- [x] Unit: `reportError` emits structured `app:error` (AC-4) — hooks spec green
- [x] Structure check: `frontend/check-layout.mjs` — network-graph/{apps,ui,hooks,services} (AC-5)

> Note (2026-08-07): drawer save-failure error UI is exercised via Playwright E2E (SPEC-017), not jsdom — antd v6 form submission is unreliable under jsdom. The `app:error` dispatch contract is unit-covered in hooks/use-event-stream.

---

## MODIFIED Requirements (delta — 2026-08-07, orchestrator)

- **Held — E2E suite (AC-3, AC-6, drawer error-UI, POMs):** deferred to **SPEC-021** (backlog). Until it lands, frontend verification is **manual orchestrator-driven UI testing** (`bit run` + browser).
- AC-3/AC-6 status: **held** (spike evidence only, not verified by an E2E suite).
- No changes to the conventions (AC-1/2/4/5 remain enforced by component specs).

## Open Risks

| ID | Risk | Impact | Mitigation |
|----|------|--------|------------|
| R-26 | Bit test task integration for lint checks | Low | Bit env testers (vitest); or a workspace-level script in SPEC-007 |
| R-27 | Playwright suite needs the running API + TerminusDB | Medium | Compose fixture + SPEC-010 remote test execution later |

## Relationship to Other Specs

- SPEC-017 (planned): BDD/E2E organization — Playwright POMs + bounded-context feature dirs land there
- SPEC-013/016: API + OpenAPI contract the consumption rules
- SPEC-004: SSE genesis replay is the bootstrap mechanism
- SPEC-009: registry-driven forms (additionalType warnings tolerated per schemaless philosophy)


---

## ARCHIVED (2026-08-07)

- **Verification commit:** `e543a58` (enforcement: component specs + check-layout)
- **Evidence:** 7 frontend tests green (ui/graph, ui/entity-drawer, hooks/use-event-stream specs); `frontend/check-layout.mjs` green; spike smoke test (`7887709`, `b27d661`) + browser verification by orchestrator (manual UI testing policy per SPEC-021)
- **ACs:** AC-1/AC-2/AC-4/AC-5 enforced by tests; **AC-3/AC-6 held** — deferred to SPEC-021 (Playwright E2E suite)
- **Status change:** Implemented → Archived. Conventions remain normative via `.standards/options/frontend-component-conventions.ai.yaml` + `frontend/AGENTS.md`. Re-check (spec-vs-code convergence) if touched by future work (e.g. SPEC-020 opportunistic moves).
