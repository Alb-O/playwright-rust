# pw-cli init: Opinionated Playwright Project Structure

This document describes the `pw init` command added to pw-cli, the directory structure it creates, and the current state of integration with Playwright's tooling and Nix.

## What pw-cli init does

The command scaffolds a centralized `playwright/` directory structure based on patterns from the markitime project. Running `pw init` in an empty directory creates:

```
project-root/
├── playwright.config.js
└── playwright/
    ├── tests/
    │   └── example.spec.js
    ├── scripts/
    │   └── common.sh
    ├── results/
    ├── reports/
    ├── screenshots/
    ├── auth/
    └── .gitignore
```

The `playwright.config.js` points all artifact paths into the `playwright/` subtree: test results go to `playwright/results`, HTML and JSON reports to `playwright/reports`, and the test directory itself is `playwright/tests`. This keeps test infrastructure contained rather than scattered across the project root.

The `--template minimal` flag skips creating `scripts/`, `results/`, `reports/`, `screenshots/`, and `auth/`, leaving only the `tests/` directory. The `--typescript` flag generates `.ts` files instead of `.js`. Both `--no-config` and `--no-example` suppress their respective outputs.

## The generated configuration

The config template sets reasonable defaults: parallel execution locally, single-worker on CI, screenshots on failure, video retained on failure. It configures three reporter formats (HTML, JSON, JUnit) all writing to `playwright/reports/`. The `baseURL` reads from environment variables with sensible fallbacks.

```javascript
export default defineConfig({
  testDir: "playwright/tests",
  outputDir: "playwright/results",
  reporter: [
    ["html", { outputFolder: "playwright/reports/html-report", open: "never" }],
    ["json", { outputFile: "playwright/reports/test-results.json" }],
    ["junit", { outputFile: "playwright/reports/test-results.xml" }],
  ],
  // ...
});
```

The `.gitignore` excludes `results/`, `reports/`, `screenshots/`, and auth state files. The `scripts/common.sh` provides shell utilities for finding the project root and colored logging, matching the pattern from markitime.

## Nix integration

The pw-rs flake provides browser binaries via Nix's `playwright-driver.browsers` package and handles version compatibility automatically. The devshell sets up `PLAYWRIGHT_BROWSERS_PATH` pointing to a local `.playwright-browsers/` directory with symlinks to Nix-provided browsers.

### The version mismatch problem (solved)

Nix's `playwright-driver.browsers` pins to a specific Chromium revision (1181 at time of writing). Different Playwright versions expect different revisions:

- playwright-rs 1.56.1 expects revision 1194
- `@playwright/test` 1.57 expects revision 1200

Additionally, Playwright 1.57+ changed the internal directory structure for headless shell:
- Old (1181): `chromium_headless_shell-1181/chrome-linux/headless_shell`
- New (1200): `chromium_headless_shell-1200/chrome-headless-shell-linux64/chrome-headless-shell`

Simple symlinks (`chromium-1200 -> chromium-1181`) work for the main chromium directory, but the headless shell requires a nested directory structure with the binary symlinked to the correct path.

### The solution

The flake.nix shellHook creates compatibility symlinks:

```bash
# Simple symlinks for chromium
ln -sf "$BROWSERS_BASE/chromium-1181" "$BROWSERS_COMPAT/chromium-1194"
ln -sf "$BROWSERS_BASE/chromium-1181" "$BROWSERS_COMPAT/chromium-1200"

# Nested structure for headless shell (1200 has different internal layout)
mkdir -p "$BROWSERS_COMPAT/chromium_headless_shell-1200/chrome-headless-shell-linux64"
ln -sf "$BROWSERS_BASE/chromium_headless_shell-1181/chrome-linux/headless_shell" \
       "$BROWSERS_COMPAT/chromium_headless_shell-1200/chrome-headless-shell-linux64/chrome-headless-shell"
```

This allows `@playwright/test` 1.57 to find the browser binary at the expected path despite using Nix's older browser revision.

## Running tests

After running `pw init`, users have two options for running tests:

### Recommended: Pure Nix (no npm)

The simplest approach uses nixpkgs' `playwright-test` package, which is already version-aligned with the Nix-provided browsers:

```bash
nix shell nixpkgs#playwright-test nixpkgs#playwright-driver.browsers \
  -c playwright test
```

This requires no setup scripts, no symlinks, and no npm. The versions match automatically.

### Alternative: npm with setup script

If you need a specific npm version of `@playwright/test` (e.g., for features not yet in nixpkgs), use the `--nix` flag when initializing:

```bash
pw init --nix
```

This generates `playwright/scripts/setup-browsers.sh` which creates version compatibility symlinks:

```bash
eval "$(playwright/scripts/setup-browsers.sh)"
npm install @playwright/test
npx playwright test
```

The script detects Nix-provided browsers and creates symlinks for the revisions that different Playwright versions expect.

## Why version compatibility matters

Nix's `playwright-driver.browsers` pins to a specific Chromium revision (1181 at time of writing). Different Playwright versions expect different revisions:

| Playwright Version | Expected Browser Revision |
|-------------------|---------------------------|
| nixpkgs playwright-test 1.54.1 | 1181 (matches Nix browsers) |
| pw-core (playwright-rs 1.56.1) | 1194 |
| npm @playwright/test 1.57+ | 1200+ |

Additionally, Playwright 1.57+ changed the internal directory structure for headless shell:
- Old (1181): `chromium_headless_shell-1181/chrome-linux/headless_shell`
- New (1200): `chromium_headless_shell-1200/chrome-headless-shell-linux64/chrome-headless-shell`

The pw-rs `flake.nix` shellHook and the generated `setup-browsers.sh` script handle these differences by creating appropriate symlinks.

## Areas for future work

**Project-aware commands.** Currently pw-cli commands are stateless; they don't read `playwright.config.js` or respect the project structure. A `pw screenshot` command writes to the path you specify, not to `playwright/screenshots/`. Adding project awareness would require:

- Detecting `playwright.config.js` in parent directories
- Parsing the config (which is JavaScript, so either shell out to Node or maintain a subset parser)
- Using the configured paths as defaults

**pw-cli's own browser usage.** The pw-cli commands (`navigate`, `screenshot`, `console`, etc.) use pw-core, which downloads Playwright driver 1.56.1 expecting browser revision 1194. The pw-rs `flake.nix` handles this with symlinks. Users outside the pw-rs devshell would need similar symlinks or to use the setup-browsers.sh approach.

## Files changed

The implementation added:

- `crates/pw-cli/src/commands/init/mod.rs`: Scaffold logic and 9 unit tests
- `crates/pw-cli/src/commands/init/templates.rs`: Embedded file templates including `setup-browsers.sh`
- `crates/pw-cli/src/cli.rs`: `Init` command variant, `InitTemplate` enum, and `--nix` flag
- `crates/pw-cli/src/commands/mod.rs`: Dispatch for the new command
- `crates/pw-cli/src/error.rs`: `Init` error variant
- `crates/pw-cli/Cargo.toml`: `tempfile` dev dependency for tests
- `flake.nix`: Browser version compatibility symlinks for pw-core
