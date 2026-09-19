import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte(), tailwindcss()],

  // Tauri espera un puerto fijo y falla si no está disponible.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: {
      // El backend Rust tiene su propio watcher.
      ignored: ['**/src-tauri/**'],
    },
  },
  // Variables de entorno expuestas al frontend.
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
})
