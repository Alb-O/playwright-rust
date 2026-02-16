//! Session request specification used by CLI command execution.

use std::path::Path;

use pw_rs::WaitUntil;

use crate::context::{BlockConfig, CommandContext, DownloadConfig, HarConfig};
use crate::types::BrowserKind;

/// Fully resolved request for acquiring a browser session.
pub struct SessionRequest<'a> {
	/// Navigation wait strategy used by session page operations.
	pub wait_until: WaitUntil,
	/// Whether the session should run headless.
	pub headless: bool,
	/// Optional auth file used to bootstrap storage state.
	pub auth_file: Option<&'a Path>,
	/// Browser engine to launch/connect.
	pub browser: BrowserKind,
	/// Optional CDP endpoint to attach to an existing browser.
	pub cdp_endpoint: Option<&'a str>,
	/// Whether to launch a browser server instead of direct launch.
	pub launch_server: bool,
	/// Remote debugging port for persistent Chromium sessions.
	pub remote_debugging_port: Option<u16>,
	/// Whether browser lifecycle should outlive the session handle.
	pub keep_browser_running: bool,
	/// URL patterns excluded from page-reuse selection.
	pub protected_urls: &'a [String],
	/// Preferred URL for page-reuse selection.
	pub preferred_url: Option<&'a str>,
	/// HAR recording configuration.
	pub har_config: &'a HarConfig,
	/// Request-blocking configuration.
	pub block_config: &'a BlockConfig,
	/// Download-tracking configuration.
	pub download_config: &'a DownloadConfig,
}

impl<'a> SessionRequest<'a> {
	/// Builds a request from global command context defaults.
	pub fn from_context(wait_until: WaitUntil, ctx: &'a CommandContext) -> Self {
		Self {
			wait_until,
			headless: true,
			auth_file: ctx.auth_file(),
			browser: ctx.browser,
			cdp_endpoint: ctx.cdp_endpoint(),
			launch_server: ctx.launch_server(),
			remote_debugging_port: None,
			keep_browser_running: false,
			protected_urls: &[],
			preferred_url: None,
			har_config: ctx.har_config(),
			block_config: ctx.block_config(),
			download_config: ctx.download_config(),
		}
	}

	/// Sets protected URL patterns for page-reuse filtering.
	pub fn with_protected_urls(mut self, urls: &'a [String]) -> Self {
		self.protected_urls = urls;
		self
	}

	/// Sets headless/headful mode.
	pub fn with_headless(mut self, headless: bool) -> Self {
		self.headless = headless;
		self
	}

	/// Sets the auth storage-state file.
	pub fn with_auth_file(mut self, auth_file: Option<&'a Path>) -> Self {
		self.auth_file = auth_file;
		self
	}

	/// Sets the target browser engine.
	pub fn with_browser(mut self, browser: BrowserKind) -> Self {
		self.browser = browser;
		self
	}

	/// Sets an explicit CDP endpoint for attach mode.
	pub fn with_cdp_endpoint(mut self, endpoint: Option<&'a str>) -> Self {
		self.cdp_endpoint = endpoint;
		self
	}

	/// Sets the persistent remote-debugging port.
	pub fn with_remote_debugging_port(mut self, port: Option<u16>) -> Self {
		self.remote_debugging_port = port;
		self
	}

	/// Controls whether browser shutdown is skipped on close.
	pub fn with_keep_browser_running(mut self, keep: bool) -> Self {
		self.keep_browser_running = keep;
		self
	}

	/// Sets the preferred URL used for tab/page reuse.
	pub fn with_preferred_url(mut self, url: Option<&'a str>) -> Self {
		self.preferred_url = url;
		self
	}
}
