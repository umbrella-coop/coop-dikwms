import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Bit DX (SPEC-022): watch the org scope so component edits hot-reload in
// `bit run`, and exclude it from dep optimization (it must appear in the
// dependency graph to trigger HMR). See bit.dev blog "Local Cross-Project
// Component Development with Bit Link Target".
export default defineConfig({
  plugins: [react()],
  server: {
    watch: {
      ignored: ['!**/node_modules/@coop-codes/**'],
    },
  },
  optimizeDeps: {
    exclude: ['@coop-codes'],
    // CJS deps reached through the linked org scope get served raw from
    // /@fs/ without an interop wrapper → "does not provide an export named
    // ..." crashes. Force pre-bundling (ESM interop) for the offenders.
    include: ['eventemitter3', '@antv/g6', 'react-is'],
  },
});
