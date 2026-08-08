# ADR-003: data-testid Naming Convention

- **Status:** Accepted (2026-08-07)
- **Source:** user prompt (SPEC-017/018) + DDD brainstorm
- **Standard:** `.standards/options/frontend-component-conventions.ai.yaml`

## Context
Generated `data-testid` values (e.g. `layout-select`) are generic and
collide-prone across components. Selectors are a stable E2E contract.

## Decision
`data-testid` values are **context-prefixed, kebab-case, intent-descriptive**:
- `layout-select` → `graph-select-layout` (data-graph context, intent: choose layout)
- testid prefixes derive from the owning context (`graph-` for data-graph,
  `entity-` for governance, `iam-`, `schema-`, `app-`)
- `entity-drawer` → `entity-drawer` stays (already context-prefixed)

Rules:
1. Prefix = owning context (graph-, entity-, stream-, workspace-).
2. Describes the user intent, not the implementation.
3. Conventions apply to NEW components; existing selectors migrate
   incrementally (never a mass churn in one change).

## Consequences
- E2E and AI agents get deterministic, collision-free selectors.
- Component specs assert the new selectors (SPEC-018 AC-1).

## Alternatives Considered
- Keep generator defaults — rejected: generic and collision-prone.
- No prefixing, longer names — rejected: loses context signal.
