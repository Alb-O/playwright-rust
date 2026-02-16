//! Page command flow helpers.
//!
//! Consolidates the repetitive command skeleton used by page-oriented commands:
//! build navigation plan, acquire a session, run command-specific browser logic,
//! and close/cleanup session through [`crate::session_helpers::with_session`].

use std::future::Future;
use std::pin::Pin;

use pw_rs::WaitUntil;

use crate::commands::def::ExecCtx;
use crate::commands::exec_flow::navigation_plan;
use crate::error::Result;
use crate::session::SessionHandle;
use crate::session_helpers::{ArtifactsPolicy, with_session};
use crate::target::{ResolvedTarget, Target};

/// Runtime values shared by page-flow command callbacks.
#[derive(Debug, Clone)]
pub struct PageFlowCtx {
	pub timeout_ms: Option<u64>,
	pub target: Target,
}

/// Execute shared page command flow and run command-specific browser logic.
pub async fn run_page_flow<'exec, 'ctx, T>(
	exec: &mut ExecCtx<'exec, 'ctx>,
	resolved_target: &ResolvedTarget,
	wait_until: WaitUntil,
	artifacts: ArtifactsPolicy,
	run: impl for<'s> FnOnce(&'s SessionHandle, PageFlowCtx) -> Pin<Box<dyn Future<Output = Result<T>> + 's>>,
) -> Result<T>
where
	'ctx: 'exec,
{
	let plan = navigation_plan(exec.ctx, exec.last_url, resolved_target, wait_until);
	let flow_ctx = PageFlowCtx {
		timeout_ms: plan.timeout_ms,
		target: plan.target,
	};

	with_session(exec, plan.request, artifacts, move |session| run(session, flow_ctx)).await
}
