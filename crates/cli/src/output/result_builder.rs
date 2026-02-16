use std::io::{self, Write};
use std::time::Instant;

use serde::Serialize;

use crate::output::format::OutputFormat;
use crate::output::model::{
	Artifact, CommandError, CommandInputs, CommandResult, Diagnostic, DiagnosticLevel, EffectiveConfig, ErrorCode, FailureWithArtifacts, SCHEMA_VERSION,
};

/// Builder for constructing command results.
pub struct ResultBuilder<T: Serialize> {
	schema_version: Option<u32>,
	command: String,
	inputs: Option<CommandInputs>,
	data: Option<T>,
	error: Option<CommandError>,
	start_time: Option<Instant>,
	duration_ms: Option<u64>,
	artifacts: Vec<Artifact>,
	diagnostics: Vec<Diagnostic>,
	config: Option<EffectiveConfig>,
}

impl<T: Serialize> ResultBuilder<T> {
	pub fn new(command: impl Into<String>) -> Self {
		Self {
			schema_version: Some(SCHEMA_VERSION),
			command: command.into(),
			inputs: None,
			data: None,
			error: None,
			start_time: Some(Instant::now()),
			duration_ms: None,
			artifacts: Vec::new(),
			diagnostics: Vec::new(),
			config: None,
		}
	}

	pub fn schema_version(mut self, version: u32) -> Self {
		self.schema_version = Some(version);
		self
	}

	pub fn no_schema_version(mut self) -> Self {
		self.schema_version = None;
		self
	}

	pub fn inputs(mut self, inputs: CommandInputs) -> Self {
		self.inputs = Some(inputs);
		self
	}

	pub fn data(mut self, data: T) -> Self {
		self.data = Some(data);
		self
	}

	pub fn error(mut self, code: ErrorCode, message: impl Into<String>) -> Self {
		self.error = Some(CommandError {
			code,
			message: message.into(),
			details: None,
		});
		self
	}

	pub fn error_with_details(mut self, code: ErrorCode, message: impl Into<String>, details: serde_json::Value) -> Self {
		self.error = Some(CommandError {
			code,
			message: message.into(),
			details: Some(details),
		});
		self
	}

	pub fn artifact(mut self, artifact: Artifact) -> Self {
		self.artifacts.push(artifact);
		self
	}

	pub fn diagnostic(mut self, level: DiagnosticLevel, message: impl Into<String>) -> Self {
		self.diagnostics.push(Diagnostic {
			level,
			message: message.into(),
			source: None,
		});
		self
	}

	pub fn diagnostic_with_source(mut self, level: DiagnosticLevel, message: impl Into<String>, source: impl Into<String>) -> Self {
		self.diagnostics.push(Diagnostic {
			level,
			message: message.into(),
			source: Some(source.into()),
		});
		self
	}

	pub fn config(mut self, config: EffectiveConfig) -> Self {
		self.config = Some(config);
		self
	}

	pub fn duration_ms(mut self, duration_ms: u64) -> Self {
		self.duration_ms = Some(duration_ms);
		self
	}

	pub fn build(self) -> CommandResult<T> {
		let ok = self.error.is_none() && self.data.is_some();
		let duration_ms = self.duration_ms.or_else(|| self.start_time.map(|start| start.elapsed().as_millis() as u64));

		CommandResult {
			schema_version: self.schema_version,
			ok,
			command: self.command,
			inputs: self.inputs,
			data: self.data,
			error: self.error,
			duration_ms,
			artifacts: self.artifacts,
			diagnostics: self.diagnostics,
			config: self.config,
		}
	}
}

/// Print a command result to stdout in the specified format.
pub fn print_result<T: Serialize>(result: &CommandResult<T>, format: OutputFormat) {
	match format {
		OutputFormat::Toon => {
			if let Ok(json_value) = serde_json::to_value(result) {
				println!("{}", toon::encode(&json_value, None));
			}
		}
		OutputFormat::Json => {
			if let Ok(json) = serde_json::to_string_pretty(result) {
				println!("{json}");
			}
		}
		OutputFormat::Ndjson => {
			if let Ok(json) = serde_json::to_string(result) {
				println!("{json}");
			}
		}
		OutputFormat::Text => {
			print_result_text(result);
		}
	}
}

fn print_result_text<T: Serialize>(result: &CommandResult<T>) {
	let mut stdout = io::stdout().lock();

	if result.ok {
		if let Some(ref data) = result.data {
			if let Ok(json) = serde_json::to_string_pretty(data) {
				let _ = writeln!(stdout, "{json}");
			}
		}
	} else if let Some(ref error) = result.error {
		let _ = writeln!(stdout, "Error [{}]: {}", error.code, error.message);
		if let Some(ref details) = error.details {
			if let Ok(json) = serde_json::to_string_pretty(details) {
				let _ = writeln!(stdout, "Details: {json}");
			}
		}
	}

	for diag in &result.diagnostics {
		let prefix = match diag.level {
			DiagnosticLevel::Info => "info",
			DiagnosticLevel::Warning => "warning",
			DiagnosticLevel::Error => "error",
		};
		if let Some(ref source) = diag.source {
			let _ = writeln!(stdout, "[{prefix}:{source}] {}", diag.message);
		} else {
			let _ = writeln!(stdout, "[{prefix}] {}", diag.message);
		}
	}

	for artifact in &result.artifacts {
		let _ = writeln!(stdout, "Saved {:?}: {}", artifact.artifact_type, artifact.path.display());
	}

	if let Some(duration_ms) = result.duration_ms {
		let _ = writeln!(stdout, "Completed in {duration_ms}ms");
	}
}

/// Print an error to stderr in human-readable format.
pub fn print_error_stderr(error: &CommandError) {
	eprintln!("Error [{}]: {}", error.code, error.message);
}

/// Print a failure result with artifacts to stdout.
pub fn print_failure_with_artifacts(command: &str, failure: &FailureWithArtifacts, format: OutputFormat) {
	let result: CommandResult<()> = ResultBuilder::new(command).error(failure.error.code, &failure.error.message).build();

	let result_with_artifacts = CommandResult {
		schema_version: result.schema_version,
		ok: false,
		command: result.command,
		inputs: result.inputs,
		data: None::<()>,
		error: Some(failure.error.clone()),
		duration_ms: result.duration_ms,
		artifacts: failure.artifacts.clone(),
		diagnostics: result.diagnostics,
		config: result.config,
	};

	print_result(&result_with_artifacts, format);
}
