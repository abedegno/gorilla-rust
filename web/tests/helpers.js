import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));

/** A framebuffer captured from the original: one palette index per byte, 640 wide. */
export function fixture(name) {
  return [...readFileSync(path.join(here, '..', '..', 'fixtures', `${name}.bin`))];
}

export const INTRO = fixture('intro');

/** Captured from the real game after typing Alice, Bob, 1 and Enter. */
export const CHOICE = fixture('choice');

/**
 * The sparkling border animates on text rows 1 and 22 and columns 1 and 80,
 * and the intro was captured without it, so those cells are left out.
 */
export const BORDER = { excludeRows: [1, 22], excludeCols: [1, 80] };

/** Load the page and dismiss the start overlay. Muted and seeded by default. */
export async function startGame(page, query = '?seed=1&mute') {
  await page.goto('index.html' + query);
  await page.click('#start');
}

/** Answer the prompts as the capture in CHOICE did. */
export async function typeTheCapturedAnswers(page) {
  await page.keyboard.type('Alice');
  await page.keyboard.press('Enter');
  await page.keyboard.type('Bob');
  await page.keyboard.press('Enter');
  await page.keyboard.type('1');
  await page.keyboard.press('Enter');
  await page.keyboard.press('Enter'); // the default gravity
}

/**
 * Watch what the game does with Web Audio, from before the page loads.
 *
 * `window.__audio` counts the oscillators made (one per note), keeps the
 * contexts and the latest time any note was told to stop, lists the nodes
 * connected straight to a destination, and counts calls to resume. Setting
 * `window.__audio.blockResume` makes resume do nothing, as a browser that
 * refuses it outside a gesture would.
 *
 * Options, for tests that must not depend on how fast the machine is:
 * `freezeOnFirstNote` suspends the context the moment the first note is
 * made, so the tune it belongs to is certain to be still to come;
 * `neverStarts` makes the context report itself suspended with its clock
 * at zero, and ignore resume, as a context the browser never starts would,
 * while notes are still really scheduled on it.
 */
export async function spyOnAudio(page, { freezeOnFirstNote = false, neverStarts = false } = {}) {
  await page.addInitScript(({ freezeOnFirstNote, neverStarts }) => {
    const spy = {
      oscillators: 0,
      contexts: [],
      lastStop: 0,
      toDestination: [],
      resumes: 0,
      blockResume: false,
    };
    window.__audio = spy;

    const Context = window.AudioContext;
    const createOscillator = BaseAudioContext.prototype.createOscillator;
    BaseAudioContext.prototype.createOscillator = function () {
      spy.oscillators++;
      if (freezeOnFirstNote && spy.oscillators === 1) this.suspend();
      return createOscillator.call(this);
    };
    const stop = AudioScheduledSourceNode.prototype.stop;
    AudioScheduledSourceNode.prototype.stop = function (when = 0) {
      spy.lastStop = Math.max(spy.lastStop, when);
      return stop.call(this, when);
    };
    const connect = AudioNode.prototype.connect;
    AudioNode.prototype.connect = function (target, ...rest) {
      if (target instanceof AudioDestinationNode) spy.toDestination.push(this);
      return connect.call(this, target, ...rest);
    };
    const resume = Context.prototype.resume;
    Context.prototype.resume = function () {
      spy.resumes++;
      return spy.blockResume ? Promise.resolve() : resume.call(this);
    };
    window.AudioContext = class extends Context {
      constructor(...args) {
        super(...args);
        spy.contexts.push(this);
        if (neverStarts) {
          Object.defineProperty(this, 'state', { get: () => 'suspended' });
          Object.defineProperty(this, 'currentTime', { get: () => 0 });
          this.resume = () => {
            spy.resumes++;
            return Promise.resolve();
          };
        }
      }
    };
  }, { freezeOnFirstNote, neverStarts });
}

/**
 * Count the canvas pixels that differ from a fixture.
 *
 * The canvas holds RGBA. Each pixel is mapped back to a palette index
 * through the default EGA palette, which is what the text screens use, and
 * compared with the fixture. Cells are 8 pixels wide and `cellH` tall;
 * `minRow`, `excludeRows` and `excludeCols` are 1-based text cells.
 * Returns `{ diff }`, or `{ diff: -1, reason }` when the canvas is not yet
 * the fixture's size.
 */
export async function canvasDiff(page, fix, opts = {}) {
  const { cellH = 16, minRow = 1, excludeRows = [], excludeCols = [] } = opts;
  return page.evaluate(
    ({ fix, cellH, minRow, excludeRows, excludeCols }) => {
      const canvas = document.getElementById('screen');
      const height = fix.length / 640;
      if (canvas.width !== 640 || canvas.height !== height) {
        return { diff: -1, reason: `canvas is ${canvas.width}x${canvas.height}` };
      }
      const data = canvas.getContext('2d').getImageData(0, 0, 640, height).data;
      const regs = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x14, 0x07,
                    0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f];
      const rgb = (r) => [
        ((r >> 2) & 1) * 2 + ((r >> 5) & 1),
        ((r >> 1) & 1) * 2 + ((r >> 4) & 1),
        (r & 1) * 2 + ((r >> 3) & 1),
      ].map((v) => v * 85).join(',');
      const indexOf = new Map(regs.map((r, i) => [rgb(r), i]));
      let diff = 0;
      for (let y = 0; y < height; y++) {
        const row = Math.floor(y / cellH) + 1;
        if (row < minRow || excludeRows.includes(row)) continue;
        for (let x = 0; x < 640; x++) {
          if (excludeCols.includes(Math.floor(x / 8) + 1)) continue;
          const o = (y * 640 + x) * 4;
          const got = indexOf.get(`${data[o]},${data[o + 1]},${data[o + 2]}`) ?? -1;
          if (got !== fix[y * 640 + x]) diff++;
        }
      }
      return { diff };
    },
    { fix, cellH, minRow, excludeRows, excludeCols },
  );
}
