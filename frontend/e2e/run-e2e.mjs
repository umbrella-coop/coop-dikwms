import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// Run the Playwright suite from the e2e/ package (own toolchain, independent
// of the Bit workspace). Usage: node e2e/run-e2e.mjs [playwright args]
const e2eDir = fileURLToPath(new URL('.', import.meta.url));
const res = spawnSync('npx', ['playwright', 'test', ...process.argv.slice(2)], {
  stdio: 'inherit',
  cwd: e2eDir,
});
process.exit(res.status ?? 1);
