//! Path resolution helpers for connect profile data and CDP port selection.

use std::path::{Path, PathBuf};

use pw_rs::dirs;

use super::wsl;
use crate::context_store::ContextState;
use crate::error::Result;
use crate::workspace::{STATE_VERSION_DIR, compute_cdp_port, ensure_state_root_gitignore};

/// Resolves/creates user-data-dir used for launched browser profiles.
pub fn resolve_user_data_dir(ctx_state: &ContextState, user_data_dir: Option<&Path>) -> Result<PathBuf> {
	let resolved = if wsl::is_wsl() {
		wsl::resolve_wsl_user_data_dir(ctx_state, user_data_dir)
	} else if let Some(dir) = user_data_dir {
		if dir.is_absolute() {
			dir.to_path_buf()
		} else {
			ctx_state.workspace_root().join(dir)
		}
	} else {
		let state_root = ctx_state.workspace_root().join(dirs::PLAYWRIGHT).join(STATE_VERSION_DIR);
		ensure_state_root_gitignore(&state_root)?;
		state_root.join("profiles").join(ctx_state.namespace()).join("connect-user-data")
	};

	std::fs::create_dir_all(&resolved)?;
	Ok(resolved)
}

/// Resolves effective CDP port from explicit value or namespace identity.
pub fn resolve_connect_port(ctx_state: &ContextState, requested_port: Option<u16>) -> u16 {
	requested_port.unwrap_or_else(|| compute_cdp_port(&ctx_state.namespace_id()))
}

#[cfg(test)]
mod tests {
	use tempfile::TempDir;

	use super::*;

	#[test]
	fn resolve_user_data_dir_defaults_to_namespace_scoped_path() {
		let temp = TempDir::new().unwrap();
		let ctx_state = ContextState::new(
			temp.path().to_path_buf(),
			"workspace-id".to_string(),
			"agent-a".to_string(),
			None,
			false,
			true,
			false,
		)
		.unwrap();

		let dir = resolve_user_data_dir(&ctx_state, None).unwrap();
		if wsl::is_wsl() {
			assert_eq!(
				dir,
				PathBuf::from(wsl::WSL_MANAGED_USER_DATA_ROOT).join("workspace-id").join("agent-a"),
				"resolved path was {}",
				dir.display()
			);
		} else {
			assert!(
				dir.ends_with("playwright/.pw-cli-v4/profiles/agent-a/connect-user-data"),
				"resolved path was {}",
				dir.display()
			);
			assert!(temp.path().join("playwright").join(".pw-cli-v4").join(".gitignore").exists());
		}
	}

	#[test]
	fn resolve_user_data_dir_makes_relative_paths_workspace_relative() {
		let temp = TempDir::new().unwrap();
		let ctx_state = ContextState::new(
			temp.path().to_path_buf(),
			"workspace-id".to_string(),
			"default".to_string(),
			None,
			false,
			true,
			false,
		)
		.unwrap();

		let dir = resolve_user_data_dir(&ctx_state, Some(std::path::Path::new("profiles/debug")));
		let expected = temp.path().join("profiles/debug");
		assert_eq!(dir.unwrap(), expected);
	}

	#[test]
	fn resolve_connect_port_uses_namespace_identity_when_unspecified() {
		let temp = TempDir::new().unwrap();
		let ctx_state = ContextState::new(
			temp.path().to_path_buf(),
			"workspace-id".to_string(),
			"agent-a".to_string(),
			None,
			false,
			true,
			false,
		)
		.unwrap();

		let expected = compute_cdp_port(&ctx_state.namespace_id());
		assert_eq!(resolve_connect_port(&ctx_state, None), expected);
	}

	#[test]
	fn resolve_connect_port_prefers_explicit_port() {
		let temp = TempDir::new().unwrap();
		let ctx_state = ContextState::new(
			temp.path().to_path_buf(),
			"workspace-id".to_string(),
			"agent-a".to_string(),
			None,
			false,
			true,
			false,
		)
		.unwrap();

		assert_eq!(resolve_connect_port(&ctx_state, Some(9555)), 9555);
	}
}
