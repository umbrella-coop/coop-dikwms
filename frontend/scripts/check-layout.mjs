// SPEC-020 AC-8: frontend structure check — the dikwms scope layout.
// Top-level namespaces: {app, hook, ui}. ui/ holds bounded-context dirs
// from the whitelist; every component dir must expose an index.ts(x).
// Usage: node frontend/scripts/check-layout.mjs

import { readdirSync, existsSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', 'dikwms');
const namespaces = new Set(['app', 'hook', 'type', 'ui']);
const contexts = new Set(['data-graph', 'data-graph-antv-g6', 'data-graph-governance', 'iam', 'data-schema-registry']);
const errors = [];

if (!existsSync(root)) {
  console.error(`dikwms/ not found at ${root}`);
  process.exit(1);
}

for (const entry of readdirSync(root)) {
  if (entry === 'node_modules') continue;
  if (!namespaces.has(entry)) {
    errors.push(`unexpected namespace: dikwms/${entry} (allowed: ${[...namespaces].join(', ')})`);
    continue;
  }
  const containers = entry === 'ui' ? contexts : new Set();
  for (const comp of readdirSync(join(root, entry))) {
    if (comp === 'node_modules') continue;
    const compRoot = join(root, entry, comp);
    const stat = existsSync(compRoot) ? statSync(compRoot) : null;
    if (!stat || !stat.isDirectory()) continue;
    if (entry === 'ui') {
      if (!containers.has(comp)) {
        errors.push(`unknown context: dikwms/ui/${comp} (allowed: ${[...containers].join(', ')})`);
        continue;
      }
      for (const sub of readdirSync(compRoot)) {
        if (sub === 'node_modules') continue;
        const subRoot = join(compRoot, sub);
        if (!statSync(subRoot).isDirectory()) continue;
        if (!existsSync(join(subRoot, 'index.ts')) && !existsSync(join(subRoot, 'index.tsx'))) {
          errors.push(`component ${entry}/${comp}/${sub} missing index.ts`);
        }
      }
      continue;
    }
    if (!existsSync(join(compRoot, 'index.ts')) && !existsSync(join(compRoot, 'index.tsx'))) {
      errors.push(`component ${entry}/${comp} missing index.ts`);
    }
  }
}

if (errors.length) {
  console.error('STRUCTURE ERRORS:');
  for (const e of errors) console.error(`  - ${e}`);
  process.exit(1);
}
console.log('layout ok: dikwms/{app,hook,type,ui} + context whitelist');
