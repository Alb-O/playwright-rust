import init from './dist/background.js';
init().catch((err) => {
  console.error('[pw-ext] Failed to initialize WASM background worker:', err);
});
