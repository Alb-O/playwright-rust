# pw-rs Codebase Map

This document provides a focused map of the pw-rs codebase - a Rust implementation of Playwright browser automation.

## Architecture Overview

The codebase consists of two main crates:
- **pw-core** (`crates/pw-core`): Core Playwright protocol implementation
- **pw-cli** (`crates/pw-cli`): Command-line interface for browser automation

### Key Components

```
pw-cli                          pw-core
├── cli.rs (CLI args)           ├── protocol/
├── browser/session.rs          │   ├── playwright.rs (entry point)
├── context.rs                  │   ├── browser_type.rs (launch)
├── context_store.rs            │   ├── browser.rs
├── commands/                   │   ├── browser_context.rs
│   ├── navigate.rs             │   ├── page.rs (main API)
│   ├── click.rs                │   ├── frame.rs
│   ├── elements.rs             │   ├── locator.rs
│   ├── screenshot.rs           │   └── route.rs
│   └── ...                     ├── error.rs
└── types.rs                    └── assertions.rs
```

---

## pw-cli Crate

```rust
// ...
```

### Source: crates/pw-cli/src/browser/session.rs

```rust
pub struct BrowserSession {
	// ...
	_playwright: pw::Playwright,
	browser: pw::protocol::Browser,
	context: pw::protocol::BrowserContext,
	page: pw::protocol::Page,
	wait_until: pw::WaitUntil,
	ws_endpoint: Option<String>,
	launched_server: Option<pw::LaunchedServer>,
	keep_server_running: bool,
}

impl BrowserSession {
	pub async fn new(wait_until: WaitUntil) -> Result<Self> {
		Self::with_options(wait_until, None, true, BrowserKind::default(), None, false).await
	}

	// Create a session with optional auth file (convenience for commands)
	pub async fn with_auth(
		wait_until: WaitUntil,
		auth_file: Option<&Path>,
		cdp_endpoint: Option<&str>,
	) -> Result<Self> {
		Self::with_auth_and_browser(wait_until, auth_file, BrowserKind::default(), cdp_endpoint)
			.await
	}

	// Create a session with optional auth file and specific browser
	pub async fn with_auth_and_browser(
		wait_until: WaitUntil,
		auth_file: Option<&Path>,
		browser_kind: BrowserKind,
		cdp_endpoint: Option<&str>,
	) -> Result<Self> {
		match auth_file {
			Some(path) => {
				Self::with_auth_file_and_browser(wait_until, path, browser_kind, cdp_endpoint).await
			}
			None => {
				Self::with_options(wait_until, None, true, browser_kind, cdp_endpoint, false).await
			}
		}
	}

	// Create a new session with optional storage state and headless mode
	pub async fn with_options(
		wait_until: WaitUntil,
		storage_state: Option<StorageState>,
		headless: bool,
		browser_kind: BrowserKind,
		cdp_endpoint: Option<&str>,
		launch_server: bool,
	) -> Result<Self> {
		debug!(
			target = "pw",
			browser = %browser_kind,
			cdp = cdp_endpoint.is_some(),
			launch_server,
			"starting Playwright..."
		);
		let mut playwright = Playwright::launch()
			.await
			.map_err(|e| PwError::BrowserLaunch(e.to_string()))?;

		let mut ws_endpoint = None;
		let mut launched_server = None;
		let mut keep_server_running = false;

		let (browser, context) = if let Some(endpoint) = cdp_endpoint {
			if browser_kind != BrowserKind::Chromium {
				return Err(PwError::BrowserLaunch(
					"CDP endpoint connections require the chromium browser".to_string(),
				));
			}

			let connect_result = playwright
				.chromium()
				.connect_over_cdp(endpoint)
				.await
				.map_err(|e| PwError::BrowserLaunch(e.to_string()))?;

			let browser = connect_result.browser;
			let context = if let Some(state) = storage_state {
				let options = BrowserContextOptions::builder()
					.storage_state(state)
					.build();
				browser.new_context_with_options(options).await?
			} else if let Some(default_ctx) = connect_result.default_context {
				default_ctx
			} else {
				browser.new_context().await?
			};

			(browser, context)
		} else if launch_server {
			playwright.keep_server_running();
			keep_server_running = true;

			let launch_options = pw::LaunchOptions {
				headless: Some(headless),
				..Default::default()
			};

			let launched = match browser_kind {
				BrowserKind::Chromium => playwright
					.chromium()
					.launch_server_with_options(launch_options)
					.await
					.map_err(|e| PwError::BrowserLaunch(e.to_string()))?,
				BrowserKind::Firefox => playwright
					.firefox()
					.launch_server_with_options(launch_options)
					.await
					.map_err(|e| PwError::BrowserLaunch(e.to_string()))?,
				BrowserKind::Webkit => playwright
					.webkit()
					.launch_server_with_options(launch_options)
					.await
					.map_err(|e| PwError::BrowserLaunch(e.to_string()))?,
			};

			ws_endpoint = Some(launched.ws_endpoint().to_string());
			launched_server = Some(launched.clone());

			let browser = launched.browser().clone();
			let context = if let Some(state) = storage_state {
				let options = BrowserContextOptions::builder()
					.storage_state(state)
					.build();
				browser.new_context_with_options(options).await?
			} else {
				browser.new_context().await?
			};

			(browser, context)
		} else {
			let launch_options = pw::LaunchOptions {
				headless: Some(headless),
				..Default::default()
			};

			// Select browser type based on browser_kind
			let browser = match browser_kind {
				BrowserKind::Chromium => {
					playwright
						.chromium()
						.launch_with_options(launch_options)
						.await?
				}
				BrowserKind::Firefox => {
					playwright
						.firefox()
						.launch_with_options(launch_options)
						.await?
				}
				BrowserKind::Webkit => {
					playwright
						.webkit()
						.launch_with_options(launch_options)
						.await?
				}
			};

			// Create context with optional storage state
			let context = if let Some(state) = storage_state {
				let options = BrowserContextOptions::builder()
					.storage_state(state)
					.build();
				browser.new_context_with_options(options).await?
			} else {
				browser.new_context().await?
			};

			(browser, context)
		};

		let page = context.new_page().await?;

		Ok(Self {
			_playwright: playwright,
			browser,
			context,
			page,
			wait_until,
			ws_endpoint,
			launched_server,
			keep_server_running,
		})
	}

	// Create a session with auth loaded from a file
	pub async fn with_auth_file(wait_until: WaitUntil, auth_file: &Path) -> Result<Self> {
		Self::with_auth_file_and_browser(wait_until, auth_file, BrowserKind::default(), None).await
	}

	// Create a session with auth loaded from a file and specific browser
	pub async fn with_auth_file_and_browser(
		wait_until: WaitUntil,
		auth_file: &Path,
		browser_kind: BrowserKind,
		cdp_endpoint: Option<&str>,
	) -> Result<Self> {
		let storage_state = StorageState::from_file(auth_file)
			.map_err(|e| PwError::BrowserLaunch(format!("Failed to load auth file: {}", e)))?;
		Self::with_options(
			wait_until,
			Some(storage_state),
			true,
			browser_kind,
			cdp_endpoint,
			false,
		)
		.await
	}

	pub async fn launch_server_session(
		wait_until: WaitUntil,
		storage_state: Option<StorageState>,
		headless: bool,
		browser_kind: BrowserKind,
	) -> Result<Self> {
		Self::with_options(
			wait_until,
			storage_state,
			headless,
			browser_kind,
			None,
			true,
		)
		.await
	}

	pub async fn goto(&self, url: &str) -> Result<()> {
		let goto_opts = GotoOptions {
			wait_until: Some(self.wait_until),
			..Default::default()
		};

		self.page
			.goto(url, Some(goto_opts))
			.await
			.map(|_| ())
			.map_err(|e| PwError::Navigation {
				url: url.to_string(),
				source: anyhow::Error::new(e),
			})
	}

	pub fn page(&self) -> &pw::protocol::Page {
		&self.page
	}

	pub fn context(&self) -> &pw::protocol::BrowserContext {
		&self.context
	}

	pub fn ws_endpoint(&self) -> Option<&str> {
		self.ws_endpoint.as_deref()
	}

	pub async fn close(self) -> Result<()> {
		if self.launched_server.is_some() {
			// Close the context/page but keep the server running for reuse
			let _ = self.context.close().await;
			return Ok(());
		}

		self.browser.close().await?;
		Ok(())
	}

	pub async fn shutdown_server(mut self) -> Result<()> {
		if let Some(server) = self.launched_server.take() {
			server.close().await?;
			self.keep_server_running = false;
			self._playwright.enable_server_shutdown();
		} else {
			self.browser.close().await?;
		}

		Ok(())
	}
}

// ...
```

### Source: crates/pw-cli/src/cli.rs

```rust
#[derive(Debug)]
pub struct Cli {
	// Increase verbosity (-v info, -vv debug)
	pub verbose: u8,
	// Load authentication state from file (cookies, localStorage)
	pub auth: Option<std::path::PathBuf>,
	// Browser to use for automation
	pub browser: crate::types::BrowserKind,
	// Connect to an existing CDP endpoint instead of launching a browser
	pub cdp_endpoint: Option<String>,
	// Launch a reusable local browser server and persist its endpoint
	pub launch_server: bool,
	// Disable project detection (use current directory paths)
	pub no_project: bool,
	// Named context to load for this run
	pub context: Option<String>,
	// Disable contextual inference/caching for this invocation
	pub no_context: bool,
	// Do not persist command results back to context store
	pub no_save_context: bool,
	// Clear cached context data before running
	pub refresh_context: bool,
	// Base URL used when URL argument is relative or omitted
	pub base_url: Option<String>,
	pub command: Commands,
}

impl CommandFactory for Cli {
	fn command<'b>() -> clap::Command {}

	fn command_for_update<'b>() -> clap::Command {}
}

impl FromArgMatches for Cli {
	fn from_arg_matches(
		__clap_arg_matches: &clap::ArgMatches,
	) -> ::std::result::Result<Self, clap::Error> {
	}

	fn from_arg_matches_mut(
		__clap_arg_matches: &mut clap::ArgMatches,
	) -> ::std::result::Result<Self, clap::Error> {
	}

	fn update_from_arg_matches(
		&mut self,
		__clap_arg_matches: &clap::ArgMatches,
	) -> ::std::result::Result<(), clap::Error> {
	}

	fn update_from_arg_matches_mut(
		&mut self,
		__clap_arg_matches: &mut clap::ArgMatches,
	) -> ::std::result::Result<(), clap::Error> {
	}
}

impl Args for Cli {
	fn group_id() -> Option<clap::Id> {}

	fn augment_args<'b>(__clap_app: clap::Command) -> clap::Command {}

	fn augment_args_for_update<'b>(__clap_app: clap::Command) -> clap::Command {}
}

// ...
#[derive(Debug)]
pub enum Commands {}
```

### Source: crates/pw-cli/src/context.rs

Context passed to all pw-cli commands

```rust
#[derive(Debug, Clone)]
pub struct CommandContext {
	// ...
	// Detected project (if any)
	pub project: Option<crate::project::Project>,
	// Browser to use for automation
	pub browser: crate::types::BrowserKind,
	// Optional CDP endpoint for connecting to a running browser
	cdp_endpoint: Option<String>,
	// Whether to launch a reusable browser server
	launch_server: bool,
	// Auth file to use (resolved path)
	auth_file: Option<std::path::PathBuf>,
	// Whether project detection is disabled
	pub no_project: bool,
}

impl CommandContext {
	// Create a new command context
	pub fn new(
		browser: BrowserKind,
		no_project: bool,
		auth_file: Option<PathBuf>,
		cdp_endpoint: Option<String>,
		launch_server: bool,
	) -> Self {
	}

	// Get the auth file path
	pub fn auth_file(&self) -> Option<&Path> {}

	// Get the CDP endpoint URL if provided
	pub fn cdp_endpoint(&self) -> Option<&str> {}

	pub fn launch_server(&self) -> bool {}

	// Get the screenshot output path, using project paths if available
	pub fn screenshot_path(&self, output: &Path) -> PathBuf {}

	// Get a path relative to project root, or as-is if no project
	pub fn project_path(&self, path: &Path) -> PathBuf {}

	// Get the project root directory, or current directory if no project
	pub fn root(&self) -> PathBuf {}
}

// ...
```

### Source: crates/pw-cli/src/types.rs

Browser type for pw-cli commands

```rust
#[derive(
	Clone, Copy, Debug, Default, StructuralPartialEq, PartialEq, Eq, Serialize, Deserialize, Display,
)]
pub enum BrowserKind {}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NavigateResult {
	pub url: String,
	pub title: String,
	pub errors: Vec<String>,
	pub warnings: Vec<String>,
	pub has_errors: bool,
}
```

### Raw source: /home/albert/@/pw-rs/crates/pw-cli/src/commands/navigate.rs:1:53

```rust
use std::time::Duration;

use crate::context::CommandContext;
use crate::error::Result;
use crate::session_broker::{SessionBroker, SessionRequest};
use crate::types::NavigateResult;
use pw::WaitUntil;
use tracing::{info, warn};

pub async fn execute(
    url: &str,
    ctx: &CommandContext,
    broker: &mut SessionBroker<'_>,
) -> Result<()> {
    info!(target = "pw", %url, browser = %ctx.browser, "navigate");
    let session = broker
        .session(SessionRequest::from_context(WaitUntil::NetworkIdle, ctx))
        .await?;
    session.goto(url).await?;

    tokio::time::sleep(Duration::from_millis(2000)).await;

    let title = session.page().title().await.unwrap_or_default();
    let final_url = session.page().url();

    let errors_json = session
        .page()
        .evaluate_value("JSON.stringify(window.__playwrightErrors || [])")
        .await
        .unwrap_or_else(|_| "[]".to_string());

    let errors: Vec<String> = serde_json::from_str(&errors_json).unwrap_or_default();

    if !errors.is_empty() {
        warn!(
            target = "pw.browser",
            count = errors.len(),
            "page reported errors"
        );
    }

    let result = NavigateResult {
        url: final_url,
        title,
        has_errors: !errors.is_empty(),
        errors,
        warnings: vec![],
    };

    println!("{}", serde_json::to_string_pretty(&result)?);

    session.close().await
}
```


---

## pw-core Crate

The core library implements the Playwright protocol for browser automation.

### Hierarchy

1. **Playwright** - Entry point, creates browser types
2. **BrowserType** - Launches browsers (chromium, firefox, webkit)
3. **Browser** - Browser instance, creates contexts
4. **BrowserContext** - Isolated browser context, creates pages
5. **Page** - Main API for interacting with web pages
6. **Frame** - Represents frames within pages
7. **Locator** - Element selection and interaction

```rust
// ...
```

### Source: crates/pw-core/src/protocol/browser.rs

Browser represents a browser instance.

A Browser is created when you call `BrowserType::launch()`. It provides methods to create browser contexts and pages.

# Example

```rust
use pw::protocol::Playwright;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let playwright = Playwright::launch().await?;
    let chromium = playwright.chromium();

    // Launch browser and get info
    let browser = chromium.launch().await?;
    println!("Browser: {} version {}", browser.name(), browser.version());

    // Check connection status
    assert!(browser.is_connected());

    // Create and use contexts and pages
    let context = browser.new_context().await?;
    let page = context.new_page().await?;

    // Convenience: create page directly (auto-creates default context)
    let page2 = browser.new_page().await?;

    // Cleanup
    browser.close().await?;
    assert!(!browser.is_connected());
    Ok(())
}
```

See: <https://playwright.dev/docs/api/class-browser>

```rust
#[derive(Clone, Debug)]
pub struct Browser {
	// ...
	base: crate::server::channel_owner::ChannelOwnerImpl,
	version: String,
	name: String,
	is_connected: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Browser {
```

Creates a new Browser from protocol initialization

This is called by the object factory when the server sends a `__create__` message for a Browser object.

# Arguments

* `parent` - The parent BrowserType object
* `type_name` - The protocol type name ("Browser")
* `guid` - The unique identifier for this browser instance
* `initializer` - The initialization data from the server

# Errors

Returns error if initializer is missing required fields (version, name)

```rust
pub fn new(
	parent: Arc<dyn ChannelOwner>,
	type_name: String,
	guid: Arc<str>,
	initializer: Value,
) -> Result<Self> {
}
```

Returns the browser version string.

See: <https://playwright.dev/docs/api/class-browser#browser-version>

```rust
pub fn version(&self) -> &str {}
```

Returns the browser name (e.g., "chromium", "firefox", "webkit").

See: <https://playwright.dev/docs/api/class-browser#browser-name>

```rust
pub fn name(&self) -> &str {}
```

Returns true if the browser is connected.

The browser is connected when it is launched and becomes disconnected when:

- `browser.close()` is called
- The browser process crashes
- The browser is closed by the user

See: <https://playwright.dev/docs/api/class-browser#browser-is-connected>

```rust
pub fn is_connected(&self) -> bool {}

// ...
```

Creates a new browser context.

A browser context is an isolated session within the browser instance, similar to an incognito profile. Each context has its own cookies, cache, and local storage.

# Errors

Returns error if:

- Browser has been closed
- Communication with browser process fails

See: <https://playwright.dev/docs/api/class-browser#browser-new-context>

```rust
pub async fn new_context(&self) -> Result<BrowserContext> {}
```

Creates a new browser context with custom options.

A browser context is an isolated session within the browser instance, similar to an incognito profile. Each context has its own cookies, cache, and local storage.

This method allows customizing viewport, user agent, locale, timezone, and other settings.

# Errors

Returns error if:

- Browser has been closed
- Communication with browser process fails
- Invalid options provided

See: <https://playwright.dev/docs/api/class-browser#browser-new-context>

```rust
pub async fn new_context_with_options(
	&self,
	options: crate::protocol::BrowserContextOptions,
) -> Result<BrowserContext> {
}
```

Creates a new page in a new browser context.

This is a convenience method that creates a default context and then creates a page in it. This is equivalent to calling `browser.new_context().await?.new_page().await?`.

The created context is not directly accessible, but will be cleaned up when the page is closed.

# Errors

Returns error if:

- Browser has been closed
- Communication with browser process fails

See: <https://playwright.dev/docs/api/class-browser#browser-new-page>

```rust
pub async fn new_page(&self) -> Result<Page> {}
```

Closes the browser and all of its pages (if any were opened).

This is a graceful operation that sends a close command to the browser and waits for it to shut down properly.

# Errors

Returns error if:

- Browser has already been closed
- Communication with browser process fails

See: <https://playwright.dev/docs/api/class-browser#browser-close>

```rust
	pub async fn close(&self) -> Result<()> {}
}

// ...
```

### Source: crates/pw-core/src/protocol/browser_type.rs

BrowserType represents a browser engine (Chromium, Firefox, or WebKit).

Each Playwright instance provides three BrowserType objects accessible via:

- `playwright.chromium()`
- `playwright.firefox()`
- `playwright.webkit()`

# Example

```rust
let playwright = Playwright::launch().await?;
let chromium = playwright.chromium();

// Verify browser type info
assert_eq!(chromium.name(), "chromium");
assert!(!chromium.executable_path().is_empty());

// Launch with default options
let browser1 = chromium.launch().await?;
assert_eq!(browser1.name(), "chromium");
assert!(!browser1.version().is_empty());
browser1.close().await?;

// Launch with custom options
let options = LaunchOptions::default()
    .headless(true)
    .slow_mo(100.0)
    .args(vec!["--no-sandbox".to_string()]);

let browser2 = chromium.launch_with_options(options).await?;
assert_eq!(browser2.name(), "chromium");
assert!(!browser2.version().is_empty());
browser2.close().await?;
```

See: <https://playwright.dev/docs/api/class-browsertype>

```rust
#[derive(Debug)]
pub struct BrowserType {
	// ...
	// Base ChannelOwner implementation
	base: crate::server::channel_owner::ChannelOwnerImpl,
	// Browser name ("chromium", "firefox", or "webkit")
	name: String,
	// Path to browser executable
	executable_path: String,
}

impl BrowserType {
```

Creates a new BrowserType object from protocol initialization.

Called by the object factory when server sends __create__ message.

# Arguments

* `parent` - Parent Playwright object
* `type_name` - Protocol type name ("BrowserType")
* `guid` - Unique GUID from server (e.g., "browserType@chromium")
* `initializer` - Initial state with name and executablePath

```rust
pub fn new(
	parent: Arc<dyn ChannelOwner>,
	type_name: String,
	guid: Arc<str>,
	initializer: Value,
) -> Result<Self> {
}

// Returns the browser name ("chromium", "firefox", or "webkit").
pub fn name(&self) -> &str {}

// Returns the path to the browser executable.
pub fn executable_path(&self) -> &str {}
```

Launches a browser instance with default options.

This is equivalent to calling `launch_with_options(LaunchOptions::default())`.

# Errors

Returns error if:

- Browser executable not found
- Launch timeout (default 30s)
- Browser process fails to start

See: <https://playwright.dev/docs/api/class-browsertype#browser-type-launch>

```rust
pub async fn launch(&self) -> Result<Browser> {}
```

Launches a browser instance with custom options.

# Arguments

* `options` - Launch options (headless, args, etc.)

# Errors

Returns error if:

- Browser executable not found
- Launch timeout
- Invalid options
- Browser process fails to start

See: <https://playwright.dev/docs/api/class-browsertype#browser-type-launch>

```rust
pub async fn launch_with_options(&self, options: LaunchOptions) -> Result<Browser> {}

// Launches a browser server and returns its websocket endpoint.
pub async fn launch_server(&self) -> Result<LaunchedServer> {}

// Launches a browser server with custom options and returns a handle.
pub async fn launch_server_with_options(
	&self,
	options: LaunchOptions,
) -> Result<LaunchedServer> {
}
```

Connects to an existing browser over the Chrome DevTools Protocol.

This keeps the standard Playwright driver in the loop while reusing a running GUI browser (for example, the extension relay). The returned default context, when present, represents the persistent browser profile.

```rust
	pub async fn connect_over_cdp(
		&self,
		endpoint_url: impl Into<String>,
	) -> Result<ConnectOverCDPResult> {
	}
}
```

### Source: crates/pw-core/src/protocol/playwright.rs

Playwright is the root object that provides access to browser types.

This is the main entry point for the Playwright API. It provides access to the three browser types (Chromium, Firefox, WebKit) and other top-level services.

# Example

```rust
use pw::protocol::Playwright;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Launch Playwright server and initialize
    let playwright = Playwright::launch().await?;

    // Verify all three browser types are available
    let chromium = playwright.chromium();
    let firefox = playwright.firefox();
    let webkit = playwright.webkit();

    assert_eq!(chromium.name(), "chromium");
    assert_eq!(firefox.name(), "firefox");
    assert_eq!(webkit.name(), "webkit");

    // Verify we can launch a browser
    let browser = chromium.launch().await?;
    assert!(!browser.version().is_empty());
    browser.close().await?;

    // Shutdown when done
    playwright.shutdown().await?;

    Ok(())
}
```

See: <https://playwright.dev/docs/api/class-playwright>

```rust
#[derive(Debug)]
pub struct Playwright {
	// ...
	// Base ChannelOwner implementation
	base: crate::server::channel_owner::ChannelOwnerImpl,
	// Chromium browser type (stored as `Arc<dyn ChannelOwner>`, downcast on access)
	chromium: std::sync::Arc<dyn ChannelOwner>,
	// Firefox browser type (stored as `Arc<dyn ChannelOwner>`, downcast on access)
	firefox: std::sync::Arc<dyn ChannelOwner>,
	// WebKit browser type (stored as `Arc<dyn ChannelOwner>`, downcast on access)
	webkit: std::sync::Arc<dyn ChannelOwner>,
```

Playwright server process (for clean shutdown)

Stored as `Option<PlaywrightServer>` wrapped in Arc<Mutex<>> to allow:

- Sharing across clones (Arc)
- Taking ownership during shutdown (Option::take)
- Interior mutability (Mutex)

```rust
	server: std::sync::Arc<
		parking_lot::Mutex<Option<crate::server::playwright_server::PlaywrightServer>>,
	>,
	// Whether to keep the launched server running when Playwright is dropped
	keep_server_running: bool,
}

impl Playwright {
```

Launches Playwright and returns a handle to interact with browser types.

This is the main entry point for the Playwright API. It will:

1. Launch the Playwright server process
2. Establish a connection via stdio
3. Initialize the protocol
4. Return a Playwright instance with access to browser types

# Errors

Returns error if:

- Playwright server is not found or fails to launch
- Connection to server fails
- Protocol initialization fails
- Server doesn't respond within timeout (30s)

```rust
pub async fn launch() -> Result<Self> {}

// Connect to a running Playwright driver over WebSocket.
pub async fn connect_ws(ws_url: &str) -> Result<Self> {}
```

Creates a new Playwright object from protocol initialization.

Called by the object factory when server sends __create__ message for root object.

# Arguments

* `connection` - The connection (Playwright is root, so no parent)
* `type_name` - Protocol type name ("Playwright")
* `guid` - Unique GUID from server (typically "playwright@1")
* `initializer` - Initial state with references to browser types

# Initializer Format

The initializer contains GUID references to BrowserType objects:

```json
{
  "chromium": { "guid": "browserType@chromium" },
  "firefox": { "guid": "browserType@firefox" },
  "webkit": { "guid": "browserType@webkit" }
}
```

```rust
pub async fn new(
	connection: Arc<dyn ConnectionLike>,
	type_name: String,
	guid: Arc<str>,
	initializer: Value,
) -> Result<Self> {
}

// Returns the Chromium browser type.
pub fn chromium(&self) -> &BrowserType {}

// Returns the Firefox browser type.
pub fn firefox(&self) -> &BrowserType {}

// Returns the WebKit browser type.
pub fn webkit(&self) -> &BrowserType {}

// Allow the launched Playwright server to keep running after this handle is dropped.
pub fn keep_server_running(&mut self) {}

// Re-enable automatic server shutdown on drop (default behavior).
pub fn enable_server_shutdown(&mut self) {}
```

Shuts down the Playwright server gracefully.

This method should be called when you're done using Playwright to ensure the server process is terminated cleanly, especially on Windows.

# Platform-Specific Behavior

**Windows**: Closes stdio pipes before shutting down to prevent hangs.

**Unix**: Standard graceful shutdown.

# Errors

Returns an error if the server shutdown fails.

```rust
	pub async fn shutdown(&self) -> Result<()> {}
}

impl Drop for Playwright {
```

Ensures Playwright server is shut down when Playwright is dropped.

This is critical on Windows to prevent process hangs when tests complete. The Drop implementation will attempt to kill the server process synchronously.

Note: For graceful shutdown, prefer calling `playwright.shutdown().await` explicitly before dropping.

```rust
	fn drop(&mut self) {}
}
```

### Raw source: /home/albert/@/pw-rs/crates/pw-core/src/protocol/page.rs:126:180

```rust
pub struct Page {
    base: ChannelOwnerImpl,
    /// Current URL of the page
    /// Wrapped in RwLock to allow updates from events
    url: Arc<RwLock<String>>,
    /// GUID of the main frame
    main_frame_guid: Arc<str>,
    /// Route handlers for network interception
    route_handlers: Arc<Mutex<Vec<RouteHandlerEntry>>>,
    /// Download event handlers
    download_handlers: Arc<Mutex<Vec<DownloadHandler>>>,
    /// Dialog event handlers
    dialog_handlers: Arc<Mutex<Vec<DialogHandler>>>,
}

/// Type alias for boxed route handler future
type RouteHandlerFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

/// Type alias for boxed download handler future
type DownloadHandlerFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

/// Type alias for boxed dialog handler future
type DialogHandlerFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

/// Storage for a single route handler
#[derive(Clone)]
struct RouteHandlerEntry {
    pattern: String,
    handler: Arc<dyn Fn(Route) -> RouteHandlerFuture + Send + Sync>,
}

/// Download event handler
type DownloadHandler = Arc<dyn Fn(Download) -> DownloadHandlerFuture + Send + Sync>;

/// Dialog event handler
type DialogHandler = Arc<dyn Fn(Dialog) -> DialogHandlerFuture + Send + Sync>;

impl Page {
    /// Creates a new Page from protocol initialization
    ///
    /// This is called by the object factory when the server sends a `__create__` message
    /// for a Page object.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent BrowserContext object
    /// * `type_name` - The protocol type name ("Page")
    /// * `guid` - The unique identifier for this page
    /// * `initializer` - The initialization data from the server
    ///
    /// # Errors
    ///
    /// Returns error if initializer is malformed
    pub fn new(
        parent: Arc<dyn ChannelOwner>,
```

### Raw source: /home/albert/@/pw-rs/crates/pw-core/src/protocol/locator.rs:98:200

```rust
pub struct Locator {
    frame: Arc<Frame>,
    selector: String,
}

impl Locator {
    /// Creates a new Locator (internal use only)
    ///
    /// Use `page.locator()` or `frame.locator()` to create locators in application code.
    pub(crate) fn new(frame: Arc<Frame>, selector: String) -> Self {
        Self { frame, selector }
    }

    /// Returns the selector string for this locator
    pub fn selector(&self) -> &str {
        &self.selector
    }

    /// Creates a locator for the first matching element.
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-first>
    pub fn first(&self) -> Locator {
        Locator::new(
            Arc::clone(&self.frame),
            format!("{} >> nth=0", self.selector),
        )
    }

    /// Creates a locator for the last matching element.
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-last>
    pub fn last(&self) -> Locator {
        Locator::new(
            Arc::clone(&self.frame),
            format!("{} >> nth=-1", self.selector),
        )
    }

    /// Creates a locator for the nth matching element (0-indexed).
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-nth>
    pub fn nth(&self, index: i32) -> Locator {
        Locator::new(
            Arc::clone(&self.frame),
            format!("{} >> nth={}", self.selector, index),
        )
    }

    /// Creates a sub-locator within this locator's subtree.
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-locator>
    pub fn locator(&self, selector: &str) -> Locator {
        Locator::new(
            Arc::clone(&self.frame),
            format!("{} >> {}", self.selector, selector),
        )
    }

    /// Returns the number of elements matching this locator.
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-count>
    pub async fn count(&self) -> Result<usize> {
        self.frame.locator_count(&self.selector).await
    }

    /// Returns the text content of the element.
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-text-content>
    pub async fn text_content(&self) -> Result<Option<String>> {
        self.frame.locator_text_content(&self.selector).await
    }

    /// Returns the inner text of the element (visible text).
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-inner-text>
    pub async fn inner_text(&self) -> Result<String> {
        self.frame.locator_inner_text(&self.selector).await
    }

    /// Returns the inner HTML of the element.
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-inner-html>
    pub async fn inner_html(&self) -> Result<String> {
        self.frame.locator_inner_html(&self.selector).await
    }

    /// Returns the value of the specified attribute.
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-get-attribute>
    pub async fn get_attribute(&self, name: &str) -> Result<Option<String>> {
        self.frame.locator_get_attribute(&self.selector, name).await
    }

    /// Returns whether the element is visible.
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-is-visible>
    pub async fn is_visible(&self) -> Result<bool> {
        self.frame.locator_is_visible(&self.selector).await
    }

    /// Returns whether the element is enabled.
    ///
    /// See: <https://playwright.dev/docs/api/class-locator#locator-is-enabled>
```

