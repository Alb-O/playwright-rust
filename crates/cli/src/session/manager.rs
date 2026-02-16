//! Session orchestration for browser acquisition and lifecycle.

use std::path::Path;

use pw_rs::{StorageState, WaitUntil};
use serde_json::json;
use tracing::{debug, info, warn};

use super::descriptor::{DRIVER_HASH, SESSION_DESCRIPTOR_SCHEMA_VERSION, SessionDescriptor, now_ts};
use super::outcome::SessionHandle;
use super::repository::SessionRepository;
use super::spec::SessionRequest;
use super::strategy::{PrimarySessionStrategy, SessionStrategyInput, resolve_session_strategy};
use crate::browser::{BrowserSession, SessionOptions};
use crate::context::CommandContext;
use crate::daemon;
use crate::error::{PwError, Result};
use crate::output::SessionSource;
use crate::types::BrowserKind;

struct DaemonLease {
	endpoint: String,
	session_key: String,
}

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
		self.repository.load()
	}

	/// Clears descriptor metadata from persistence.
	pub fn clear_descriptor(&self) -> Result<bool> {
		self.repository.clear()
	}

	/// Returns the structured payload used by `session.status`.
	pub fn descriptor_status(&self) -> Result<serde_json::Value> {
		let Some(path) = self.descriptor_path().map(Path::to_path_buf) else {
			return Ok(json!({
				"active": false,
				"message": "No active namespace; session status unavailable"
			}));
		};

		match self.load_descriptor()? {
			Some(desc) => {
				let alive = desc.is_alive();
				Ok(json!({
					"active": true,
					"path": path,
					"schema_version": desc.schema_version,
					"browser": desc.browser,
					"headless": desc.headless,
					"cdp_endpoint": desc.cdp_endpoint,
					"ws_endpoint": desc.ws_endpoint,
					"workspace_id": desc.workspace_id,
					"namespace": desc.namespace,
					"session_key": desc.session_key,
					"driver_hash": desc.driver_hash,
					"pid": desc.pid,
					"created_at": desc.created_at,
					"alive": alive,
				}))
			}
			None => Ok(json!({
				"active": false,
				"message": "No session descriptor for namespace; run a browser command to create one"
			})),
		}
	}

	/// Removes descriptor metadata and returns the structured payload for `session.clear`.
	pub fn clear_descriptor_response(&self) -> Result<serde_json::Value> {
		let Some(path) = self.descriptor_path().map(Path::to_path_buf) else {
			return Ok(json!({
				"cleared": false,
				"message": "No active namespace; nothing to clear"
			}));
		};

		if self.clear_descriptor()? {
			info!(target = "pw.session", path = %path.display(), "session descriptor removed");
			Ok(json!({
				"cleared": true,
				"path": path,
			}))
		} else {
			warn!(target = "pw.session", path = %path.display(), "no session descriptor to remove");
			Ok(json!({
				"cleared": false,
				"path": path,
				"message": "No session descriptor found"
			}))
		}
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
		let storage_state = request.auth_file.map(load_storage_state).transpose()?;
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
			if let Some(handle) = self.acquire_from_descriptor(&request, storage_state.clone()).await? {
				return Ok(handle);
			}
		}

		let daemon_lease = self.acquire_from_daemon(&request, strategy.try_daemon_lease).await?;
		let (mut session, source) = self.acquire_primary(&request, strategy.primary, storage_state, daemon_lease.as_ref()).await?;

		self.auto_inject_auth_if_needed(&request, daemon_lease.as_ref(), &mut session).await?;
		self.persist_descriptor_if_needed(&request, &session, daemon_lease.as_ref());

		Ok(SessionHandle { session, source })
	}

	async fn acquire_from_descriptor(&self, request: &SessionRequest<'_>, storage_state: Option<StorageState>) -> Result<Option<SessionHandle>> {
		let Some(descriptor) = self.load_descriptor()? else {
			return Ok(None);
		};

		if !(descriptor.belongs_to(self.ctx)
			&& descriptor.matches(request.browser, request.headless, request.cdp_endpoint, Some(DRIVER_HASH))
			&& descriptor.is_alive())
		{
			return Ok(None);
		}

		let Some(endpoint) = descriptor.cdp_endpoint.as_deref().or(descriptor.ws_endpoint.as_deref()) else {
			debug!(target = "pw.session", "descriptor lacks endpoint; ignoring");
			return Ok(None);
		};

		debug!(
			target = "pw.session",
			%endpoint,
			pid = descriptor.pid,
			"reusing existing browser via cdp"
		);

		let mut session = self.session_with_options(request, storage_state, Some(endpoint)).await?;
		session.set_keep_browser_running(true);

		Ok(Some(SessionHandle {
			session,
			source: SessionSource::CachedDescriptor,
		}))
	}

	async fn acquire_from_daemon(&self, request: &SessionRequest<'_>, try_daemon_lease: bool) -> Result<Option<DaemonLease>> {
		if !try_daemon_lease {
			return Ok(None);
		}

		let Some(client) = daemon::try_connect().await else {
			return Ok(None);
		};

		let Some(namespace_id) = &self.namespace_id else {
			return Ok(None);
		};

		let session_key = format!("{}:{}:{}", namespace_id, request.browser, if request.headless { "headless" } else { "headful" });
		match daemon::request_browser(&client, request.browser, request.headless, &session_key).await {
			Ok(endpoint) => {
				debug!(
					target = "pw.session",
					%endpoint,
					session_key = %session_key,
					"using daemon browser"
				);
				Ok(Some(DaemonLease { endpoint, session_key }))
			}
			Err(err) => {
				debug!(
					target = "pw.session",
					error = %err,
					"daemon request failed; falling back"
				);
				Ok(None)
			}
		}
	}

	async fn acquire_primary(
		&self,
		request: &SessionRequest<'_>,
		primary: PrimarySessionStrategy,
		storage_state: Option<StorageState>,
		daemon_lease: Option<&DaemonLease>,
	) -> Result<(BrowserSession, SessionSource)> {
		if let Some(lease) = daemon_lease {
			let mut session = self.session_with_options(request, storage_state.clone(), Some(lease.endpoint.as_str())).await?;
			session.set_keep_browser_running(true);
			return Ok((session, SessionSource::Daemon));
		}

		match primary {
			PrimarySessionStrategy::AttachCdp => {
				let endpoint = request
					.cdp_endpoint
					.ok_or_else(|| PwError::Context("missing CDP endpoint for attach strategy".to_string()))?;
				let mut session = self.session_with_options(request, storage_state, Some(endpoint)).await?;
				session.set_keep_browser_running(true);
				Ok((session, SessionSource::CdpConnect))
			}
			PrimarySessionStrategy::PersistentDebug => {
				let port = request
					.remote_debugging_port
					.ok_or_else(|| PwError::Context("missing remote_debugging_port for persistent strategy".to_string()))?;
				if request.browser != BrowserKind::Chromium {
					return Err(PwError::BrowserLaunch(
						"Persistent sessions with remote_debugging_port require Chromium".to_string(),
					));
				}
				let session =
					BrowserSession::launch_persistent(request.wait_until, storage_state, request.headless, port, request.keep_browser_running).await?;
				Ok((session, SessionSource::PersistentDebug))
			}
			PrimarySessionStrategy::LaunchServer => {
				let session = BrowserSession::launch_server_session(request.wait_until, storage_state, request.headless, request.browser).await?;
				Ok((session, SessionSource::BrowserServer))
			}
			PrimarySessionStrategy::FreshLaunch => {
				let session = self.session_with_options(request, storage_state, None).await?;
				Ok((session, SessionSource::Fresh))
			}
		}
	}

	async fn session_with_options(
		&self,
		request: &SessionRequest<'_>,
		storage_state: Option<StorageState>,
		cdp_endpoint: Option<&str>,
	) -> Result<BrowserSession> {
		BrowserSession::with_options(SessionOptions {
			wait_until: request.wait_until,
			storage_state,
			headless: request.headless,
			browser_kind: request.browser,
			cdp_endpoint,
			launch_server: false,
			protected_urls: request.protected_urls,
			preferred_url: request.preferred_url,
			har_config: request.har_config,
			block_config: request.block_config,
			download_config: request.download_config,
		})
		.await
	}

	async fn auto_inject_auth_if_needed(&self, request: &SessionRequest<'_>, daemon_lease: Option<&DaemonLease>, session: &mut BrowserSession) -> Result<()> {
		let attached_endpoint = request.cdp_endpoint.is_some() || daemon_lease.is_some();
		if attached_endpoint && request.auth_file.is_none() {
			let auth_files = self.ctx.auth_files();
			if !auth_files.is_empty() {
				debug!(
					target = "pw.session",
					count = auth_files.len(),
					"auto-injecting cookies from project auth files"
				);
				session.inject_auth_files(&auth_files).await?;
			}
		}
		Ok(())
	}

	fn persist_descriptor_if_needed(&self, request: &SessionRequest<'_>, session: &BrowserSession, daemon_lease: Option<&DaemonLease>) {
		if self.descriptor_path().is_none() {
			return;
		}

		let cdp = session.cdp_endpoint().map(|e| e.to_string());
		let ws = session.ws_endpoint().map(|e| e.to_string());
		if cdp.is_none() && ws.is_none() {
			debug!(target = "pw.session", "no endpoint available; skipping descriptor save");
			return;
		}

		let descriptor = SessionDescriptor {
			schema_version: SESSION_DESCRIPTOR_SCHEMA_VERSION,
			pid: std::process::id(),
			browser: request.browser,
			headless: request.headless,
			cdp_endpoint: cdp,
			ws_endpoint: ws,
			workspace_id: Some(self.ctx.workspace_id().to_string()),
			namespace: Some(self.ctx.namespace().to_string()),
			session_key: daemon_lease
				.map(|lease| lease.session_key.clone())
				.or_else(|| Some(self.ctx.session_key(request.browser, request.headless))),
			driver_hash: Some(DRIVER_HASH.to_string()),
			created_at: now_ts(),
		};

		if let Err(err) = self.repository.save(&descriptor) {
			if let Some(path) = self.descriptor_path() {
				warn!(
					target = "pw.session",
					path = %path.display(),
					error = %err,
					"failed to save session descriptor"
				);
			} else {
				warn!(target = "pw.session", error = %err, "failed to save session descriptor");
			}
		} else {
			debug!(
				target = "pw.session",
				cdp = ?descriptor.cdp_endpoint,
				ws = ?descriptor.ws_endpoint,
				"saved session descriptor"
			);
		}
	}
}

fn load_storage_state(path: &Path) -> Result<StorageState> {
	StorageState::from_file(path).map_err(|e| PwError::BrowserLaunch(format!("Failed to load auth file: {}", e)))
}

#[cfg(test)]
mod tests {
	use pw_rs::WaitUntil;

	use super::*;
	use crate::context::{BlockConfig, DownloadConfig, HarConfig};

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
