//! Browser executable discovery for connect launch flows.

use std::path::PathBuf;

pub(super) fn find_chrome_executable() -> Option<String> {
	let candidates: Vec<String> = if cfg!(target_os = "macos") {
		vec![
			"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
			"/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
			"/Applications/Chromium.app/Contents/MacOS/Chromium",
			"/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
		]
		.into_iter()
		.map(str::to_string)
		.collect()
	} else if cfg!(target_os = "windows") {
		windows_browser_candidates()
	} else {
		vec![
			"helium",
			"brave",
			"brave-browser",
			"google-chrome-stable",
			"google-chrome",
			"chromium-browser",
			"chromium",
			"/usr/bin/helium",
			"/usr/bin/brave",
			"/usr/bin/brave-browser",
			"/usr/bin/google-chrome-stable",
			"/usr/bin/google-chrome",
			"/usr/bin/chromium-browser",
			"/usr/bin/chromium",
			"/snap/bin/chromium",
			"/snap/bin/brave",
		]
		.into_iter()
		.map(str::to_string)
		.collect()
	};

	for candidate in candidates {
		if candidate.starts_with('/') || candidate.contains('\\') || candidate.contains(':') {
			if std::path::Path::new(&candidate).exists() {
				return Some(candidate);
			}
		} else if which::which(&candidate).is_ok() {
			return Some(candidate);
		}
	}

	None
}

pub(super) fn windows_browser_candidates() -> Vec<String> {
	let mut candidates = Vec::new();

	let mut roots = Vec::new();
	for key in ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"] {
		if let Ok(value) = std::env::var(key) {
			roots.push(PathBuf::from(value));
		}
	}
	if roots.is_empty() {
		roots.push(PathBuf::from(r"C:\Program Files"));
		roots.push(PathBuf::from(r"C:\Program Files (x86)"));
	}

	let suffixes: &[&[&str]] = &[
		&["Google", "Chrome", "Application", "chrome.exe"],
		&["Microsoft", "Edge", "Application", "msedge.exe"],
		&["BraveSoftware", "Brave-Browser", "Application", "brave.exe"],
		&["Chromium", "Application", "chrome.exe"],
	];

	for root in roots {
		for suffix in suffixes {
			let mut path = root.clone();
			for component in *suffix {
				path.push(component);
			}
			candidates.push(path.to_string_lossy().to_string());
		}
	}

	candidates.extend([
		"chrome".to_string(),
		"chrome.exe".to_string(),
		"msedge".to_string(),
		"msedge.exe".to_string(),
		"brave".to_string(),
		"brave.exe".to_string(),
		"chromium".to_string(),
		"chromium.exe".to_string(),
	]);

	candidates
}

#[cfg(test)]
mod tests {
	use super::windows_browser_candidates;

	#[test]
	fn windows_browser_candidates_include_common_commands() {
		let candidates = windows_browser_candidates();
		assert!(candidates.contains(&"chrome.exe".to_string()));
		assert!(candidates.contains(&"msedge.exe".to_string()));
		assert!(candidates.contains(&"brave.exe".to_string()));
	}
}
