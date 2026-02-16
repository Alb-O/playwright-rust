//! CDP endpoint probing and discovery helpers.

use std::time::Duration;

use serde::Deserialize;

use crate::error::{PwError, Result};

/// `/json/version` response subset from Chrome DevTools Protocol.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdpVersionInfo {
	#[serde(rename = "webSocketDebuggerUrl")]
	pub web_socket_debugger_url: String,
	#[serde(rename = "Browser")]
	pub browser: Option<String>,
}

/// Resolves CDP version metadata from `/json/version` on `port`.
pub async fn fetch_cdp_endpoint(port: u16) -> Result<CdpVersionInfo> {
	let client = reqwest::Client::builder()
		.timeout(Duration::from_millis(400))
		.build()
		.map_err(|e| PwError::Context(format!("Failed to create HTTP client: {}", e)))?;
	let mut last_error = "no response".to_string();

	for url in [
		format!("http://127.0.0.1:{}/json/version", port),
		format!("http://localhost:{}/json/version", port),
		format!("http://[::1]:{}/json/version", port),
	] {
		let response = match client.get(&url).send().await {
			Ok(r) => r,
			Err(e) => {
				last_error = e.to_string();
				continue;
			}
		};

		if !response.status().is_success() {
			last_error = format!("unexpected status {}", response.status());
			continue;
		}

		let info: CdpVersionInfo = response
			.json()
			.await
			.map_err(|e| PwError::Context(format!("Failed to parse CDP response: {}", e)))?;
		return Ok(info);
	}

	Err(PwError::Context(format!("Failed to connect to port {}: {}", port, last_error)))
}

/// Discovers an existing debug browser and returns endpoint metadata.
pub async fn discover_chrome(port: u16) -> Result<CdpVersionInfo> {
	let launch_hint = if cfg!(target_os = "windows") {
		format!("msedge.exe --remote-debugging-port={}", port)
	} else {
		format!("google-chrome --remote-debugging-port={}", port)
	};

	fetch_cdp_endpoint(port).await.map_err(|e| {
		PwError::Context(format!(
			"No Chrome instance with remote debugging found on port {}. \n\
	             Last error: {}\n\
	             Try running: {}\n\
	             Or use: pw connect --launch --port {}",
			port, e, launch_hint, port
		))
	})
}
