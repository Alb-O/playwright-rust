//! Browser process launch helpers for connect flows.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::browser_finder::find_chrome_executable;
use super::cdp_probe::{CdpVersionInfo, fetch_cdp_endpoint};
use super::wsl;
use crate::error::{PwError, Result};

pub(super) async fn launch_chrome(port: u16, user_data_dir: Option<&Path>) -> Result<CdpVersionInfo> {
	if wsl::is_wsl() {
		return wsl::launch_windows_chrome_from_wsl(port, user_data_dir).await;
	}

	let chrome_path = find_chrome_executable().ok_or_else(|| {
		PwError::Context(
			"Could not find Chrome/Chromium executable. \n\
             Please install Chrome or specify path manually."
				.into(),
		)
	})?;

	let mut args = vec![
		format!("--remote-debugging-port={}", port),
		"--no-first-run".to_string(),
		"--no-default-browser-check".to_string(),
	];

	if let Some(dir) = user_data_dir {
		args.push(format!("--user-data-dir={}", dir.display()));
	}

	let mut cmd = Command::new(&chrome_path);
	cmd.args(&args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());

	#[cfg(unix)]
	std::os::unix::process::CommandExt::process_group(&mut cmd, 0);

	let mut child = cmd
		.spawn()
		.map_err(|e| PwError::Context(format!("Failed to launch Chrome at {}: {}", chrome_path, e)))?;

	let max_attempts = 8;
	let mut last_error = "endpoint not reachable".to_string();
	for _ in 0..max_attempts {
		tokio::time::sleep(Duration::from_millis(200)).await;

		if let Ok(Some(status)) = child.try_wait() {
			return Err(PwError::Context(format!(
				"Chrome exited before debugging endpoint became available (status: {}). \
	             Launch it manually with --remote-debugging-port={} and retry `pw connect --discover`.",
				status, port
			)));
		}

		match fetch_cdp_endpoint(port).await {
			Ok(info) => return Ok(info),
			Err(e) => {
				last_error = match e {
					PwError::Context(msg) => msg,
					other => other.to_string(),
				};
				continue;
			}
		}
	}

	Err(PwError::Context(format!(
		"Chrome launched but debugging endpoint not available on port {}. \n\
         Last error: {}\n\
         If Chrome/Chromium recently updated, remote debugging may require a dedicated \
         --user-data-dir. Try: pw connect --launch --user-data-dir <path>",
		port, last_error
	)))
}
