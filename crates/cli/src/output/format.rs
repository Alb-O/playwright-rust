use clap::ValueEnum;

/// Output format for CLI results.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
	/// TOON output (default, token-efficient for LLMs)
	#[default]
	Toon,
	/// JSON output
	Json,
	/// Newline-delimited JSON (streaming)
	Ndjson,
	/// Human-readable text
	Text,
}

impl std::str::FromStr for OutputFormat {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"toon" => Ok(OutputFormat::Toon),
			"json" => Ok(OutputFormat::Json),
			"ndjson" => Ok(OutputFormat::Ndjson),
			"text" => Ok(OutputFormat::Text),
			_ => Err(format!("unknown format: {s}")),
		}
	}
}

impl std::fmt::Display for OutputFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			OutputFormat::Toon => write!(f, "toon"),
			OutputFormat::Json => write!(f, "json"),
			OutputFormat::Ndjson => write!(f, "ndjson"),
			OutputFormat::Text => write!(f, "text"),
		}
	}
}
