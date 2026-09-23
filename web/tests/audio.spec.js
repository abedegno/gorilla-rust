import { test, expect } from '@playwright/test';
import {
  INTRO,
  CHOICE,
  BORDER,
  startGame,
  canvasDiff,
  typeTheCapturedAnswers,
  spyOnAudio,
} from './helpers.js';

// Nothing here can listen to the speakers, so these tests watch what the
// game asks Web Audio to do instead: every note is one oscillator, and
// every tone is meant to reach the speakers only through the master gain.

const oscillators = (page) => page.evaluate(() => window.__audio.oscillators);
const contextState = (page) => page.evaluate(() => window.__audio.contexts[0]?.state);
const suspend = (page) => page.evaluate(() => window.__audio.contexts[0].suspend());

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
  // The clock stops at the intro tune's first note, so the tune is still to
  // come when Mute is clicked however slow the machine is.
  await spyOnAudio(page, { freezeOnFirstNote: true });
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

test('a context that never starts gets only the first tune', async ({ page }) => {
  // Some browsers have not started a new context's clock by the time the
  // intro tune arrives, so that one tune is scheduled anyway and plays when
  // the clock starts. If the clock never does, nothing after it may pile up
  // behind it.
  await spyOnAudio(page, { neverStarts: true });
  await startGame(page, '?seed=1');
  await waitForIntro(page);
  // The intro tune: C D E D C D E C C.
  await expect.poll(() => oscillators(page)).toBe(9);

  await page.keyboard.press('x');
  await typeTheCapturedAnswers(page);
  await expect.poll(() => canvasDiff(page, CHOICE)).toEqual({ diff: 0 });
  // View Intro: the gorillas dance to four more tunes.
  await page.keyboard.press('v');
  await expect.poll(() => page.evaluate(() => document.getElementById('screen').height)).toBe(350);
  await page.waitForTimeout(2500);

  expect(await oscillators(page)).toBe(9);
  // Each of those tunes asked the context to resume instead.
  expect(await page.evaluate(() => window.__audio.resumes)).toBeGreaterThan(1);
});

test('a key or a tap resumes sound the browser suspended', async ({ page }) => {
  await spyOnAudio(page);
  await startGame(page, '?seed=1');
  await waitForIntro(page);
  // A context only runs where the browser has somewhere to send sound.
  // Headless Firefox on a Linux CI runner has no audio device, so its
  // context never starts and there is nothing to resume. That is skipped,
  // visibly, rather than passed; everywhere with an audio device it runs.
  const started = await expect
    .poll(() => contextState(page), { timeout: 5_000 })
    .toBe('running')
    .then(() => true, () => false);
  test.skip(!started, 'no audio output here, so the context never starts');

  await suspend(page);
  expect(await contextState(page)).toBe('suspended');
  // The name prompt that follows plays nothing, so only the key handler
  // can be what resumes the context.
  await page.keyboard.press('x');
  await expect.poll(() => contextState(page)).toBe('running');

  await suspend(page);
  expect(await contextState(page)).toBe('suspended');
  await page.mouse.click(10, 10);
  await expect.poll(() => contextState(page)).toBe('running');
});

test('tunes are dropped, not piled up, while the context is suspended', async ({ page }) => {
  await spyOnAudio(page);
  await startGame(page, '?seed=1');
  await waitForIntro(page);
  await page.keyboard.press('x');
  await typeTheCapturedAnswers(page);
  await expect.poll(() => canvasDiff(page, CHOICE)).toEqual({ diff: 0 });

  // Suspended, and kept so, as a browser that refuses to resume would.
  await suspend(page);
  await page.evaluate(() => {
    window.__audio.blockResume = true;
  });
  const before = await oscillators(page);
  const resumesBefore = await page.evaluate(() => window.__audio.resumes);

  // View Intro: the gorillas dance to four tunes, the first a second after
  // the screen changes to graphics.
  await page.keyboard.press('v');
  await expect.poll(() => page.evaluate(() => document.getElementById('screen').height)).toBe(350);
  await page.waitForTimeout(2500);

  expect(await contextState(page)).toBe('suspended');
  expect(await oscillators(page)).toBe(before);
  // The key asked once; each tune the game tried to play asked again.
  expect(await page.evaluate(() => window.__audio.resumes)).toBeGreaterThan(resumesBefore + 1);
});
