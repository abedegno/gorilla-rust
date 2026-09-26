import { readFileSync } from 'node:fs';
import { test, expect } from '@playwright/test';
import { startGame } from './helpers.js';

/** Where an element is, and the size of the window it must fit in. */
async function placement(page, selector) {
  const box = await page.locator(selector).boundingBox();
  const view = await page.evaluate(() => ({ width: innerWidth, height: innerHeight }));
  return { box, view };
}

async function expectInView(page, selector) {
  const { box, view } = await placement(page, selector);
  expect(box, `${selector} is laid out`).not.toBeNull();
  expect(box.x, `${selector} left`).toBeGreaterThanOrEqual(0);
  expect(box.y, `${selector} top`).toBeGreaterThanOrEqual(0);
  expect(box.x + box.width, `${selector} right`).toBeLessThanOrEqual(view.width);
  expect(box.y + box.height, `${selector} bottom`).toBeLessThanOrEqual(view.height);
}

test('the screen is drawn with square pixels', async ({ page }) => {
  await page.goto('index.html?seed=1&mute');
  const rendering = await page.evaluate(
    () => getComputedStyle(document.getElementById('screen')).imageRendering,
  );
  expect(rendering).toBe('pixelated');
});

const phones = [
  { name: 'a phone held upright', width: 390, height: 844 },
  { name: 'a phone on its side', width: 844, height: 390 },
  { name: 'a small phone on its side', width: 568, height: 320 },
];

for (const phone of phones) {
  test.describe(`on ${phone.name} (${phone.width}x${phone.height})`, () => {
    test.use({
      hasTouch: true,
      isMobile: true,
      viewport: { width: phone.width, height: phone.height },
    });
    test.skip(({ browserName }) => browserName === 'firefox', 'Playwright has no mobile emulation for Firefox');

    test('the screen and the keypad both fit without scrolling', async ({ page }) => {
      await startGame(page);
      await expect(page.locator('#keypad')).toBeVisible();
      await expectInView(page, '#screen');
      await expectInView(page, '#keypad');
      await expectInView(page, '#mute');
    });

    test('the letters fit without scrolling too', async ({ page }) => {
      await startGame(page);
      await page.locator('#keypad').getByRole('button', { name: 'ABC', exact: true }).tap();
      await expect(page.locator('#keypad').getByRole('button', { name: 'q', exact: true })).toBeVisible();
      await expectInView(page, '#screen');
      await expectInView(page, '#keypad');
      await expectInView(page, '#mute');
    });

    test('the screen keeps its size when the game switches to graphics', async ({ page }) => {
      await startGame(page);
      // The default names, one game, the default gravity: the menu, still
      // on the 640 by 400 text screen.
      for (const key of ['Enter', 'Enter', 'Enter', '1', 'Enter', 'Enter']) {
        await page.locator(`#keypad [data-key="${key}"]:visible`).tap();
      }
      await expect.poll(() => page.evaluate(() => document.getElementById('screen').height)).toBe(400);
      const text = await page.locator('#screen').boundingBox();
      await page.locator('#keypad [data-key="P"]:visible').tap();
      await expect.poll(() => page.evaluate(() => document.getElementById('screen').height)).toBe(350);
      expect(await page.locator('#screen').boundingBox()).toEqual(text);
    });
  });
}

test('the page shows the version of the build it runs', async ({ page }) => {
  // Read from Cargo.toml, the version the wasm build is compiled with, so
  // the page is checked against what it should say rather than a copy.
  const cargo = readFileSync(new URL('../../Cargo.toml', import.meta.url), 'utf8');
  const version = cargo.match(/^version = "(.+)"$/m)[1];
  await page.goto('index.html?seed=1');
  await expect(page.locator('#version')).toHaveText(`gorilla-rust ${version}`);
  await expect(page.locator('#version')).toBeVisible();
});
