import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import vueDevTools from 'vite-plugin-vue-devtools'
import AutoImport from 'unplugin-auto-import/vite'
import { getConfig } from './config/unified.config'

// Unified configuration that consolidates multiple config files
const config = getConfig()
export default defineConfig({
  plugins: [
    vue(),
    vueJsx(),
    vueDevTools(),
    AutoImport({
      imports: ['vue', 'vue-router', 'pinia'],
      dirs: [
        './src/appearance/composables/**',
        './src/appearance/directives/**',
        './src/bridge/client/index.ts',
        './src/bridge/client/zod.gen.ts',
      ],
      vueTemplate: true,
      dts: true,
      eslintrc: {
        enabled: true,
      },
    }),
  ],
  resolve: {
    alias: {
      '@/appearance': fileURLToPath(new URL('./src/appearance', import.meta.url)),
      '@/bridge': fileURLToPath(new URL('./src/bridge', import.meta.url)),
      '@/components': fileURLToPath(new URL('./src/appearance/components', import.meta.url)),
      '@/views': fileURLToPath(new URL('./src/appearance/views', import.meta.url)),
      '@/pages': fileURLToPath(new URL('./src/appearance/pages', import.meta.url)),
      '@/layouts': fileURLToPath(new URL('./src/appearance/layouts', import.meta.url)),
      '@/styles': fileURLToPath(new URL('./src/styles', import.meta.url)),
      '@/api': fileURLToPath(new URL('./src/bridge/api', import.meta.url)),
      '@/stores': fileURLToPath(new URL('./src/bridge/stores', import.meta.url)),
      '@/types': fileURLToPath(new URL('./src/bridge/types', import.meta.url)),
      '@/router': fileURLToPath(new URL('./src/bridge/router', import.meta.url)),
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  // Unified TypeScript configuration
  esbuild: {
    target: 'ESNext',
  },
  // Unified testing configuration
  test: {
    environment: config.testing.unit.environment,
    exclude: config.testing.unit.exclude,
    root: fileURLToPath(new URL('./', import.meta.url)),
    globals: config.testing.unit.globals,
  },
  // Unified formatting configuration
  css: {
    devSourcemap: true,
  },
  // Unified build configuration
  build: {
    target: 'ESNext',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['vue', 'vue-router', 'pinia'],
        },
      },
    },
  },
  // Unified development server configuration
  server: {
    port: 5173,
    host: true,
  },
  preview: {
    port: 4173,
    host: true,
  },
})
