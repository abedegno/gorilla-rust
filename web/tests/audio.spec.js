import { test, expect } from '@playwright/test';
import {
  INTRO,
  BORDER,
  startGame,
  canvasDiff,
  spyOnAudio,
} from './helpers.js';

// Nothing here can listen to the speakers, so these tests watch what the
// game asks Web Audio to do instead: every note is one oscillator, and
// every tone is meant to reach the speakers only through the master gain.

const oscillators = (page) => page.evaluate(() => window.__audio.oscillators);

/** The gain of every node wired straight to the speakers. */
const gainsToSpeakers = (page) =>
  page.evaluate(() => window.__audio.toDestination.map((n) => n.gain?.value ?? null));

async function waitForIntro(page) {
  await expect.poll(() => canvasDiff(page, INTRO, BORDER)).toEqual({ diff: 0 });
}

test('the intro tune plays', async ({ page }) => {
  await spyOnAudio(page);
  await startGame(page, '?seed=1');
  await expect.poll(() => oscillators(page)).toBeGreaterThan(0);
});

test('?mute plays no notes at all', async ({ page }) => {
  await spyOnAudio(page);
  await startGame(page, '?seed=1&mute');
  await waitForIntro(page);
  // The intro tune is scheduled before the intro screen is first shown, and
  // it is given time here anyway, so an oscillator made late would count.
  await page.waitForTimeout(1500);
  expect(await page.evaluate(() => window.__audio.contexts.length)).toBe(1);
  expect(await oscillators(page)).toBe(0);
});

test('the mute button silences a tune that is already sounding', async ({ page }) => {
  await spyOnAudio(page);
  await startGame(page, '?seed=1');
  await expect.poll(() => oscillators(page)).toBeGreaterThan(0);

  // One node reaches the speakers, the master gain, and it is at full
  // volume. Tones wired straight to the speakers would add one each.
  expect(await gainsToSpeakers(page)).toEqual([1]);
  // The intro tune has notes still to come.
  const { now, end } = await page.evaluate(() => ({
    now: window.__audio.contexts[0].currentTime,
    end: window.__audio.lastStop,
  }));
  expect(now).toBeLessThan(end);

  await page.click('#mute');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'true');
  expect(await gainsToSpeakers(page)).toEqual([0]);

  await page.click('#mute');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'false');
  expect(await gainsToSpeakers(page)).toEqual([1]);
});
