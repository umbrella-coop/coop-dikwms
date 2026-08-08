# ADR-002: Frontend Component Directory Layout (context-first)

- **Status:** Accepted (2026-08-07)
- **Source:** DDD brainstorm (docs/ddd/bounded-contexts.md, approved)
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

## MODIFIED (2026-08-08, SPEC-020 implemented — supersedes the above)

- Bit scope is **`coop-codes.dikwms`** (was `coop-codes.network-graph`); workspace dir `frontend/dikwms/`.
- Realized layout (codebase is the source of truth, docs follow it):
  - `frontend/dikwms/app/` — app shells (component `app`)
  - `frontend/dikwms/hook/<name>` — hooks, context in the hook name (`hook/use-data-graph-sse`)
  - `frontend/dikwms/ui/<context>/<name>` — visual components (`ui/data-graph/canvas`, `ui/data-graph/node-drawer`, `ui/data-graph/layout-select`)
- `frontend/check-layout.mjs` enforces `{app, hook, ui}` + context whitelist (AC-8).
- Restructure done via `bit rename -s/-p` + `bit move` per SPEC-020 (commits `1fc67fb`, `c0d3511`, `f5d92c8`); future moves remain opportunistic.

## Alternatives Considered
- Keep Bit standard layout — rejected: type-based navigation.
- Flat per-component dirs — rejected: no context signal.

- **Spelling correction (2026-08-08):** canonical name is **`dikwms`** (DIKW+ms) — scope `coop-codes.dikwms`, dir `frontend/dikwms/` (typo `diwkms` fixed everywhere).

- **Rendering-engine namespace (2026-08-08):** G6 implementation lives at
  `ui/data-graph-antv-g6/{canvas,layout-select}` (renderer SPI split; future
  renderers get their own namespace under `data-graph-*`); whitelist updated.

- **Shared types namespace (2026-08-08):** `type/<name>` added for wire-agnostic
  domain types (`type/core-v1`: DataGraphNode/Edge/Combo, DIKW Data layer, mirroring registry `core.v1`);
  `use-data-graph-sse` re-exports them; check-layout allows `type`.

- **Organization context (2026-08-08):** `ui/organization/` lands with
  `console-page-index-container` (org console index page shell —
  `Scope::Org` domain). Whitelist updated.
