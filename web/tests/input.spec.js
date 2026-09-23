import { test, expect } from '@playwright/test';
import { fixture, INTRO, BORDER, startGame, canvasDiff } from './helpers.js';

// Captured from the real game after typing Alice, Bob, 1 and Enter.
const CHOICE = fixture('choice');

async function waitForIntro(page) {
  await expect.poll(() => canvasDiff(page, INTRO, BORDER)).toEqual({ diff: 0 });
}

async function typeTheCapturedAnswers(page) {
  await page.keyboard.type('Alice');
  await page.keyboard.press('Enter');
  await page.keyboard.type('Bob');
  await page.keyboard.press('Enter');
  await page.keyboard.type('1');
  await page.keyboard.press('Enter');
  await page.keyboard.press('Enter'); // the default gravity
}

/** RGB of the canvas pixels across row `y`, from column `from` to `to` inclusive. */
async function rowPixels(page, y, from, to) {
  return page.evaluate(
    ({ y, from, to }) => {
      const canvas = document.getElementById('screen');
      const data = canvas.getContext('2d').getImageData(0, y, canvas.width, 1).data;
      const pixels = [];
      for (let x = from; x <= to; x++) {
        const o = x * 4;
        pixels.push([data[o], data[o + 1], data[o + 2]]);
      }
      return pixels;
    },
    { y, from, to },
  );
}

test('typing the answers reaches the menu exactly as the original drew it', async ({ page }) => {
  await startGame(page);
  await waitForIntro(page);
  await page.keyboard.press('x'); // any key leaves the intro
  await typeTheCapturedAnswers(page);
  await expect.poll(() => canvasDiff(page, CHOICE)).toEqual({ diff: 0 });
});

test('clicking the mute button does not take the keyboard away', async ({ page }) => {
  await startGame(page); // starts muted
  await waitForIntro(page);
  await page.click('#mute');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'false');
  // Space would press the button again if it had kept focus.
  await page.keyboard.press(' ');
  await typeTheCapturedAnswers(page);
  await expect.poll(() => canvasDiff(page, CHOICE)).toEqual({ diff: 0 });
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'false');
});

test('the mute choice is remembered', async ({ page }) => {
  await page.goto('/index.html?seed=1');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'false');
  await page.click('#mute');
  await page.goto('/index.html?seed=1');
  await expect(page.locator('#mute')).toHaveAttribute('aria-pressed', 'true');
});

test('Backspace and Space stay in the game', async ({ page }) => {
  // A short viewport, so the page could scroll if Space were let through.
  await page.setViewportSize({ width: 800, height: 300 });
  await startGame(page);
  await waitForIntro(page);
  const url = page.url();
  await page.keyboard.press('x');
  await page.keyboard.press('Backspace');
  await page.keyboard.press('Backspace');
  await page.keyboard.press(' ');
  await page.keyboard.press(' ');
  expect(page.url()).toBe(url);
  expect(await page.evaluate(() => window.scrollY)).toBe(0);
});

test('starting twice runs one game, not two', async ({ page }) => {
  await page.goto('/index.html?seed=1&mute');
  await page.dblclick('#start');
  await waitForIntro(page);
  // The Rust side refuses a second start even if the page tried one.
  const second = await page.evaluate(async () => {
    const m = await import('./pkg/gorillas.js');
    try {
      m.start('screen', 1, true);
      return 'started again';
    } catch (e) {
      return String(e);
    }
  });
  expect(second).toContain('already running');
  // Two games would split the keys between them and corrupt this screen.
  await page.keyboard.press('x');
  await typeTheCapturedAnswers(page);
  await expect.poll(() => canvasDiff(page, CHOICE)).toEqual({ diff: 0 });
});

test('the screen keeps its size when the game switches to graphics', async ({ page }) => {
  await startGame(page);
  await waitForIntro(page);
  await page.keyboard.press('x');
  await typeTheCapturedAnswers(page);
  await expect.poll(() => canvasDiff(page, CHOICE)).toEqual({ diff: 0 });
  const text = await page.locator('#screen').boundingBox();
  await page.keyboard.press('p'); // play: SCREEN 9, 640 by 350
  await expect.poll(() => page.evaluate(() => document.getElementById('screen').height)).toBe(350);
  const graphics = await page.locator('#screen').boundingBox();
  expect(graphics).toEqual(text);

  // Row 2, columns 150-250 is clear of the names (top corners), the sun
  // (about x 290-350) and any building (the tallest building's roof is
  // below y 30): plain sky. That is EGA blue, palette index 0 which is
  // register 1, RGB (0, 0, 170). The fixtures every other test compares
  // against are all grey, so only a check like this one would notice a
  // red/blue channel swap.
  const sky = Array.from({ length: 101 }, () => [0, 0, 170]);
  await expect.poll(() => rowPixels(page, 2, 150, 250)).toEqual(sky);
});

test.describe('on a touch screen', () => {
  test.use({ hasTouch: true, isMobile: true, viewport: { width: 390, height: 844 } });
  test.skip(({ browserName }) => browserName === 'firefox', 'Playwright has no mobile emulation for Firefox');

  test('the keypad reaches the menu with the default names', async ({ page }) => {
    await startGame(page);
    await waitForIntro(page);
    await expect(page.locator('#keypad')).toBeVisible();
    for (const key of ['Enter', 'Enter', 'Enter', '1', 'Enter', 'Enter']) {
      await page.locator(`#keypad [data-key="${key}"]`).tap();
    }
    // The capture typed Alice and Bob; the keypad takes the default names.
    // So compare from the points prompt, row 11, downwards.
    await expect.poll(() => canvasDiff(page, CHOICE, { minRow: 11 })).toEqual({ diff: 0 });
  });
});
