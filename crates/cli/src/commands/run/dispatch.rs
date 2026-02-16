//! Command dispatch for batch execution.

use super::{BatchRequest, BatchResponse};
use crate::commands::def::{ExecCtx, ExecMode};
use crate::commands::registry::{command_name, lookup_command, run_command};
use crate::context::CommandContext;
use crate::context_store::ContextState;
use crate::output::OutputFormat;
use crate::session_broker::SessionBroker;

/// Dispatches a single batch command and returns the response.
///
/// This handles URL/selector resolution from context state, delegates to the
/// appropriate command module, and records state updates on success.
pub async fn execute_batch_command<'ctx>(
	request: &BatchRequest,
	ctx: &'ctx CommandContext,
	ctx_state: &mut ContextState,
	broker: &mut SessionBroker<'ctx>,
	format: OutputFormat,
	schema_version: u32,
) -> BatchResponse {
	let id = request.id.clone();
	let cmd_str = request.command.as_str();
	let has_cdp = ctx.cdp_endpoint().is_some();

	let Some(cmd_id) = lookup_command(cmd_str) else {
		return BatchResponse::error(id, cmd_str, "UNKNOWN_COMMAND", &format!("Unknown command: {}", cmd_str), None, schema_version);
	};

	let last_url = ctx_state.last_url().map(str::to_string);
	let exec = ExecCtx {
		mode: ExecMode::Batch,
		ctx,
		ctx_state,
		broker,
		format,
		artifacts_dir: None,
		last_url: last_url.as_deref(),
	};

	match run_command(cmd_id, request.args.clone(), has_cdp, exec).await {
		Ok(out) => {
			out.delta.apply(ctx_state);
			BatchResponse::success(id, out.command, out.data, schema_version).with_inputs(out.inputs)
		}
		Err(e) => {
			let err = e.to_command_error();
			BatchResponse::error(id, command_name(cmd_id), &err.code.to_string(), &err.message, err.details, schema_version)
		}
	}
}
