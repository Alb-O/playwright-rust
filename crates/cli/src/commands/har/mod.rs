use std::path::PathBuf;

use serde_json::json;

use crate::cli::{CliHarContentPolicy, CliHarMode};
use crate::context_store::ContextState;
use crate::context_store::types::HarDefaults;
use crate::error::Result;
use crate::output::{OutputFormat, ResultBuilder, print_result};

pub fn set(
	ctx_state: &mut ContextState,
	format: OutputFormat,
	file: PathBuf,
	content: CliHarContentPolicy,
	mode: CliHarMode,
	omit_content: bool,
	url_filter: Option<String>,
) -> Result<()> {
	let har = HarDefaults {
		path: file,
		content_policy: content.into(),
		mode: mode.into(),
		omit_content,
		url_filter,
	};
	let changed = ctx_state.set_har_defaults(har.clone());

	let result = ResultBuilder::new("har set")
		.data(json!({
			"enabled": true,
			"changed": changed,
			"har": har_payload(&har),
		}))
		.build();

	print_result(&result, format);
	Ok(())
}

pub fn show(ctx_state: &ContextState, format: OutputFormat) -> Result<()> {
	let har = ctx_state.har_defaults();
	let result = ResultBuilder::new("har show")
		.data(json!({
			"enabled": har.is_some(),
			"har": har.map(har_payload),
		}))
		.build();

	print_result(&result, format);
	Ok(())
}

pub fn clear(ctx_state: &mut ContextState, format: OutputFormat) -> Result<()> {
	let cleared = ctx_state.clear_har_defaults();
	let result = ResultBuilder::new("har clear")
		.data(json!({
			"cleared": cleared,
			"enabled": ctx_state.har_defaults().is_some(),
		}))
		.build();

	print_result(&result, format);
	Ok(())
}

fn har_payload(har: &HarDefaults) -> serde_json::Value {
	json!({
		"path": har.path,
		"contentPolicy": har.content_policy,
		"mode": har.mode,
		"omitContent": har.omit_content,
		"urlFilter": har.url_filter,
	})
}
