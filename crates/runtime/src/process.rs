//! Process and port lifecycle helpers shared by CLI/runtime consumers.

use std::path::PathBuf;

/// Returns `true` when a process with `pid` appears alive on this platform.
pub fn pid_is_alive(pid: u32) -> bool {
	#[cfg(unix)]
	{
		if pid == 0 {
			return false;
		}

		if PathBuf::from("/proc").join(pid.to_string()).exists() {
			return true;
		}

		std::process::Command::new("kill")
			.arg("-0")
			.arg(pid.to_string())
			.status()
			.map(|status| status.success())
			.unwrap_or(pid == std::process::id())
	}

	#[cfg(windows)]
	{
		let filter = format!("PID eq {pid}");
		if let Ok(output) = std::process::Command::new("tasklist").args(["/FI", &filter, "/FO", "CSV", "/NH"]).output() {
			if output.status.success() {
				let stdout = String::from_utf8_lossy(&output.stdout);
				return tasklist_has_pid(stdout.as_ref(), pid);
			}
		}

		pid == std::process::id()
	}

	#[cfg(not(any(unix, windows)))]
	{
		pid == std::process::id()
	}
}

/// Returns `true` when `port` can be bound on localhost.
pub fn port_available(port: u16) -> bool {
	std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

#[cfg(any(test, windows))]
fn tasklist_has_pid(output: &str, pid: u32) -> bool {
	let pid_str = pid.to_string();
	output.lines().any(|line| {
		let line = line.trim();
		if !line.starts_with('"') {
			return false;
		}

		line.trim_matches('"')
			.split("\",\"")
			.nth(1)
			.is_some_and(|field| field.trim() == pid_str.as_str())
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(any(test, windows))]
	#[test]
	fn tasklist_parser_matches_csv_line() {
		let output = "\"chrome.exe\",\"1234\",\"Console\",\"1\",\"250,000 K\"\r\n";
		assert!(tasklist_has_pid(output, 1234));
		assert!(!tasklist_has_pid(output, 9999));
	}

	#[cfg(any(test, windows))]
	#[test]
	fn tasklist_parser_ignores_non_csv_lines() {
		let output = "INFO: No tasks are running which match the specified criteria.\r\n";
		assert!(!tasklist_has_pid(output, 1234));
	}

	#[cfg(unix)]
	#[test]
	fn current_process_is_alive() {
		assert!(pid_is_alive(std::process::id()));
	}

	#[cfg(unix)]
	#[test]
	fn pid_zero_is_never_alive() {
		assert!(!pid_is_alive(0));
	}

	#[test]
	fn bound_port_is_reported_unavailable() {
		let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
		let port = listener.local_addr().unwrap().port();
		assert!(!port_available(port));
		drop(listener);
		assert!(port_available(port));
	}
}
