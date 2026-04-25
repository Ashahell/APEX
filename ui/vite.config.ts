/// <reference types="vitest" />
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';
// Bundle analyzer - run with: npm run build && npx vite-bundle-visualizer dist/stats.html
// or: npm run preview -- --open

const manualChunks = {
  'vendor-react': ['react', 'react-dom'],
  'vendor-markdown': ['react-markdown', 'react-syntax-highlighter'],
  'vendor-radix': [
    '@radix-ui/react-dialog',
    '@radix-ui/react-dropdown-menu',
    '@radix-ui/react-scroll-area',
    '@radix-ui/react-tabs',
    '@radix-ui/react-tooltip',
  ],
  'vendor-motion': ['framer-motion'],
  'vendor-query': ['@tanstack/react-query'],
};

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks,
      },
    },
    chunkSizeWarningLimit: 500,
  },
  server: {
    host: '0.0.0.0',
    port: 8083,
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:3000',
        ws: true,
      },
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{js,ts,jsx,tsx}'],
  },
});
