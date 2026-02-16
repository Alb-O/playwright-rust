//! Session orchestration for browser acquisition and lifecycle.

use std::path::Path;

use pw_rs::WaitUntil;
use serde_json::json;

use super::daemon_lease::acquire_daemon_lease;
use super::descriptor::SessionDescriptor;
use super::descriptor_lifecycle::DescriptorLifecycle;
use super::outcome::SessionHandle;
use super::repository::SessionRepository;
use super::session_factory::SessionFactory;
use super::spec::SessionRequest;
use super::strategy::{SessionStrategyInput, resolve_session_strategy};
use crate::context::CommandContext;
use crate::error::Result;

/// Session manager that applies strategy selection and orchestrates acquisition.
pub struct SessionManager<'a> {
	ctx: &'a CommandContext,
	repository: SessionRepository,
	namespace_id: Option<String>,
	refresh: bool,
}

impl<'a> SessionManager<'a> {
	/// Creates a manager for the current command execution scope.
	pub fn new(ctx: &'a CommandContext, descriptor_path: Option<std::path::PathBuf>, namespace_id: Option<String>, refresh: bool) -> Self {
		Self {
			ctx,
			repository: SessionRepository::new(descriptor_path),
			namespace_id,
			refresh,
		}
	}

	/// Returns immutable command context used by this manager.
	pub fn context(&self) -> &'a CommandContext {
		self.ctx
	}

	/// Returns descriptor path when persistence is enabled.
	pub fn descriptor_path(&self) -> Option<&Path> {
		self.repository.path()
	}

	/// Loads descriptor metadata from persistence.
	pub fn load_descriptor(&self) -> Result<Option<SessionDescriptor>> {
		self.descriptors().load()
	}

	/// Clears descriptor metadata from persistence.
	pub fn clear_descriptor(&self) -> Result<bool> {
		self.descriptors().clear()
	}

	/// Returns the structured payload used by `session.status`.
	pub fn descriptor_status(&self) -> Result<serde_json::Value> {
		self.descriptors().status_payload()
	}

	/// Removes descriptor metadata and returns the structured payload for `session.clear`.
	pub fn clear_descriptor_response(&self) -> Result<serde_json::Value> {
		self.descriptors().clear_payload()
	}

	/// Stops an active descriptor-backed browser session.
	pub async fn stop_descriptor_session(&mut self) -> Result<serde_json::Value> {
		let Some(path) = self.descriptor_path().map(Path::to_path_buf) else {
			return Ok(json!({
				"stopped": false,
				"message": "No active namespace; nothing to stop"
			}));
		};

		let Some(descriptor) = self.load_descriptor()? else {
			return Ok(json!({
				"stopped": false,
				"message": "No session descriptor for namespace; nothing to stop"
			}));
		};

		let endpoint = descriptor.cdp_endpoint.as_deref().or(descriptor.ws_endpoint.as_deref());
		let Some(endpoint) = endpoint else {
			let _ = self.clear_descriptor()?;
			return Ok(json!({
				"stopped": false,
				"path": path,
				"message": "Descriptor missing endpoint; removed descriptor"
			}));
		};

		let mut request = SessionRequest::from_context(WaitUntil::NetworkIdle, self.context());
		request.browser = descriptor.browser;
		request.headless = descriptor.headless;
		request.cdp_endpoint = Some(endpoint);
		request.launch_server = false;

		let session = self.session(request).await?;
		session.browser().close().await?;
		let _ = self.clear_descriptor()?;

		Ok(json!({
			"stopped": true,
			"path": path,
		}))
	}

	/// Acquires a session using descriptor reuse, daemon leasing, or launch flows.
	pub async fn session(&mut self, request: SessionRequest<'_>) -> Result<SessionHandle> {
		let storage_state = request.auth_file.map(SessionFactory::load_storage_state).transpose()?;
		let strategy = resolve_session_strategy(SessionStrategyInput {
			has_descriptor_path: self.descriptor_path().is_some(),
			refresh: self.refresh,
			no_daemon: self.ctx.no_daemon(),
			browser: request.browser,
			cdp_endpoint: request.cdp_endpoint,
			remote_debugging_port: request.remote_debugging_port,
			launch_server: request.launch_server,
		});

		if self.refresh {
			let _ = self.clear_descriptor();
		} else if strategy.try_descriptor_reuse {
			if let Some(descriptor) = self.load_descriptor()? {
				let factory = SessionFactory::new(self.ctx);
				if let Some(handle) = factory.acquire_from_descriptor(&descriptor, &request, storage_state.clone()).await? {
					return Ok(handle);
				}
			}
		}

		let daemon_lease = acquire_daemon_lease(self.namespace_id.as_deref(), &request, strategy.try_daemon_lease).await?;
		let factory = SessionFactory::new(self.ctx);
		let (mut session, source) = factory
			.acquire_primary(&request, strategy.primary, storage_state, daemon_lease.as_ref())
			.await?;

		factory.auto_inject_auth_if_needed(&request, daemon_lease.as_ref(), &mut session).await?;
		self.descriptors().persist_for_session(&request, &session, daemon_lease.as_ref());

		Ok(SessionHandle { session, source })
	}

	fn descriptors(&self) -> DescriptorLifecycle<'_> {
		DescriptorLifecycle::new(self.ctx, &self.repository)
	}
}

#[cfg(test)]
mod tests {
	use pw_rs::WaitUntil;

	use super::*;
	use crate::context::{BlockConfig, DownloadConfig, HarConfig};
	use crate::types::BrowserKind;

	static DEFAULT_HAR_CONFIG: HarConfig = HarConfig {
		path: None,
		content_policy: None,
		mode: None,
		omit_content: false,
		url_filter: None,
	};

	static DEFAULT_BLOCK_CONFIG: BlockConfig = BlockConfig { patterns: Vec::new() };
	static DEFAULT_DOWNLOAD_CONFIG: DownloadConfig = DownloadConfig { dir: None };

	#[test]
	fn session_request_builders_round_trip() {
		let ctx = CommandContext::new(BrowserKind::Chromium, false, None, None, false, false);
		let request = SessionRequest::from_context(WaitUntil::NetworkIdle, &ctx)
			.with_headless(false)
			.with_browser(BrowserKind::Chromium)
			.with_auth_file(None)
			.with_cdp_endpoint(Some("http://127.0.0.1:9222"))
			.with_remote_debugging_port(Some(9555))
			.with_keep_browser_running(true)
			.with_preferred_url(Some("https://example.com"))
			.with_protected_urls(&[]);
		assert!(!request.headless);
		assert_eq!(request.cdp_endpoint, Some("http://127.0.0.1:9222"));
		assert_eq!(request.remote_debugging_port, Some(9555));
		assert!(request.keep_browser_running);
		assert_eq!(request.preferred_url, Some("https://example.com"));
	}

	#[test]
	fn default_configs_are_accessible() {
		let request = SessionRequest {
			wait_until: WaitUntil::NetworkIdle,
			headless: true,
			auth_file: None,
			browser: BrowserKind::Chromium,
			cdp_endpoint: None,
			launch_server: false,
			remote_debugging_port: None,
			keep_browser_running: false,
			protected_urls: &[],
			preferred_url: None,
			har_config: &DEFAULT_HAR_CONFIG,
			block_config: &DEFAULT_BLOCK_CONFIG,
			download_config: &DEFAULT_DOWNLOAD_CONFIG,
		};
		assert_eq!(request.block_config.patterns.len(), 0);
		assert!(request.download_config.dir.is_none());
	}

	#[test]
	fn descriptor_status_without_path_reports_inactive() {
		let ctx = CommandContext::new(BrowserKind::Chromium, true, None, None, false, false);
		let manager = SessionManager::new(&ctx, None, None, false);
		let status = manager.descriptor_status().unwrap();
		assert_eq!(status["active"], false);
		assert_eq!(status["message"], "No active namespace; session status unavailable");
	}

	#[test]
	fn clear_descriptor_without_path_reports_noop() {
		let ctx = CommandContext::new(BrowserKind::Chromium, true, None, None, false, false);
		let manager = SessionManager::new(&ctx, None, None, false);
		let status = manager.clear_descriptor_response().unwrap();
		assert_eq!(status["cleared"], false);
		assert_eq!(status["message"], "No active namespace; nothing to clear");
	}
}
