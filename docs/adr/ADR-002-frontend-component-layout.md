# ADR-002: Frontend Component Directory Layout (context-first)

- **Status:** Accepted (2026-08-07)
- **Source:** DDD brainstorm (docs/brainstorm/ddd-bounded-contexts.md, approved)
- **Standard:** `.standards/options/ddd-bounded-context-org.ai.yaml`

## Context
Bit standard layout (`apps/ui/hooks/services`) organizes by component type.
The DDD map (ADR-001) requires context-first navigation.

## Decision
Frontend components live at `frontend/network-graph/<context>/<component>`:
- `canvas/graph`, `canvas/select-layout` (Graph Knowledge)
- `editor/entity-drawer` (Governance-adjacent entity editing — NOTE: re-scoped
  under its owning context when Governance UI lands; v1 keeps it in Graph
  Knowledge's editing surface)
- `stream/use-event-stream` (Stream)
- `workspace/coop-graph-app` (Workspace Shell)

Restructure via `bit move` (preserves history), never re-creation.

## Consequences
- Existing `ui/`, `hooks/`, `apps/` namespaces migrate to contexts.
- `frontend/check-layout.mjs` enforces the rule.
- Renames are batch `bit move` operations with testid stability preserved.

## MODIFIED (2026-08-07, orchestrator review)

- Layout is `ui/<context>/<component>` for visual components and hooks:
  - `ui/data-graph/canvas`, `ui/data-graph/select-layout`,
    `ui/data-graph/hook-use-event-stream`
  - `app/diwkms` (app shell)
- Hook components carry a `hook-` prefix within their context
  (`hook-use-event-stream`).
- Restructure via `bit move` per SPEC-020.

## Alternatives Considered
- Keep Bit standard layout — rejected: type-based navigation.
- Flat per-component dirs — rejected: no context signal.
