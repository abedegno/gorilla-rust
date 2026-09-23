import { test, expect } from '@playwright/test';
import { INTRO, BORDER, startGame, canvasDiff } from './helpers.js';

test('the intro screen matches the original to the pixel', async ({ page }) => {
  await startGame(page);
  await expect.poll(() => canvasDiff(page, INTRO, BORDER)).toEqual({ diff: 0 });
});

test('a browser without Web Audio still gets a playable game', async ({ page }) => {
  // Not muted, so the game really does try to create an audio context.
  await page.addInitScript(() => {
    delete window.AudioContext;
    delete window.webkitAudioContext;
  });
  await startGame(page, '?seed=1');
  await expect.poll(() => canvasDiff(page, INTRO, BORDER)).toEqual({ diff: 0 });
});

test('the keypad is hidden on a desktop', async ({ page }) => {
  await startGame(page);
  await expect(page.locator('#keypad')).toBeHidden();
});

test('?mute starts muted', async ({ page }) => {
  await page.goto('index.html?seed=1&mute');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'true');
});

test('?mute is remembered, as the mute button is', async ({ page }) => {
  // A keyboard player cannot reach the button, so ?mute is their way to it.
  await page.goto('index.html?seed=1&mute');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'true');
  await page.goto('index.html?seed=1');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'true');
});

test('the start button waits until the game has loaded', async ({ page }) => {
  // Hold the game's code back, so the page can be seen before it arrives.
  let release;
  const held = new Promise((resolve) => {
    release = resolve;
  });
  await page.route('**/pkg/gorillas_bg.wasm', async (route) => {
    await held;
    await route.continue();
  });
  await page.goto('index.html?seed=1&mute');
  const overlay = page.locator('#start');
  await expect(overlay).toBeDisabled();
  await expect(overlay).toHaveText('Loading…');

  release();
  await expect(overlay).toBeEnabled();
  await expect(overlay).toHaveText('Click or tap to start');
  await overlay.click();
  await expect.poll(() => canvasDiff(page, INTRO, BORDER)).toEqual({ diff: 0 });
});

test('a game that cannot load says so', async ({ page }) => {
  await page.route('**/pkg/gorillas.js', (route) => route.fulfill({ status: 404 }));
  await page.goto('index.html?seed=1&mute');
  await expect(page.locator('#error')).toBeVisible();
  await expect(page.locator('#error')).toContainText('could not load the game');
  await expect(page.locator('#start')).toBeDisabled();
});

test('page buttons are pointer-only: Tab and Space cannot reach or press mute', async ({
  page,
}) => {
  await startGame(page);
  await expect.poll(() => canvasDiff(page, INTRO, BORDER)).toEqual({ diff: 0 });

  const before = await page.locator('#mute').getAttribute('aria-pressed');
  for (let i = 0; i < 5; i++) {
    await page.keyboard.press('Tab');
    await expect(page.locator('#mute')).not.toBeFocused();
  }

  await page.keyboard.press('Space');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', before);
});
