# AGENTS.md — Frontend (Bit workspace)

Frontend conventions for AI agents and E2E testing. Derived from
`.wip/todo-friently-e2e-to-code-agents.md` (see SPEC-018).

## Commands

- `bit start` — dev server (component preview + app)
- `bit build <component>` — build pipeline (tsc + vite)
- `bit create react-app <name>` — new app component
- `bit install <pkg>` — add dependencies
- `bit tag` / `bit export` — release to `@coop-codes`

## Scope

- Default scope: `coop-codes.network-graph` (SPEC-019)
- Component ids: `@coop-codes/network-graph.<namespace>.<name>`
- Layout (Bit standard, no SSR — client-only apps):
  - `network-graph/apps/<name>` — app shells composing components
  - `network-graph/ui/<name>` — visual components (graph, entity-drawer)
  - `network-graph/hooks/<name>` — logic hooks (use-event-stream)
  - `network-graph/services/<name>` — future backend/service components
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
