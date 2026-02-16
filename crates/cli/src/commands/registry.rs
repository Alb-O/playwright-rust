//! Command registry and generated dispatch glue.

use crate::output::{CommandInputs, OutputFormat, ResultBuilder, print_result};

pub use crate::commands::catalog::{CommandId, command_name, lookup_command, run_command};

/// Print success result in the given format.
pub fn emit_success(command: &'static str, inputs: CommandInputs, data: serde_json::Value, format: OutputFormat) {
	let result = ResultBuilder::new(command).inputs(inputs).data(data).build();
	print_result(&result, format);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn lookup_command_by_primary_name() {
		assert_eq!(lookup_command("navigate"), Some(CommandId::Navigate));
		assert_eq!(lookup_command("click"), Some(CommandId::Click));
		assert_eq!(lookup_command("page.text"), Some(CommandId::PageText));
		assert_eq!(lookup_command("connect"), Some(CommandId::Connect));
		assert_eq!(lookup_command("session.status"), Some(CommandId::SessionStatus));
		assert_eq!(lookup_command("har.show"), Some(CommandId::HarShow));
	}

	#[test]
	fn lookup_command_by_alias() {
		assert_eq!(lookup_command("nav"), Some(CommandId::Navigate));
		assert_eq!(lookup_command("ss"), Some(CommandId::Screenshot));
	}

	#[test]
	fn lookup_command_unknown_returns_none() {
		assert_eq!(lookup_command("unknown"), None);
		assert_eq!(lookup_command(""), None);
		assert_eq!(lookup_command("navigat"), None);
	}

	#[test]
	fn command_name_returns_primary() {
		assert_eq!(command_name(CommandId::Navigate), "navigate");
		assert_eq!(command_name(CommandId::Screenshot), "screenshot");
		assert_eq!(command_name(CommandId::PageText), "page.text");
		assert_eq!(command_name(CommandId::Connect), "connect");
		assert_eq!(command_name(CommandId::SessionStatus), "session.status");
		assert_eq!(command_name(CommandId::HarShow), "har.show");
	}
}
