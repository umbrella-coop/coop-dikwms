// SPEC-018 AC-5: frontend structure check — component dirs must follow the
// Bit standard layout network-graph/{apps,ui,hooks,services}/<name>.
// Usage: node frontend/check-layout.mjs

import { readdirSync, existsSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), 'network-graph');
const allowed = new Set(['apps', 'ui', 'hooks', 'services']);
const errors = [];

if (!existsSync(root)) {
  console.error(`network-graph/ not found at ${root}`);
  process.exit(1);
}

for (const entry of readdirSync(root)) {
  if (entry === 'node_modules') continue;
  if (!allowed.has(entry)) {
    errors.push(`unexpected namespace: network-graph/${entry} (allowed: ${[...allowed].join(', ')})`);
  } else {
    for (const comp of readdirSync(join(root, entry))) {
      if (comp === 'node_modules') continue;
      const compRoot = join(root, entry, comp);
      const stat = existsSync(compRoot) ? statSync(compRoot) : null;
      if (!stat || !stat.isDirectory()) continue;
      // each component dir must contain an index.ts
      if (!existsSync(join(compRoot, 'index.ts')) && !existsSync(join(compRoot, 'index.tsx'))) {
        errors.push(`component ${entry}/${comp} missing index.ts`);
      }
    }
  }
}

if (errors.length) {
  console.error('STRUCTURE ERRORS:');
  for (const e of errors) console.error(`  - ${e}`);
  process.exit(1);
}
console.log('layout ok: network-graph/{apps,ui,hooks,services}');
