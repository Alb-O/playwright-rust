//! High-level connect/session orchestration over CDP endpoint state.

use std::path::Path;

use serde_json::Value;

use super::connect::{clear_cdp_endpoint, discover_and_connect, kill_browser_on_port, launch_and_connect, set_cdp_endpoint, show_cdp_endpoint};
use crate::context_store::ContextState;
use crate::error::Result;

/// Service facade used by protocol/CLI commands for connect flows.
pub struct ConnectService<'a> {
	ctx_state: &'a mut ContextState,
	auth_file: Option<&'a Path>,
}

impl<'a> ConnectService<'a> {
	/// Creates a service for the current mutable context state.
	pub fn new(ctx_state: &'a mut ContextState, auth_file: Option<&'a Path>) -> Self {
		Self { ctx_state, auth_file }
	}

	/// Kills a browser bound to `port` and clears stored endpoint when found.
	pub async fn kill(&mut self, port: u16) -> Result<Value> {
		kill_browser_on_port(self.ctx_state, port).await
	}

	/// Clears stored endpoint metadata.
	pub fn clear(&mut self) -> Value {
		clear_cdp_endpoint(self.ctx_state)
	}

	/// Launches browser and stores discovered endpoint.
	pub async fn launch(&mut self, port: u16, user_data_dir: Option<&Path>) -> Result<Value> {
		launch_and_connect(self.ctx_state, port, user_data_dir, self.auth_file).await
	}

	/// Discovers an existing debug browser and stores endpoint.
	pub async fn discover(&mut self, port: u16) -> Result<Value> {
		discover_and_connect(self.ctx_state, port, self.auth_file).await
	}

	/// Stores explicit endpoint.
	pub fn set_endpoint(&mut self, endpoint: &str) -> Value {
		set_cdp_endpoint(self.ctx_state, endpoint)
	}

	/// Shows stored endpoint metadata.
	pub fn show(&self) -> Value {
		show_cdp_endpoint(self.ctx_state)
	}
}
