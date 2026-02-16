//! Backward-compatible re-exports for the decomposed connect subsystem.
//!
//! New code should prefer `crate::session::connect`.

pub use crate::session::connect::{
	CdpVersionInfo, clear_cdp_endpoint, discover_and_connect, fetch_cdp_endpoint, kill_browser_on_port, launch_and_connect, resolve_connect_port,
	resolve_user_data_dir, set_cdp_endpoint, show_cdp_endpoint,
};
