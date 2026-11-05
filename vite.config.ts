import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { readFileSync } from 'node:fs';
import path from 'path';

// Single source of truth for the version the UI reports. `package.json` is
// already required to match `tauri.conf.json` + `Cargo.toml` by the release
// workflow, so reading it here removes the last hand-maintained copy
// (`CURRENT_APP_VERSION`, which silently stayed at 0.2.20 while 0.2.22 shipped).
const pkg = JSON.parse(
  readFileSync(path.resolve(__dirname, 'package.json'), 'utf-8'),
) as { version: string };

export default defineConfig({
  plugins: [react()],
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    // Rust/Tauri writes many locked DLLs while compiling. Watching target/
    // makes chokidar crash with EBUSY on Windows, so keep build artifacts out
    // of the frontend watcher.
    watch: {
      ignored: ['**/src-tauri/target/**'],
    },
  },
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_'],
});
