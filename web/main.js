const MUTE_KEY = 'gorillas-muted';

/** The character the game should see for a key name, or null to ignore it. */
function keyToChar(key) {
  if (key === 'Enter') return '\r';
  if (key === 'Backspace') return '\b';
  if (key.length === 1 && key >= ' ' && key <= '~') return key;
  return null;
}

function readStoredMute() {
  try {
    return localStorage.getItem(MUTE_KEY) === '1';
  } catch {
    return false;
  }
}

function storeMute(muted) {
  try {
    localStorage.setItem(MUTE_KEY, muted ? '1' : '0');
  } catch {
    // A private window or blocked storage: the choice just is not remembered.
  }
}

/**
 * Tell the browser what kind of sound the page makes. iOS treats a page's
 * Web Audio as "ambient" by default, and mutes ambient sound when the phone
 * is in Silent mode, so with sound on the game says it is "playback", as a
 * video does. Muted, it goes back to "ambient", so a silent game never
 * interrupts music another app is playing. Only Safari 17 and later have
 * navigator.audioSession; everywhere else this does nothing.
 */
function setAudioSession(muted) {
  if (navigator.audioSession) {
    navigator.audioSession.type = muted ? 'ambient' : 'playback';
  }
}

/** ?seed=N fixes the game, like --seed. Anything else gets a random seed. */
function seedFromUrl(params) {
  const raw = params.get('seed');
  if (raw !== null && /^\d{1,15}$/.test(raw)) return Number(raw);
  return Math.floor(Math.random() * 2 ** 32);
}

async function main() {
  const params = new URLSearchParams(location.search);
  const seed = seedFromUrl(params);
  // ?mute is remembered like the button, so a keyboard player, who cannot
  // reach the button, only has to use it once.
  let muted = params.has('mute') || readStoredMute();
  if (params.has('mute')) storeMute(true);

  const overlay = document.getElementById('start');
  const muteButton = document.getElementById('mute');
  const error = document.getElementById('error');

  const showError = (message) => {
    error.textContent = message;
    error.hidden = false;
  };
  const showMute = () => {
    muteButton.setAttribute('aria-pressed', String(muted));
    muteButton.textContent = muted ? 'Sound off' : 'Sound on';
    setAudioSession(muted);
  };
  showMute();

  // Imported here rather than at the top of the file, so that a missing or
  // broken module still reaches the error message instead of stopping this
  // script before it runs.
  let wasm;
  try {
    wasm = await import('./pkg/gorillas.js');
    await wasm.default();
  } catch (e) {
    overlay.textContent = 'The game could not load';
    showError(`This browser could not load the game: ${e}`);
    return;
  }
  const { start, push_key, set_muted, resume_audio, version } = wasm;

  // From the wasm build itself, so the page names the build it is running.
  document.getElementById('version').textContent = `gorilla-rust ${version()}`;
  document.querySelector('footer .version').hidden = false;

  // The overlay stays disabled, reading "Loading…", until now, so a tap
  // made before the game could start is not silently lost.
  overlay.textContent = 'Click or tap to start';
  overlay.disabled = false;

  let started = false;
  // Click, not pointerdown: some browsers only let sound start on a click.
  overlay.addEventListener('click', () => {
    if (started) return;
    started = true;
    overlay.remove();
    try {
      start('screen', seed, muted);
    } catch (e) {
      showError(String(e));
    }
  });

  // Page buttons must never hold keyboard focus, or Space and Enter meant
  // for the game would press them instead.
  muteButton.addEventListener('pointerdown', (e) => e.preventDefault());
  muteButton.addEventListener('click', () => {
    muted = !muted;
    set_muted(muted);
    storeMute(muted);
    showMute();
  });

  // On window, not the canvas, so the keyboard works whatever was clicked.
  window.addEventListener('keydown', (e) => {
    if (!started) return;
    // Any key is a user gesture, the moment a browser allows sound that it
    // suspended to resume.
    resume_audio();
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const c = keyToChar(e.key);
    if (c === null) return;
    e.preventDefault();
    push_key(c);
  });

  // A tap anywhere is a gesture too, for a player with no keyboard.
  window.addEventListener('pointerdown', () => {
    if (started) resume_audio();
  });

  // The keypad has two layouts, numbers and letters, swapped by its ABC
  // and 123 keys. Shift is one-shot, as on a phone: it capitalises the
  // next letter and then lets go.
  const main = document.querySelector('main');
  const digits = document.querySelector('#keypad .digits');
  const letters = document.querySelector('#keypad .letters');
  const shiftButton = letters.querySelector('[data-shift]');
  const letterButtons = [...letters.querySelectorAll('[data-key]')].filter((b) =>
    /^[a-z]$/.test(b.dataset.key),
  );
  let shifted = false;
  const setShift = (on) => {
    shifted = on;
    shiftButton.setAttribute('aria-pressed', String(on));
    for (const b of letterButtons) {
      b.textContent = on ? b.dataset.key.toUpperCase() : b.dataset.key;
    }
  };
  const showLetters = (on) => {
    letters.hidden = !on;
    digits.hidden = on;
    main.classList.toggle('letters', on);
  };

  for (const button of document.querySelectorAll('#keypad button')) {
    button.addEventListener('pointerdown', (e) => {
      e.preventDefault();
      if (button.dataset.switch) {
        showLetters(button.dataset.switch === 'abc');
        return;
      }
      if (button.hasAttribute('data-shift')) {
        setShift(!shifted);
        return;
      }
      if (!started) return;
      let c = keyToChar(button.dataset.key);
      if (c === null) return;
      if (shifted && /^[a-z]$/.test(c)) {
        c = c.toUpperCase();
        setShift(false);
      }
      push_key(c);
    });
  }
}

main();
