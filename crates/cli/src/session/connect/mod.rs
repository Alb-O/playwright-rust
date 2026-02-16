//! Shared browser connect/discover helpers for CLI commands.
//!
//! This module owns CDP discovery, browser launch/kill orchestration, and
//! profile-scoped endpoint persistence used by `connect` and related flows.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::context_store::ContextState;
use crate::error::Result;

mod auth_injector;
mod browser_finder;
mod browser_launcher;
mod cdp_probe;
mod process_killer;
mod user_data_dir;
pub mod wsl;

pub use cdp_probe::{CdpVersionInfo, fetch_cdp_endpoint};
pub use user_data_dir::{resolve_connect_port, resolve_user_data_dir};

#[derive(Debug, Clone)]
struct ConnectAuthPayload {
	auth_file: PathBuf,
	cookies_applied: usize,
	origins_present: usize,
}

impl From<auth_injector::AuthApplySummary> for ConnectAuthPayload {
	fn from(summary: auth_injector::AuthApplySummary) -> Self {
		Self {
			auth_file: summary.auth_file,
			cookies_applied: summary.cookies_applied,
			origins_present: summary.origins_present,
		}
	}
}

#[derive(Debug, Clone)]
enum ConnectResult {
	Killed {
		port: u16,
		pids: String,
	},
	KillNoop {
		port: u16,
	},
	Cleared,
	Launched {
		endpoint: String,
		browser: Option<String>,
		port: u16,
		user_data_dir: PathBuf,
		auth: Option<ConnectAuthPayload>,
	},
	Discovered {
		endpoint: String,
		browser: Option<String>,
		port: u16,
		auth: Option<ConnectAuthPayload>,
	},
	Set {
		endpoint: String,
	},
	Show {
		endpoint: Option<String>,
	},
}

impl ConnectResult {
	fn into_json(self) -> Value {
		match self {
			ConnectResult::Killed { port, pids } => json!({
				"action": "killed",
				"port": port,
				"pids": pids,
				"message": format!("Killed Chrome process(es) on port {}: {}", port, pids),
			}),
			ConnectResult::KillNoop { port } => json!({
				"action": "kill",
				"port": port,
				"message": format!("No Chrome process found on port {}", port)
			}),
			ConnectResult::Cleared => json!({
				"action": "cleared",
				"message": "CDP endpoint cleared"
			}),
			ConnectResult::Launched {
				endpoint,
				browser,
				port,
				user_data_dir,
				auth,
			} => {
				let message = if let Some(summary) = &auth {
					format!(
						"Chrome launched and connected on port {} (applied {} auth cookies from {})",
						port,
						summary.cookies_applied,
						summary.auth_file.display()
					)
				} else {
					format!("Chrome launched and connected on port {}", port)
				};

				json!({
					"action": "launched",
					"endpoint": endpoint,
					"browser": browser,
					"port": port,
					"user_data_dir": user_data_dir,
					"auth": auth.as_ref().map(|summary| json!({
						"file": summary.auth_file,
						"cookiesApplied": summary.cookies_applied,
						"originsPresent": summary.origins_present
					})),
					"message": message,
				})
			}
			ConnectResult::Discovered { endpoint, browser, port, auth } => {
				let message = if let Some(summary) = &auth {
					format!(
						"Connected to existing Chrome instance (applied {} auth cookies from {})",
						summary.cookies_applied,
						summary.auth_file.display()
					)
				} else {
					"Connected to existing Chrome instance".to_string()
				};

				json!({
					"action": "discovered",
					"endpoint": endpoint,
					"browser": browser,
					"port": port,
					"auth": auth.as_ref().map(|summary| json!({
						"file": summary.auth_file,
						"cookiesApplied": summary.cookies_applied,
						"originsPresent": summary.origins_present
					})),
					"message": message,
				})
			}
			ConnectResult::Set { endpoint } => json!({
				"action": "set",
				"endpoint": endpoint,
				"message": format!("CDP endpoint set to {}", endpoint)
			}),
			ConnectResult::Show { endpoint } => match endpoint {
				Some(endpoint) => json!({
					"action": "show",
					"endpoint": endpoint,
					"message": format!("Current CDP endpoint: {}", endpoint)
				}),
				None => json!({
					"action": "show",
					"endpoint": null,
					"message": "No CDP endpoint configured. Use --launch or --discover to connect."
				}),
			},
		}
	}
}

/// Kills the browser listening on `port` and clears stored endpoint if found.
pub async fn kill_browser_on_port(ctx_state: &mut ContextState, port: u16) -> Result<Value> {
	let result = match process_killer::kill_chrome(port).await? {
		Some(pids) => {
			ctx_state.set_cdp_endpoint(None);
			ConnectResult::Killed { port, pids }
		}
		None => ConnectResult::KillNoop { port },
	};
	Ok(result.into_json())
}

/// Clears the stored CDP endpoint from context defaults.
pub fn clear_cdp_endpoint(ctx_state: &mut ContextState) -> Value {
	ctx_state.set_cdp_endpoint(None);
	ConnectResult::Cleared.into_json()
}

/// Launches a browser with remote debugging and stores discovered endpoint.
pub async fn launch_and_connect(ctx_state: &mut ContextState, port: u16, user_data_dir: Option<&Path>, auth_file: Option<&Path>) -> Result<Value> {
	let launch_data_dir = resolve_user_data_dir(ctx_state, user_data_dir)?;
	let info = browser_launcher::launch_chrome(port, Some(launch_data_dir.as_path())).await?;
	let auth_applied = auth_injector::maybe_apply_auth(&info.web_socket_debugger_url, auth_file)
		.await?
		.map(ConnectAuthPayload::from);
	ctx_state.set_cdp_endpoint(Some(info.web_socket_debugger_url.clone()));

	Ok(ConnectResult::Launched {
		endpoint: info.web_socket_debugger_url,
		browser: info.browser,
		port,
		user_data_dir: launch_data_dir,
		auth: auth_applied,
	}
	.into_json())
}

/// Discovers an existing remote-debugging browser and stores endpoint.
pub async fn discover_and_connect(ctx_state: &mut ContextState, port: u16, auth_file: Option<&Path>) -> Result<Value> {
	let info = cdp_probe::discover_chrome(port).await?;
	let auth_applied = auth_injector::maybe_apply_auth(&info.web_socket_debugger_url, auth_file)
		.await?
		.map(ConnectAuthPayload::from);
	ctx_state.set_cdp_endpoint(Some(info.web_socket_debugger_url.clone()));

	Ok(ConnectResult::Discovered {
		endpoint: info.web_socket_debugger_url,
		browser: info.browser,
		port,
		auth: auth_applied,
	}
	.into_json())
}

/// Stores an explicit CDP endpoint in context defaults.
pub fn set_cdp_endpoint(ctx_state: &mut ContextState, endpoint: &str) -> Value {
	ctx_state.set_cdp_endpoint(Some(endpoint.to_string()));
	ConnectResult::Set {
		endpoint: endpoint.to_string(),
	}
	.into_json()
}

/// Returns current endpoint configuration payload for command output.
pub fn show_cdp_endpoint(ctx_state: &ContextState) -> Value {
	ConnectResult::Show {
		endpoint: ctx_state.cdp_endpoint().map(str::to_string),
	}
	.into_json()
}
