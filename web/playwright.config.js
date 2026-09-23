import { defineConfig, devices } from '@playwright/test';

// The page is served as plain files, exactly as GitHub Pages serves it.
// Python's http.server gives .wasm the application/wasm type that
// streaming compilation needs.
export default defineConfig({
  testDir: './tests',
  timeout: 60_000,
  expect: { timeout: 15_000 },
  use: { baseURL: 'http://127.0.0.1:8123' },
  webServer: {
    command: 'python3 -m http.server 8123 --bind 127.0.0.1',
    url: 'http://127.0.0.1:8123/index.html',
    reuseExistingServer: !process.env.CI,
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],
});
