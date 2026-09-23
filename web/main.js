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

/** ?seed=N fixes the game, like --seed. Anything else gets a random seed. */
function seedFromUrl(params) {
  const raw = params.get('seed');
  if (raw !== null && /^\d{1,15}$/.test(raw)) return Number(raw);
  return Math.floor(Math.random() * 2 ** 32);
}

async function main() {
  const params = new URLSearchParams(location.search);
  const seed = seedFromUrl(params);
  let muted = params.has('mute') || readStoredMute();

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
  const { start, push_key, set_muted } = wasm;

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
    if (!started || e.ctrlKey || e.metaKey || e.altKey) return;
    const c = keyToChar(e.key);
    if (c === null) return;
    e.preventDefault();
    push_key(c);
  });

  for (const button of document.querySelectorAll('#keypad button')) {
    button.addEventListener('pointerdown', (e) => {
      e.preventDefault();
      if (!started) return;
      const c = keyToChar(button.dataset.key);
      if (c !== null) push_key(c);
    });
  }
}

main();
