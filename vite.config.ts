import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
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
