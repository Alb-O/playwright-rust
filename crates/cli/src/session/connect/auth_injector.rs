//! Auth storage-state loading and CDP cookie injection helpers.

use std::path::{Path, PathBuf};

use pw_rs::{Playwright, StorageState};

use crate::error::{PwError, Result};

#[derive(Debug, Clone)]
pub(super) struct AuthApplySummary {
	pub auth_file: PathBuf,
	pub cookies_applied: usize,
	pub origins_present: usize,
}

pub(super) fn load_auth_state(auth_file: &Path) -> Result<StorageState> {
	StorageState::from_file(auth_file).map_err(|e| PwError::BrowserLaunch(format!("Failed to load auth file: {}", e)))
}

async fn apply_auth_state_to_cdp(endpoint: &str, auth_file: &Path, state: StorageState) -> Result<AuthApplySummary> {
	let cookies_applied = state.cookies.len();
	let origins_present = state.origins.len();

	let playwright = Playwright::launch()
		.await
		.map_err(|e| PwError::BrowserLaunch(format!("Failed to start Playwright: {}", e)))?;
	let connected = playwright
		.chromium()
		.connect_over_cdp(endpoint)
		.await
		.map_err(|e| PwError::Context(format!("Failed to connect over CDP for auth injection: {}", e)))?;

	let context = connected
		.default_context
		.ok_or_else(|| PwError::Context("Connected browser did not expose a default context for auth injection".into()))?;

	if cookies_applied > 0 {
		context
			.add_cookies(state.cookies)
			.await
			.map_err(|e| PwError::Context(format!("Failed to inject auth cookies from {}: {}", auth_file.display(), e)))?;
	}

	Ok(AuthApplySummary {
		auth_file: auth_file.to_path_buf(),
		cookies_applied,
		origins_present,
	})
}

pub(super) async fn maybe_apply_auth(endpoint: &str, auth_file: Option<&Path>) -> Result<Option<AuthApplySummary>> {
	let Some(path) = auth_file else {
		return Ok(None);
	};
	let state = load_auth_state(path)?;
	let summary = apply_auth_state_to_cdp(endpoint, path, state).await?;
	Ok(Some(summary))
}

#[cfg(test)]
mod tests {
	use std::fs;

	use tempfile::TempDir;

	use super::*;

	#[test]
	fn load_auth_state_errors_for_missing_file() {
		let err = load_auth_state(Path::new("/definitely/missing/auth.json")).unwrap_err();
		assert!(err.to_string().contains("Failed to load auth file"));
	}

	#[test]
	fn load_auth_state_accepts_storage_state_file() {
		let temp = TempDir::new().unwrap();
		let auth = temp.path().join("auth.json");
		fs::write(
			&auth,
			r#"{
  "cookies": [
    {
      "name": "session",
      "value": "token",
      "domain": ".example.com",
      "path": "/",
      "expires": -1.0,
      "httpOnly": true,
      "secure": true,
      "sameSite": "Lax"
    }
  ],
  "origins": []
}"#,
		)
		.unwrap();

		let state = load_auth_state(&auth).unwrap();
		assert_eq!(state.cookies.len(), 1);
		assert_eq!(state.origins.len(), 0);
	}
}
