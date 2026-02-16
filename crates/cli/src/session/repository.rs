//! Session descriptor persistence facade used by command/session services.

use std::path::{Path, PathBuf};

use super::descriptor::SessionDescriptor;
use crate::error::Result;

/// Repository wrapper for profile-scoped session descriptor persistence.
#[derive(Debug, Clone, Default)]
pub struct SessionRepository {
	path: Option<PathBuf>,
}

impl SessionRepository {
	/// Creates a repository from an optional descriptor path.
	pub fn new(path: Option<PathBuf>) -> Self {
		Self { path }
	}

	/// Returns descriptor path when descriptor persistence is enabled.
	pub fn path(&self) -> Option<&Path> {
		self.path.as_deref()
	}

	/// Loads the persisted descriptor from disk.
	pub fn load(&self) -> Result<Option<SessionDescriptor>> {
		let Some(path) = self.path() else {
			return Ok(None);
		};
		SessionDescriptor::load(path)
	}

	/// Persists a descriptor to disk.
	pub fn save(&self, descriptor: &SessionDescriptor) -> Result<()> {
		let Some(path) = self.path() else {
			return Ok(());
		};
		descriptor.save(path)
	}

	/// Removes the descriptor file if present.
	pub fn clear(&self) -> Result<bool> {
		let Some(path) = self.path() else {
			return Ok(false);
		};
		match std::fs::remove_file(path) {
			Ok(()) => Ok(true),
			Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
			Err(err) => Err(err.into()),
		}
	}
}
