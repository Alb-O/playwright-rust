//! Command dispatch for batch execution.

use crate::commands::def::{ExecCtx, ExecMode};
use crate::commands::registry::{command_name, lookup_command, run_command};
use crate::context::CommandContext;
use crate::context_store::ContextState;
use crate::output::OutputFormat;
use crate::session_broker::SessionBroker;
use crate::target::ResolveEnv;

use super::{BatchRequest, BatchResponse};

/// Dispatches a single batch command and returns the response.
///
/// This handles URL/selector resolution from context state, delegates to the
/// appropriate command module, and records state updates on success.
pub async fn execute_batch_command<'ctx>(
	request: &BatchRequest,
	ctx: &'ctx CommandContext,
	ctx_state: &mut ContextState,
	broker: &mut SessionBroker<'ctx>,
) -> BatchResponse {
	let id = request.id.clone();
	let cmd_str = request.command.as_str();
	let has_cdp = ctx.cdp_endpoint().is_some();

	let Some(cmd_id) = lookup_command(cmd_str) else {
		return BatchResponse::error(
			id,
			cmd_str,
			"UNKNOWN_COMMAND",
			&format!("Unknown command: {}", cmd_str),
		);
	};

	let canonical = command_name(cmd_id);
	let env = ResolveEnv::new(ctx_state, has_cdp, canonical);
	let last_url = ctx_state.last_url().map(str::to_string);
	let exec = ExecCtx {
		mode: ExecMode::Batch,
		ctx,
		broker,
		format: OutputFormat::Ndjson,
		artifacts_dir: None,
		last_url: last_url.as_deref(),
	};

	match run_command(cmd_id, request.args.clone(), &env, exec).await {
		Ok(out) => {
			out.delta.apply(ctx_state);
			BatchResponse::success(id, cmd_str, out.data).with_inputs(out.inputs)
		}
		Err(e) => BatchResponse::error(id, cmd_str, "COMMAND_FAILED", &e.to_string()),
	}
}
