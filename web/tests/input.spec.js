import { test, expect } from '@playwright/test';
import {
  INTRO,
  CHOICE,
  BORDER,
  startGame,
  canvasDiff,
  typeTheCapturedAnswers,
} from './helpers.js';

async function waitForIntro(page) {
  await expect.poll(() => canvasDiff(page, INTRO, BORDER)).toEqual({ diff: 0 });
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
  await page.goto('/index.html?seed=1&mute');
  await expect(page.locator('#start')).toBeEnabled();
  const pageHeight = await page.evaluate(() => document.documentElement.scrollHeight);
  expect(pageHeight, 'the page is taller than the window, so it could scroll').toBeGreaterThan(300);
  // Short as the window is, the start button is wholly in view.
  const start = await page.locator('#start').boundingBox();
  expect(start.y).toBeGreaterThanOrEqual(0);
  expect(start.y + start.height).toBeLessThanOrEqual(300);
  await page.click('#start');
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
      await page.locator(`#keypad [data-key="${key}"]:visible`).tap();
    }
    // The capture typed Alice and Bob; the keypad takes the default names.
    // So compare from the points prompt, row 11, downwards.
    await expect.poll(() => canvasDiff(page, CHOICE, { minRow: 11 })).toEqual({ diff: 0 });
  });

  test('the ABC keys type the names the original was captured with', async ({ page }) => {
    // The whole menu screen, names included, must match the capture: before
    // the ABC keys, a touch screen could only reach it with the default names.
    await startGame(page);
    await waitForIntro(page);
    const key = (k) => page.locator(`#keypad [data-key="${k}"]:visible`).tap();
    const button = (name) => page.locator('#keypad').getByRole('button', { name, exact: true }).tap();
    await key('Enter'); // any key leaves the intro
    await button('ABC');
    // Shift is one-shot, as on a phone: it capitalises the next letter only.
    // The x and Backspace check Backspace on this layout too.
    await button('Shift');
    for (const k of ['a', 'l', 'i', 'c', 'x', 'Backspace', 'e', 'Enter']) await key(k);
    await button('Shift');
    for (const k of ['b', 'o', 'b', 'Enter']) await key(k);
    await button('123');
    for (const k of ['1', 'Enter', 'Enter']) await key(k);
    await expect.poll(() => canvasDiff(page, CHOICE)).toEqual({ diff: 0 });
  });

  test('every ABC key types what the keyboard would', async ({ page }) => {
    // Each letter and Space, by the label a player sees, in three runs that
    // each fit on the prompt line. A key whose data-key is wrong leaves a
    // different name on the screen from the keyboard's.
    const runs = ['qwertyuiop', 'asdfghjkl', 'zxcvbnm '];
    const snapshot = () => page.evaluate(() => document.getElementById('screen').toDataURL());
    const atThePrompt = async () => {
      await startGame(page);
      await waitForIntro(page);
      await page.keyboard.press('x');
      await expect.poll(() => canvasDiff(page, INTRO, BORDER)).not.toEqual({ diff: 0 });
      await page.waitForTimeout(200);
    };
    const clear = async (n, press) => {
      for (let i = 0; i < n; i++) await press();
      await page.waitForTimeout(500);
    };

    await atThePrompt();
    const typed = [];
    for (const run of runs) {
      await page.keyboard.type(run);
      await page.waitForTimeout(1000);
      typed.push(await snapshot());
      await clear(run.length, () => page.keyboard.press('Backspace'));
    }

    await atThePrompt();
    const keypad = page.locator('#keypad');
    await keypad.getByRole('button', { name: 'ABC', exact: true }).tap();
    for (const [i, run] of runs.entries()) {
      for (const c of run) {
        const name = c === ' ' ? 'Space' : c;
        await keypad.getByRole('button', { name, exact: true }).tap();
      }
      await expect
        .poll(async () => (await snapshot()) === typed[i], {
          message: `the ABC keys left the same text as the keyboard for "${run}"`,
        })
        .toBe(true);
      const back = keypad.getByRole('button', { name: 'Backspace', exact: true });
      await clear(run.length, () => back.tap());
    }
  });

  test('every keypad button types what the keyboard would', async ({ page }) => {
    // Each character button in turn, then 0 again and Backspace to take it
    // off, typed at the first name prompt. A button whose data-key is
    // wrong would leave a different name on the screen.
    const chars = ['7', '8', '9', '4', '5', '6', 'V', '1', '2', '3', 'P', '0', '.'];
    const snapshot = () => page.evaluate(() => document.getElementById('screen').toDataURL());
    const atThePrompt = async () => {
      await startGame(page);
      await waitForIntro(page);
      await page.keyboard.press('x'); // any key leaves the intro
      // The prompt is drawn once the intro has gone, on a 640 by 400 screen.
      await expect.poll(() => canvasDiff(page, INTRO, BORDER)).not.toEqual({ diff: 0 });
      await page.waitForTimeout(200);
    };

    // The keyboard first: the game takes one key per pass of its input
    // loop, so it is given a whole second to echo them all.
    await atThePrompt();
    const empty = await snapshot();
    await page.keyboard.type(chars.join('') + '0');
    await page.keyboard.press('Backspace');
    await page.waitForTimeout(1000);
    const typed = await snapshot();
    expect(typed === empty, 'the typed name is on the screen').toBe(false);

    await atThePrompt();
    // By the name a player sees, not by data-key, which is what is on trial.
    const keypad = page.locator('#keypad');
    for (const name of [...chars, '0', 'Backspace']) {
      await keypad.getByRole('button', { name, exact: true }).tap();
    }
    await expect
      .poll(async () => (await snapshot()) === typed, {
        message: 'the keypad left the same name on the screen as the keyboard',
      })
      .toBe(true);
  });
});
