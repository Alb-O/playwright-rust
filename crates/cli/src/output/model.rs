use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Current schema version for command output.
pub const SCHEMA_VERSION: u32 = 4;

/// The result envelope returned by all commands.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult<T: Serialize> {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub schema_version: Option<u32>,
	pub ok: bool,
	pub command: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub inputs: Option<CommandInputs>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub data: Option<T>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub error: Option<CommandError>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub duration_ms: Option<u64>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub artifacts: Vec<Artifact>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub diagnostics: Vec<Diagnostic>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub config: Option<EffectiveConfig>,
}

/// Inputs used for a command execution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CommandInputs {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub url: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub selector: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub expression: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub output_path: Option<PathBuf>,
	#[serde(flatten, skip_serializing_if = "Option::is_none")]
	pub extra: Option<serde_json::Value>,
}

/// Error information for failed commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
	pub code: ErrorCode,
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub details: Option<serde_json::Value>,
}

/// Standardized error codes for programmatic handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
	BrowserLaunchFailed,
	NavigationFailed,
	SelectorNotFound,
	SelectorAmbiguous,
	Timeout,
	JsEvalFailed,
	ScreenshotFailed,
	IoError,
	SessionError,
	InvalidInput,
	UnsupportedMode,
	AuthError,
	InternalError,
}

impl std::fmt::Display for ErrorCode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ErrorCode::BrowserLaunchFailed => write!(f, "BROWSER_LAUNCH_FAILED"),
			ErrorCode::NavigationFailed => write!(f, "NAVIGATION_FAILED"),
			ErrorCode::SelectorNotFound => write!(f, "SELECTOR_NOT_FOUND"),
			ErrorCode::SelectorAmbiguous => write!(f, "SELECTOR_AMBIGUOUS"),
			ErrorCode::Timeout => write!(f, "TIMEOUT"),
			ErrorCode::JsEvalFailed => write!(f, "JS_EVAL_FAILED"),
			ErrorCode::ScreenshotFailed => write!(f, "SCREENSHOT_FAILED"),
			ErrorCode::IoError => write!(f, "IO_ERROR"),
			ErrorCode::SessionError => write!(f, "SESSION_ERROR"),
			ErrorCode::InvalidInput => write!(f, "INVALID_INPUT"),
			ErrorCode::UnsupportedMode => write!(f, "UNSUPPORTED_MODE"),
			ErrorCode::AuthError => write!(f, "AUTH_ERROR"),
			ErrorCode::InternalError => write!(f, "INTERNAL_ERROR"),
		}
	}
}

/// Artifact produced by a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
	#[serde(rename = "type")]
	pub artifact_type: ArtifactType,
	pub path: PathBuf,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub size_bytes: Option<u64>,
}

/// Artifact categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactType {
	Screenshot,
	Html,
	Auth,
	Trace,
	Video,
	Download,
}

/// Diagnostic message attached to a command result.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
	pub level: DiagnosticLevel,
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub source: Option<String>,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
	Info,
	Warning,
	Error,
}

/// Where the CDP endpoint was configured.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CdpEndpointSource {
	CliFlag,
	Context,
	#[default]
	None,
}

/// How the browser session was acquired.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionSource {
	Daemon,
	CachedDescriptor,
	#[default]
	Fresh,
	CdpConnect,
	PersistentDebug,
	BrowserServer,
}

/// Effective configuration used for command execution.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveConfig {
	pub browser: String,
	pub headless: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub wait_until: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub timeout_ms: Option<u64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub endpoint: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cdp_endpoint_source: Option<CdpEndpointSource>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub session_source: Option<SessionSource>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub target_source: Option<String>,
}

/// A command failure with collected artifacts.
#[derive(Debug)]
pub struct FailureWithArtifacts {
	pub error: CommandError,
	pub artifacts: Vec<Artifact>,
}

impl FailureWithArtifacts {
	pub fn new(error: CommandError) -> Self {
		Self { error, artifacts: Vec::new() }
	}

	pub fn with_artifacts(mut self, artifacts: Vec<Artifact>) -> Self {
		self.artifacts = artifacts;
		self
	}
}

/// A command result with no payload data.
pub type EmptyResult = CommandResult<()>;

/// A command result with a string payload.
pub type StringResult = CommandResult<String>;
