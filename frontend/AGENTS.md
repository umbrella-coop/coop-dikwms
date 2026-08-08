# AGENTS.md — Frontend (Bit workspace)

Frontend conventions for AI agents and E2E testing. Derived from
`.wip/todo-friently-e2e-to-code-agents.md` (see SPEC-018).

## Commands

- `bit start` — workspace UI (component previews, http://localhost:3000)
- `bit build <component>` — build pipeline (tsc + vite)
- **Run an app:** `bit run <app> -p <port>` (e.g. `bit run app -p 3100`).
  R-28 RESOLVED (2026-08-07): the 404s were a PORT COLLISION — a foreign
  vite server held the default :3001; Bit's banner lies about the bound
  port. Always pass an explicit free port.
- `bit test` — component specs (vitest)
- `bit create react-app <name>` — new app component
- `bit install <pkg>` — add dependencies
- `bit tag` / `bit export` — release to `@coop-codes`

## Scope

- Default scope: `coop-codes.dikwms` (SPEC-020; formerly `coop-codes.network-graph` per SPEC-019)
- Component ids: `@coop-codes/dikwms.<namespace>.<name>` (dot-separated namespaces)
- Layout (SPEC-020 context-first, Bit standard, no SSR — client-only apps):
  - `dikwms/app/<name>` — app shells composing components (currently `app`)
  - `dikwms/hook/<name>` — logic hooks, context in the hook name (`use-data-graph-sse`)
  - `dikwms/type/<name>` — shared wire-agnostic types (`type/core-v1`: DataGraphNode/Edge/Combo, mirrors schema-registry `core.v1`)
  - `dikwms/ui/<context>/<name>` — visual components per bounded context (`data-graph`, `data-graph-antv-g6` — G6 rendering engine, reserved: `data-graph-governance`, `iam`, `data-schema-registry`)
  - Reserved namespaces are created when their first component lands — no empty dirs
- Enforced by `node frontend/scripts/check-layout.mjs` (AC-8)
- Per-component `vite.config.js`/`index.html` are Bit structural — do not collapse

## E2E Conventions (mandatory for new components)

1. **data-testid** on every interactive component — never CSS classes or UI copy
   (e.g. `data-testid="entity-save-btn"`).
2. **DOM state reflection** — `data-loading`, `data-state="open|closed"`,
   `data-error` attributes for async/lifecycle states.
3. **Semantic markup + ARIA** — native `<button>/<nav>/<form>/<main>`;
   `role="dialog"`, `aria-expanded`, `aria-label` on custom widgets.
4. **Readiness flag** — set `window.__APP_READY__ = true` once the app has
   rendered its first frame (E2E agents assert on it, never sleep).
5. **Structured errors** — error boundaries and network failures log
   `{"@type":"app:Error", context, message, timestamp}` and dispatch
   `app:error` events; never bare console.log.
6. **Page Object Models** — reusable test helpers over these selectors
   (Playwright), not raw automation scripts.

## API Contract

- Base: `VITE_API_BASE` (default `http://localhost:8080`)
- OpenAPI: `GET /openapi.json` (SPEC-016) — generate clients from it
- Live updates: SSE `GET /events?cursor=` (SPEC-004) — genesis replay
  bootstraps state; `id:` cursor enables lossless reconnect
- Schemaless: `api:warnings` in responses are expected — never treat as errors

## Backend Note

The backend runs on nightly Rust (`RUSTUP_TOOLCHAIN=nightly cargo ...`).
The API server binary: `cargo run -p api` against the compose TerminusDB
(`docker compose up -d`).
