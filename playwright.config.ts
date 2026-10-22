import { defineConfig } from '@playwright/test';

// Tests de extremo a extremo: la interfaz compilada, servida por `vite preview`,
// con el backend simulado de tests/fixtures/backend.ts. Corren en WebKit, el
// motor de Tauri en Linux y macOS, y en Chromium, el de WebView2 en Windows.
export default defineConfig({
  testDir: 'tests/integration',
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : 'list',
  use: {
    baseURL: 'http://localhost:4173',
    locale: 'es-ES',
    trace: 'retain-on-failure',
  },
  projects: [
    {
      name: 'webkit',
      use: {
        browserName: 'webkit',
        // El WebKit de Playwright se presenta como un Mac en cualquier sistema,
        // y la app esperaría Cmd en los atajos. Este es el de WebKitGTK, el que
        // ve Mosaic dentro de Tauri en Linux.
        userAgent: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko)',
      },
    },
    { name: 'chromium', use: { browserName: 'chromium' } },
  ],
  webServer: {
    command: 'pnpm build && pnpm preview --port 4173 --strictPort',
    url: 'http://localhost:4173',
    reuseExistingServer: !process.env.CI,
  },
});
