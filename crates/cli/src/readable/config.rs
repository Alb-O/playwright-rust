//! Pattern configuration loaded from `clutter.json`.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex_lite::Regex;
use serde::Deserialize;

static CLUTTER: LazyLock<ClutterPatterns> = LazyLock::new(|| {
	let json = include_str!("../clutter.json");
	serde_json::from_str(json).expect("Failed to parse clutter.json")
});

static PARTIAL_PATTERN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	let all_patterns: Vec<String> = CLUTTER
		.remove
		.partial_patterns
		.patterns
		.values()
		.flatten()
		.map(|s| regex_lite::escape(s))
		.collect();

	if all_patterns.is_empty() {
		Regex::new(r"(?!.*)").expect("partial pattern fallback regex should compile")
	} else {
		let pattern = all_patterns.join("|");
		Regex::new(&format!("(?i){}", pattern)).expect("combined partial pattern regex should compile")
	}
});

pub(crate) fn clutter() -> &'static ClutterPatterns {
	&CLUTTER
}

pub(crate) fn partial_pattern_regex() -> &'static Regex {
	&PARTIAL_PATTERN_REGEX
}

#[allow(dead_code, reason = "fields are loaded from JSON and reserved for future heuristics")]
#[derive(Debug, Deserialize)]
pub(crate) struct ClutterPatterns {
	pub(crate) content_selectors: ContentSelectors,
	pub(crate) remove: RemovePatterns,
	pub(crate) preserve: PreservePatterns,
	pub(crate) scoring: ScoringPatterns,
	pub(crate) junk_text: JunkTextPatterns,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContentSelectors {
	pub(crate) selectors: Vec<String>,
}

#[allow(dead_code, reason = "fields are loaded from JSON and reserved for future heuristics")]
#[derive(Debug, Deserialize)]
pub(crate) struct RemovePatterns {
	pub(crate) exact_selectors: Vec<String>,
	pub(crate) partial_patterns: PartialPatterns,
}

#[allow(dead_code, reason = "fields are loaded from JSON and reserved for future heuristics")]
#[derive(Debug, Deserialize)]
pub(crate) struct PartialPatterns {
	pub(crate) check_attributes: Vec<String>,
	pub(crate) patterns: HashMap<String, Vec<String>>,
}

#[allow(dead_code, reason = "fields are loaded from JSON and reserved for future heuristics")]
#[derive(Debug, Deserialize)]
pub(crate) struct PreservePatterns {
	pub(crate) preserve_elements: Vec<String>,
	pub(crate) inline_elements: Vec<String>,
	pub(crate) allowed_empty: Vec<String>,
	pub(crate) allowed_attributes: Vec<String>,
}

#[allow(dead_code, reason = "fields are loaded from JSON and reserved for future heuristics")]
#[derive(Debug, Deserialize)]
pub(crate) struct ScoringPatterns {
	pub(crate) content_indicators: Vec<String>,
	pub(crate) navigation_indicators: Vec<String>,
	pub(crate) non_content_patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JunkTextPatterns {
	pub(crate) exact: Vec<String>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn clutter_patterns_load() {
		assert!(!clutter().content_selectors.selectors.is_empty());
		assert!(!clutter().remove.exact_selectors.is_empty());
		assert!(!clutter().scoring.non_content_patterns.is_empty());
		assert!(!clutter().junk_text.exact.is_empty());
	}

	#[test]
	fn partial_pattern_regex_compiles() {
		assert!(partial_pattern_regex().is_match("promo-banner"));
	}
}
