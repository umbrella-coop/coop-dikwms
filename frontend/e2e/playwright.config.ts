import { defineConfig } from '@playwright/test';

/**
 * Real-browser E2E (SPEC-027): chrome-headless-shell against the live stack.
 *
 * Prerequisites (running services):
 *   - docker compose up -d terminusdb          (:6363)
 *   - API server:  cargo run -p api            (:8080, backend/)
 *   - dev server:  npm run dev (bit run app)   (:3100, frontend/)
 *   - an imported dataset (examples/git-codebase-1/ runbook)
 */
export default defineConfig({
  testDir: './tests',
  timeout: 60_000,
  retries: 0,
  workers: 1,
  use: {
    baseURL: 'http://localhost:3100',
    channel: 'chromium-headless-shell',
    headless: true,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  reporter: [['list']],
});
