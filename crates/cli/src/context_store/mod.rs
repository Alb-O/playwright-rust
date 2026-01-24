//! Persistent context storage for CLI state across invocations.
//!
//! Manages two-tier storage: global contexts in `~/.config/pw/cli/` and
//! project-local contexts in `playwright/.pw-cli/`. Tracks last URL, selector,
//! output path, and CDP endpoint for command repetition.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use pw::dirs;
use serde::{Deserialize, Serialize};

use crate::context::CommandContext;
use crate::error::{PwError, Result};
use crate::types::BrowserKind;

#[cfg(test)]
mod tests;

const CONTEXT_SCHEMA_VERSION: u32 = 1;
const SESSION_TIMEOUT_SECS: u64 = 3600;

/// Whether a context is stored globally or per-project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContextScope {
	#[default]
	Global,
	Project,
}

/// Persisted state for a named context.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StoredContext {
	#[serde(default)]
	pub scope: ContextScope,
	#[serde(default)]
	pub project_root: Option<String>,
	#[serde(default)]
	pub base_url: Option<String>,
	#[serde(default)]
	pub last_url: Option<String>,
	#[serde(default)]
	pub last_selector: Option<String>,
	#[serde(default)]
	pub last_output: Option<String>,
	#[serde(default)]
	pub browser: Option<BrowserKind>,
	#[serde(default)]
	pub headless: Option<bool>,
	#[serde(default)]
	pub auth_file: Option<String>,
	#[serde(default)]
	pub cdp_endpoint: Option<String>,
	#[serde(default)]
	pub last_used_at: Option<u64>,
	/// URL patterns to protect from CLI access.
	#[serde(default)]
	pub protected_urls: Vec<String>,
}

/// Tracks which context is active globally and per-project.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActiveContexts {
	#[serde(default)]
	pub global: Option<String>,
	/// Maps project root paths to their active context names.
	#[serde(default)]
	pub projects: HashMap<String, String>,
}

/// On-disk format for a context store file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextStoreFile {
	pub schema: u32,
	#[serde(default)]
	pub active: ActiveContexts,
	#[serde(default)]
	pub contexts: HashMap<String, StoredContext>,
}

impl Default for ContextStoreFile {
	fn default() -> Self {
		Self {
			schema: CONTEXT_SCHEMA_VERSION,
			active: ActiveContexts::default(),
			contexts: HashMap::new(),
		}
	}
}

/// A single context store file with its path and scope.
#[derive(Debug)]
pub struct ContextStore {
	pub scope: ContextScope,
	path: PathBuf,
	pub file: ContextStoreFile,
}

impl ContextStore {
	pub fn load(path: PathBuf, scope: ContextScope) -> Self {
		let file = fs::read_to_string(&path)
			.ok()
			.and_then(|content| serde_json::from_str(&content).ok())
			.unwrap_or_default();
		Self { scope, path, file }
	}

	/// Gets or creates a context entry by name.
	pub fn ensure(&mut self, name: &str, project_root: Option<&Path>) -> &mut StoredContext {
		self.file
			.contexts
			.entry(name.to_string())
			.or_insert_with(|| StoredContext {
				scope: self.scope.clone(),
				project_root: project_root.map(|p| p.to_string_lossy().to_string()),
				..Default::default()
			})
	}

	pub fn get(&self, name: &str) -> Option<&StoredContext> {
		self.file.contexts.get(name)
	}

	pub fn save(&self) -> Result<()> {
		if let Some(parent) = self.path.parent() {
			fs::create_dir_all(parent)?;
		}
		let json = serde_json::to_string_pretty(&self.file)?;
		fs::write(&self.path, json)?;
		Ok(())
	}
}

/// Holds both global and optional project context stores.
#[derive(Debug)]
pub struct ContextBook {
	pub global: ContextStore,
	pub project: Option<ContextStore>,
}

impl ContextBook {
	pub fn new(project_root: Option<&Path>) -> Self {
		Self {
			global: ContextStore::load(global_store_path(), ContextScope::Global),
			project: project_root
				.map(|root| ContextStore::load(project_store_path(root), ContextScope::Project)),
		}
	}
}

/// The currently active context with its data loaded.
#[derive(Debug)]
pub struct SelectedContext {
	pub name: String,
	pub scope: ContextScope,
	pub data: StoredContext,
}

/// Runtime context state manager.
///
/// Handles context selection, URL/selector caching, and persistence.
/// Auto-refreshes stale sessions after [`SESSION_TIMEOUT_SECS`].
#[derive(Debug)]
pub struct ContextState {
	stores: ContextBook,
	selected: Option<SelectedContext>,
	project_root: Option<PathBuf>,
	base_url_override: Option<String>,
	no_context: bool,
	no_save: bool,
	refresh: bool,
}

impl ContextState {
	pub fn new(
		project_root: Option<PathBuf>,
		requested_context: Option<String>,
		base_url_override: Option<String>,
		no_context: bool,
		no_save: bool,
		refresh: bool,
	) -> Result<Self> {
		let mut stores = ContextBook::new(project_root.as_deref());
		let mut selected = if no_context {
			None
		} else {
			select_context(
				&mut stores,
				project_root.as_deref(),
				requested_context.as_deref(),
			)
		};

		if let (Some(ctx), Some(base)) = (&mut selected, &base_url_override) {
			ctx.data.base_url = Some(base.clone());
		}

		Ok(Self {
			refresh: refresh || is_session_stale(selected.as_ref()),
			stores,
			selected,
			project_root,
			base_url_override,
			no_context,
			no_save,
		})
	}

	#[cfg(test)]
	pub(crate) fn test_new(stores: ContextBook, selected: Option<SelectedContext>) -> Self {
		Self {
			stores,
			selected,
			project_root: None,
			base_url_override: None,
			no_context: false,
			no_save: false,
			refresh: false,
		}
	}

	pub fn active_name(&self) -> Option<&str> {
		self.selected.as_ref().map(|s| s.name.as_str())
	}

	pub fn session_descriptor_path(&self) -> Option<PathBuf> {
		if self.no_context {
			return None;
		}
		let selected = self.selected.as_ref()?;
		let dir = match selected.scope {
			ContextScope::Project => project_sessions_dir(self.project_root.as_ref()?),
			ContextScope::Global => global_sessions_dir(),
		};
		Some(dir.join(format!("{}.json", selected.name)))
	}

	pub fn refresh_requested(&self) -> bool {
		self.refresh
	}

	/// Returns true if context has a URL available.
	pub fn has_context_url(&self) -> bool {
		if self.no_context {
			return false;
		}
		if self.base_url_override.is_some() {
			return true;
		}
		self.selected.as_ref().is_some_and(|s| {
			(!self.refresh && s.data.last_url.is_some()) || s.data.base_url.is_some()
		})
	}

	pub fn resolve_selector(
		&self,
		provided: Option<String>,
		fallback: Option<&str>,
	) -> Result<String> {
		if let Some(selector) = provided {
			return Ok(selector);
		}

		if self.no_context {
			return fallback.map(String::from).ok_or_else(|| {
				PwError::Context("Selector is required when context usage is disabled".into())
			});
		}

		let Some(selected) = &self.selected else {
			return fallback
				.map(String::from)
				.ok_or_else(|| PwError::Context("No selector available".into()));
		};

		if !self.refresh {
			if let Some(selector) = &selected.data.last_selector {
				return Ok(selector.clone());
			}
		}

		fallback
			.map(String::from)
			.ok_or_else(|| PwError::Context("No selector available".into()))
	}

	/// Returns the CDP endpoint from the global `default` context.
	///
	/// CDP endpoints represent system-wide browser connections and are stored
	/// globally to prevent stale endpoint errors when switching directories.
	pub fn cdp_endpoint(&self) -> Option<&str> {
		if self.no_context {
			return None;
		}
		self.stores
			.global
			.file
			.contexts
			.get("default")
			.and_then(|ctx| ctx.cdp_endpoint.as_deref())
	}

	/// Returns the last URL from the selected context.
	pub fn last_url(&self) -> Option<&str> {
		if self.no_context {
			return None;
		}
		self.selected
			.as_ref()
			.and_then(|s| s.data.last_url.as_deref())
	}

	/// Sets the CDP endpoint in the global `default` context.
	///
	/// Also updates [`SelectedContext::data`] if the selected context is the
	/// global default, ensuring [`persist`](Self::persist) doesn't overwrite
	/// with stale data.
	pub fn set_cdp_endpoint(&mut self, endpoint: Option<String>) {
		if self.no_save || self.no_context {
			return;
		}

		self.stores
			.global
			.file
			.contexts
			.entry("default".to_string())
			.or_default()
			.cdp_endpoint = endpoint.clone();

		if let Some(ref mut selected) = self.selected {
			if selected.name == "default" && selected.scope == ContextScope::Global {
				selected.data.cdp_endpoint = endpoint;
			}
		}
	}

	/// Returns protected URL patterns from the selected context.
	pub fn protected_urls(&self) -> &[String] {
		if self.no_context {
			return &[];
		}
		self.selected
			.as_ref()
			.map(|s| s.data.protected_urls.as_slice())
			.unwrap_or(&[])
	}

	/// Returns true if the URL matches any protected pattern.
	pub fn is_protected(&self, url: &str) -> bool {
		let url_lower = url.to_lowercase();
		self.protected_urls()
			.iter()
			.any(|pattern| url_lower.contains(&pattern.to_lowercase()))
	}

	/// Adds a URL pattern to the protected list. Returns true if added.
	pub fn add_protected(&mut self, pattern: String) -> bool {
		if self.no_save || self.no_context {
			return false;
		}
		let Some(selected) = self.selected.as_mut() else {
			return false;
		};
		let pattern_lower = pattern.to_lowercase();
		if selected
			.data
			.protected_urls
			.iter()
			.any(|p| p.to_lowercase() == pattern_lower)
		{
			return false;
		}
		selected.data.protected_urls.push(pattern);
		true
	}

	/// Removes a URL pattern from the protected list. Returns true if removed.
	pub fn remove_protected(&mut self, pattern: &str) -> bool {
		if self.no_save || self.no_context {
			return false;
		}
		let Some(selected) = self.selected.as_mut() else {
			return false;
		};
		let pattern_lower = pattern.to_lowercase();
		let before_len = selected.data.protected_urls.len();
		selected
			.data
			.protected_urls
			.retain(|p| p.to_lowercase() != pattern_lower);
		selected.data.protected_urls.len() < before_len
	}

	pub fn resolve_output(&self, ctx: &CommandContext, provided: Option<PathBuf>) -> PathBuf {
		if let Some(output) = provided {
			return ctx.screenshot_path(&output);
		}

		if !self.no_context && !self.refresh {
			if let Some(last) = self
				.selected
				.as_ref()
				.and_then(|s| s.data.last_output.as_ref())
			{
				return ctx.screenshot_path(Path::new(last));
			}
		}

		ctx.screenshot_path(Path::new("screenshot.png"))
	}

	/// Applies context changes from command execution.
	pub fn apply_delta(&mut self, delta: crate::commands::def::ContextDelta) {
		if self.no_save || self.no_context {
			return;
		}
		let Some(selected) = self.selected.as_mut() else {
			return;
		};
		if let Some(url) = delta.url {
			selected.data.last_url = Some(url);
		}
		if let Some(selector) = delta.selector {
			selected.data.last_selector = Some(selector);
		}
		if let Some(output) = delta.output {
			selected.data.last_output = Some(output.to_string_lossy().to_string());
		}
		selected.data.last_used_at = Some(now_ts());
	}

	/// Records context from a resolved target.
	pub fn record_from_target(
		&mut self,
		target: &crate::target::ResolvedTarget,
		selector: Option<&str>,
	) {
		self.apply_delta(crate::commands::def::ContextDelta {
			url: target.url_str().map(String::from),
			selector: selector.map(String::from),
			output: None,
		});
	}

	/// Persists the current context state to disk.
	pub fn persist(&mut self) -> Result<()> {
		if self.no_save || self.no_context {
			return Ok(());
		}

		let Some(selected) = &self.selected else {
			return Ok(());
		};

		match selected.scope {
			ContextScope::Project => {
				if let Some(store) = self.stores.project.as_mut() {
					*store.ensure(&selected.name, self.project_root.as_deref()) =
						selected.data.clone();
				}
				if let Some(root) = &self.project_root {
					self.stores
						.global
						.file
						.active
						.projects
						.insert(root.to_string_lossy().to_string(), selected.name.clone());
				}
			}
			ContextScope::Global => {
				*self
					.stores
					.global
					.ensure(&selected.name, self.project_root.as_deref()) = selected.data.clone();
				self.stores.global.file.active.global = Some(selected.name.clone());
			}
		}

		self.stores.global.save()?;
		if let Some(store) = &self.stores.project {
			store.save()?;
		}
		Ok(())
	}

	/// Returns the effective base URL.
	pub fn base_url(&self) -> Option<&str> {
		self.base_url_override.as_deref().or_else(|| {
			self.selected
				.as_ref()
				.and_then(|c| c.data.base_url.as_deref())
		})
	}

	#[cfg(test)]
	pub(crate) fn selected(&self) -> Option<&SelectedContext> {
		self.selected.as_ref()
	}
}

/// Selects a context by name, falling back to project-active, global-active, or "default".
fn select_context(
	stores: &mut ContextBook,
	project_root: Option<&Path>,
	requested: Option<&str>,
) -> Option<SelectedContext> {
	let name = requested
		.map(String::from)
		.or_else(|| {
			project_root.and_then(|root| {
				stores
					.global
					.file
					.active
					.projects
					.get(&root.to_string_lossy().to_string())
					.cloned()
			})
		})
		.or_else(|| stores.global.file.active.global.clone())
		.unwrap_or_else(|| {
			stores.global.file.active.global = Some("default".to_string());
			"default".to_string()
		});

	Some(resolve_context_by_name(stores, project_root, &name))
}

/// Resolves a context by name from project store first, then global, creating if needed.
fn resolve_context_by_name(
	stores: &mut ContextBook,
	project_root: Option<&Path>,
	name: &str,
) -> SelectedContext {
	if let Some(store) = stores.project.as_mut() {
		if let Some(data) = store.get(name).cloned() {
			return SelectedContext {
				name: name.to_string(),
				scope: ContextScope::Project,
				data,
			};
		}
	}

	if let Some(data) = stores.global.get(name).cloned() {
		return SelectedContext {
			name: name.to_string(),
			scope: ContextScope::Global,
			data,
		};
	}

	if let Some(store) = stores.project.as_mut() {
		let data = store.ensure(name, project_root).clone();
		return SelectedContext {
			name: name.to_string(),
			scope: ContextScope::Project,
			data,
		};
	}

	let data = stores.global.ensure(name, project_root).clone();
	SelectedContext {
		name: name.to_string(),
		scope: ContextScope::Global,
		data,
	}
}

fn global_store_path() -> PathBuf {
	std::env::var_os("XDG_CONFIG_HOME")
		.map(PathBuf::from)
		.or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
		.unwrap_or_else(|| PathBuf::from("."))
		.join("pw/cli/contexts.json")
}

fn global_sessions_dir() -> PathBuf {
	global_store_path().with_file_name("sessions")
}

fn project_store_path(root: &Path) -> PathBuf {
	root.join(dirs::PLAYWRIGHT).join(".pw-cli/contexts.json")
}

fn project_sessions_dir(root: &Path) -> PathBuf {
	root.join(dirs::PLAYWRIGHT).join(".pw-cli/sessions")
}

fn now_ts() -> u64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_secs()
}

fn is_session_stale(selected: Option<&SelectedContext>) -> bool {
	selected
		.and_then(|ctx| ctx.data.last_used_at)
		.is_some_and(|last_used| now_ts().saturating_sub(last_used) > SESSION_TIMEOUT_SECS)
}
