import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, type UserConfig } from 'vite';

// Vitest configuration type (inline to avoid version conflicts)
interface VitestConfig {
  include?: string[];
  exclude?: string[];
  environment?: string;
  globals?: boolean;
  setupFiles?: string[];
  server?: {
    deps?: {
      inline?: (string | RegExp)[];
    };
  };
}

interface ConfigWithVitest extends UserConfig {
  test?: VitestConfig;
}

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
    exclude: ['e2e/**'],
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/lib/test-utils/vitest-setup.ts'],
    server: {
      deps: {
        inline: [/svelte/],
      },
    },
  },
  resolve: {
    conditions: ['browser'],
  },
} as ConfigWithVitest);
