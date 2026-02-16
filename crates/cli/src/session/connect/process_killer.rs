//! Browser process termination helpers for connect flows.

use std::process::Command;

use tracing::debug;

use super::cdp_probe::fetch_cdp_endpoint;
use crate::error::{PwError, Result};

pub(super) async fn kill_chrome(port: u16) -> Result<Option<String>> {
	if fetch_cdp_endpoint(port).await.is_err() {
		return Ok(None);
	}

	#[cfg(unix)]
	{
		let output = Command::new("lsof")
			.args(["-ti", &format!(":{}", port)])
			.output()
			.map_err(|e| PwError::Context(format!("Failed to run lsof: {}", e)))?;

		if !output.status.success() || output.stdout.is_empty() {
			return Err(PwError::Context(format!("Could not find process listening on port {}", port)));
		}

		let pids: Vec<&str> = std::str::from_utf8(&output.stdout)
			.map_err(|e| PwError::Context(format!("Invalid lsof output: {}", e)))?
			.trim()
			.lines()
			.collect();

		if pids.is_empty() {
			return Err(PwError::Context(format!("No process found on port {}", port)));
		}

		let mut killed = Vec::new();
		for pid in &pids {
			debug!("Killing PID {} on port {}", pid, port);
			let kill_result = Command::new("kill").args(["-TERM", pid]).status();

			match kill_result {
				Ok(status) if status.success() => killed.push(*pid),
				Ok(_) => debug!("kill -TERM {} returned non-zero", pid),
				Err(e) => debug!("Failed to kill {}: {}", pid, e),
			}
		}

		if killed.is_empty() {
			return Err(PwError::Context(format!("Failed to kill process on port {}", port)));
		}

		Ok(Some(killed.join(", ")))
	}

	#[cfg(windows)]
	{
		let output = Command::new("netstat")
			.args(["-ano"])
			.output()
			.map_err(|e| PwError::Context(format!("Failed to run netstat: {}", e)))?;

		let output_str = String::from_utf8_lossy(&output.stdout);
		let port_str = format!(":{}", port);

		for line in output_str.lines() {
			if line.contains(&port_str) && line.contains("LISTENING") {
				let parts: Vec<&str> = line.split_whitespace().collect();
				if let Some(pid) = parts.last() {
					let kill_result = Command::new("taskkill").args(["/PID", pid, "/F"]).status();

					if kill_result.map(|s| s.success()).unwrap_or(false) {
						return Ok(Some(pid.to_string()));
					}
				}
			}
		}

		Err(PwError::Context(format!("Could not find or kill process on port {}", port)))
	}
}
