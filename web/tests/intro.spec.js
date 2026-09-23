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
  await page.goto('/index.html?seed=1&mute');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'true');
});
