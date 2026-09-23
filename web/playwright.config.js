import { defineConfig, devices } from '@playwright/test';

// The page is served as plain files, exactly as GitHub Pages serves it.
// Python's http.server gives .wasm the application/wasm type that
// streaming compilation needs.
//
// BASE_URL points the tests at a deployed copy instead, as the Pages
// workflow does after every deploy. It must end in a slash, so that the
// tests' relative paths land inside the site rather than at the host's
// root: https://abedegno.github.io/gorilla-rust/, not .../gorilla-rust.
const baseURL = process.env.BASE_URL;

export default defineConfig({
  testDir: './tests',
  timeout: 60_000,
  expect: { timeout: 15_000 },
  use: { baseURL: baseURL ?? 'http://127.0.0.1:8123/' },
  webServer: baseURL
    ? undefined
    : {
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
