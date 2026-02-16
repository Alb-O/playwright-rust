//! Daemon browser lease acquisition helpers.

use tracing::debug;

use super::spec::SessionRequest;
use crate::daemon;
use crate::error::Result;

/// Active daemon lease metadata used for descriptor persistence and session attach.
#[derive(Debug, Clone)]
pub(super) struct DaemonLease {
	pub(super) endpoint: String,
	pub(super) session_key: String,
}

/// Attempts to acquire a daemon-provided browser endpoint for this request.
pub(super) async fn acquire_daemon_lease(namespace_id: Option<&str>, request: &SessionRequest<'_>, try_daemon_lease: bool) -> Result<Option<DaemonLease>> {
	if !try_daemon_lease {
		return Ok(None);
	}

	let Some(client) = daemon::try_connect().await else {
		return Ok(None);
	};

	let Some(namespace_id) = namespace_id else {
		return Ok(None);
	};

	let session_key = format!("{}:{}:{}", namespace_id, request.browser, if request.headless { "headless" } else { "headful" });
	match daemon::request_browser(&client, request.browser, request.headless, &session_key).await {
		Ok(endpoint) => {
			debug!(
				target = "pw.session",
				%endpoint,
				session_key = %session_key,
				"using daemon browser"
			);
			Ok(Some(DaemonLease { endpoint, session_key }))
		}
		Err(err) => {
			debug!(
				target = "pw.session",
				error = %err,
				"daemon request failed; falling back"
			);
			Ok(None)
		}
	}
}
