# pw Cookie Export Extension

A Chrome extension that exports cookies from your browser to pw-cli for authenticated automation.

## How It Works

1. Run `pw exec auth.listen --input '{}'` in your terminal - this starts a WebSocket server and displays a one-time token
2. Open the extension popup and paste the token
3. Select which domains to export (current tab's domain is auto-added)
4. Click "Export Cookies" - cookies are saved to your auth directory

## Build

```bash
nix develop . --command wasm-pack build extension/background --target web --out-dir ../dist --out-name background
```

This outputs `dist/background.js` and `dist/background_bg.wasm` used by the manifest.

## Install

1. Build the extension (see above)
2. In Chrome, go to `chrome://extensions`
3. Enable "Developer mode"
4. Click "Load unpacked" and select the `extension` directory

## Usage

### Terminal

```bash
# Start the auth listener
pw exec auth.listen --input '{}'

# Output:
# Listening for browser extension on ws://127.0.0.1:9271/
# 
# Token: abc123...
# 
# Cookies will be saved to: /path/to/playwright/auth
# 
# Press Ctrl+C to stop.
```

### Extension

1. Click the extension icon to open the popup
2. Enter the server URL (default: `ws://127.0.0.1:9271`)
3. Paste the token from the terminal
4. Click "Connect"
5. Add/remove domains as needed (current tab's domain is auto-added)
6. Click "Export Cookies"

### Using Exported Cookies

```bash
# Inspect an exported auth file
pw exec auth.show --input '{"file":"playwright/auth/github_com.json"}'

# Use an exported auth file via runtime override
cat > request.json <<'JSON'
{
  "schemaVersion": 5,
  "op": "navigate",
  "input": {
    "url": "https://github.com/settings"
  },
  "runtime": {
    "overrides": {
      "authFile": "playwright/auth/github_com.json"
    }
  }
}
JSON

pw exec --file request.json
```

## Security

- The one-time token prevents unauthorized access to your cookies
- Cookies are only exported for domains you explicitly select
- The WebSocket server only binds to localhost by default
- Tokens are not stored - you need to re-enter after disconnecting

## Files

- `manifest.json` - Extension manifest (MV3)
- `popup.html` / `popup.js` - Popup UI
- `background/src/lib.rs` - Background service worker (Rust → WASM)
- `dist/` - Built WASM output (gitignored)

## Troubleshooting

**"Failed to connect"**

* Make sure `pw exec auth.listen --input '{}'` is running
* Check the server URL matches (default port is 9271)

**"Authentication rejected"**

* Token may have been mistyped - copy it again from the terminal
* The server may have restarted - get a new token

**"No cookies found"**

* Make sure you're logged into the site in Chrome
* Some sites use different cookie domains (try adding both `example.com` and `www.example.com`)
