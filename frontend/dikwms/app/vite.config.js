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
  },
});
