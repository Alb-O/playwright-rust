//! Shared text cleanup helpers.

use std::sync::LazyLock;

use regex_lite::Regex;

static MULTI_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t]+").expect("MULTI_SPACE regex should compile"));
static MULTI_NEWLINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{2,}").expect("MULTI_NEWLINE regex should compile"));

/// Decode a small set of HTML entities used in readable extraction.
pub(crate) fn decode_html_entities(s: &str) -> String {
	s.replace("&amp;", "&")
		.replace("&lt;", "<")
		.replace("&gt;", ">")
		.replace("&quot;", "\"")
		.replace("&#39;", "'")
		.replace("&apos;", "'")
		.replace("&#x27;", "'")
		.replace("&nbsp;", " ")
}

/// Collapse runs of spaces and blank lines.
pub(crate) fn collapse_whitespace(s: &str) -> String {
	let result = MULTI_SPACE.replace_all(s, " ");
	MULTI_NEWLINE.replace_all(&result, "\n").to_string()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn decodes_known_entities() {
		assert_eq!(decode_html_entities("&amp;"), "&");
		assert_eq!(decode_html_entities("&lt;"), "<");
		assert_eq!(decode_html_entities("Hello&nbsp;World"), "Hello World");
	}
}
