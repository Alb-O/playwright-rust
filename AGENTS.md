# AI Agent Integration Guide

This document describes how AI coding agents can use `pw-cli` for browser automation tasks.

## Quick Start

```bash
# Start the daemon for persistent sessions (recommended)
pw daemon start

# Navigate and extract content
pw navigate https://example.com
pw text -s "h1"                    # get heading text
pw html -s "main"                  # get HTML content
pw screenshot -o page.png          # capture screenshot

# When done
pw daemon stop
```

## Why Use the Daemon?

Without the daemon, each `pw` command:
1. Spawns a new Playwright driver (~200ms)
2. Launches a new browser (~300ms)
3. Executes the command
4. Tears everything down

With the daemon running:
1. Command connects to existing daemon via Unix socket (~5ms)
2. Reuses existing browser instance
3. Executes immediately

For agents making multiple browser calls, the daemon reduces latency from ~500ms to ~50ms per command.

## Common Patterns

### Extract page content

```bash
pw text https://example.com -s "article"           # text content
pw html https://example.com -s "article"           # HTML content
pw eval https://example.com "document.title"       # run JavaScript
```

### Navigate and interact

```bash
pw navigate https://example.com
pw click -s "button.accept"        # click element (uses cached URL)
pw text -s ".result"               # read result
```

### Screenshots for visual verification

```bash
pw screenshot https://example.com -o before.png
pw click -s "button.toggle"
pw screenshot -o after.png
```

### Wait for dynamic content

```bash
pw navigate https://spa-app.com
pw wait -s ".loaded-content"       # wait for selector
pw text -s ".loaded-content"
```

### Authenticated sessions

```bash
# One-time: open browser and log in manually
pw auth login https://app.example.com -o auth.json

# Subsequent commands use saved session
pw --auth auth.json navigate https://app.example.com/dashboard
pw --auth auth.json text -s ".user-name"
```

## Output Format

All commands output JSON to stdout:

```json
{
  "ok": true,
  "command": "text",
  "inputs": {"url": "https://example.com", "selector": "h1"},
  "data": {"text": "Example Domain", "selector": "h1", "matchCount": 1},
  "timings": {"durationMs": 0}
}
```

Errors include structured error info:

```json
{
  "ok": false,
  "command": "text",
  "error": {"code": "ELEMENT_NOT_FOUND", "message": "No elements match selector: .missing"}
}
```

## Context Caching

The CLI caches `last_url`, `last_selector`, and `last_output` between invocations. This enables conversational workflows:

```bash
pw navigate https://example.com    # caches URL
pw text -s h1                      # uses cached URL, caches selector
pw text                            # uses cached URL and selector
pw screenshot -o page.png          # uses cached URL
```

Disable caching with `--no-context` for isolated commands.

## Daemon Management

```bash
pw daemon start              # start background daemon
pw daemon start --foreground # run in foreground (for debugging)
pw daemon status             # show running browsers
pw daemon stop               # graceful shutdown
```

The daemon spawns browsers on ports 9222-10221. Currently only Chromium is supported for daemon-managed browsers.

## Flags Reference

| Flag | Description |
|------|-------------|
| `--no-daemon` | Don't use daemon even if running |
| `--no-context` | Don't read/write context cache |
| `--auth <file>` | Use saved authentication state |
| `--headful` | Run browser with visible window |
| `--browser <kind>` | chromium (default), firefox, webkit |
| `-v` / `-vv` | Verbose / debug output |

## Best Practices for Agents

1. **Start daemon at session begin**: Run `pw daemon start` once, then make many commands
2. **Use context caching**: Let URLs and selectors carry over between related commands
3. **Parse JSON output**: All commands return structured JSON for reliable parsing
4. **Handle errors gracefully**: Check `ok` field before accessing `data`
5. **Stop daemon when done**: Run `pw daemon stop` to clean up browser processes
