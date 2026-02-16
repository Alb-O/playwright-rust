//! HTML-to-text rendering for readable output.

use std::sync::LazyLock;

use regex_lite::Regex;

use crate::readable::entities::{collapse_whitespace, decode_html_entities};
use crate::readable::junk::is_junk_line;

static TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").expect("TAG_RE should compile"));

pub(crate) fn html_to_text(html: &str) -> String {
	let mut result = html.to_string();
	for tag in ["p", "div", "br", "h1", "h2", "h3", "h4", "h5", "h6", "li", "tr"] {
		let open_pattern = format!("<{}", tag);
		result = result.replace(&open_pattern, &format!("\n<{}", tag));
	}

	let result = TAG_RE.replace_all(&result, "");
	let result = decode_html_entities(&result);
	let result = collapse_whitespace(&result);

	result
		.lines()
		.map(|l| l.trim())
		.filter(|l| !l.is_empty() && !is_junk_line(l))
		.collect::<Vec<_>>()
		.join("\n")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn converts_html_to_text() {
		let html = "<p>Hello <strong>World</strong>!</p>";
		let text = html_to_text(html);
		assert!(text.contains("Hello"));
		assert!(text.contains("World"));
	}
}
