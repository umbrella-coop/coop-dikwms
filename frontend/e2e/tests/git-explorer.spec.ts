import { test, expect, type Page } from '@playwright/test';

/**
 * SPEC-027 real-browser E2E for the git explorer (chrome-headless-shell).
 * Runs against the live stack: dev server (:3100) + API (:8080) + docker
 * TerminusDB with the imported terminusdb/terminusdb dataset (898 commits).
 */

const TOTAL_COMMITS = 898;

async function openGitExplorer(page: Page) {
  await page.goto('/');
  await page.waitForFunction(
    () => (window as unknown as { __APP_READY__?: boolean }).__APP_READY__ === true,
    undefined,
    { timeout: 30_000 },
  );
  await page.getByText('Explore', { exact: true }).click();
  await page.getByText('Git explorer', { exact: true }).click();
  await expect(page.getByTestId('git-explorer')).toBeVisible({ timeout: 30_000 });
}

function countFromTag(text: string | null): number {
  const m = text?.match(/^(\d+) \/ \d+ commits/);
  return m ? Number(m[1]) : NaN;
}

test('app boots with zero page/console errors (eventemitter3 interop fixed)', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', (e) => errors.push(`pageerror: ${e.message}`));
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(`console: ${m.text()}`);
  });
  await openGitExplorer(page);
  await expect(page.getByTestId('git-explorer-count')).toBeVisible({ timeout: 30_000 });
  expect(errors).toEqual([]);
});

test('git explorer loads the real graph and insight cards', async ({ page }) => {
  await openGitExplorer(page);
  await expect(page.getByTestId('git-explorer-count')).toContainText(
    `/ ${TOTAL_COMMITS} commits`,
    { timeout: 30_000 },
  );
  const cards = await page.locator('[data-testid^="insight-card-"]').count();
  expect(cards).toBeGreaterThan(0);
  await expect(page.getByText('bus-factor').first()).toBeVisible();
  // The canvas must fill the viewport height, not collapse to its 300px floor.
  const canvasHeight = await page
    .getByTestId('coop-canvas-canvas')
    .evaluate((el) => el.clientHeight);
  expect(canvasHeight).toBeGreaterThan(400);
  // Edges must actually reach the canvas (parent links + author links).
  const edgeCount = await page.evaluate(
    () =>
      (window as unknown as { __GIT_EXPLORER__?: { edges: () => number } }).__GIT_EXPLORER__!
        .edges(),
  );
  expect(edgeCount).toBeGreaterThan(0);
});

test('timeline slider narrows the visible commit window', async ({ page }) => {
  await openGitExplorer(page);
  const count = page.getByTestId('git-explorer-count');
  await expect(count).toContainText(`/ ${TOTAL_COMMITS} commits`, { timeout: 30_000 });
  const before = countFromTag(await count.textContent());

  // Drag the second (upper) range thumb left — the window end moves earlier
  // and the visible commit count shrinks.
  const slider = page.getByRole('slider').last();
  const box = (await slider.boundingBox())!;
  await page.mouse.move(box.x + box.width * 0.9, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.4, box.y + box.height / 2, { steps: 8 });
  await page.mouse.up();

  await expect
    .poll(async () => countFromTag(await count.textContent()))
    .toBeLessThan(before);
});

test('commit drawer opens with metadata and property-set history', async ({ page }) => {
  await openGitExplorer(page);
  await page.waitForFunction(() => {
    const h = (window as unknown as { __GIT_EXPLORER__?: { visible: number } }).__GIT_EXPLORER__;
    return h !== undefined && h.visible > 0;
  });
  const firstId = await page.evaluate(
    () =>
      (window as unknown as { __GIT_EXPLORER__?: { commitIds: () => string[] } })
        .__GIT_EXPLORER__!
        .commitIds()[0],
  );
  await page.evaluate(
    (id) =>
      (window as unknown as { __GIT_EXPLORER__?: { select: (id: string) => void } })
        .__GIT_EXPLORER__!
        .select(id),
    firstId,
  );
  await expect(page.getByTestId('git-explorer-drawer-meta')).toBeVisible();
  await expect(page.getByTestId('git-explorer-drawer-history')).toContainText('v1');
});

test('author filter narrows the commit count', async ({ page }) => {
  await openGitExplorer(page);
  const count = page.getByTestId('git-explorer-count');
  await expect(count).toContainText(`/ ${TOTAL_COMMITS} commits`, { timeout: 30_000 });

  // First combobox is the author filter (the second is the canvas layout select).
  await page.getByRole('combobox').first().click();
  const option = page.locator('.ant-select-item-option').first();
  await option.click();

  await expect
    .poll(async () => countFromTag(await count.textContent()))
    .toBeLessThan(TOTAL_COMMITS);
});
