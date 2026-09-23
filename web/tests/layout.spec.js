import { test, expect } from '@playwright/test';

test('the screen is drawn with square pixels', async ({ page }) => {
  await page.goto('/index.html?seed=1&mute');
  const rendering = await page.evaluate(
    () => getComputedStyle(document.getElementById('screen')).imageRendering,
  );
  expect(rendering).toBe('pixelated');
});
