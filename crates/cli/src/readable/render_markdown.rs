//! HTML-to-markdown rendering for readable output.

use std::sync::LazyLock;

use regex_lite::Regex;

use crate::readable::entities::{collapse_whitespace, decode_html_entities};
use crate::readable::junk::is_junk_line;

static MD_HEADER_OPEN_RES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
	(1..=6)
		.map(|level| Regex::new(&format!(r"(?i)<h{level}\s*[^>]*>")).expect("header open regex should compile"))
		.collect()
});
static MD_STRONG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<strong[^>]*>([^<]*)</strong>").expect("MD_STRONG_RE should compile"));
static MD_B_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<b[^>]*>([^<]*)</b>").expect("MD_B_RE should compile"));
static MD_EM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<em[^>]*>([^<]*)</em>").expect("MD_EM_RE should compile"));
static MD_I_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<i[^>]*>([^<]*)</i>").expect("MD_I_RE should compile"));
static MD_LINK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?i)<a[^>]*href=["']([^"']+)["'][^>]*>([^<]*)</a>"#).expect("MD_LINK_RE should compile"));
static MD_IMG_SRC_ALT_RE: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r#"(?i)<img[^>]*src=["']([^"']+)["'][^>]*alt=["']([^"']*)["'][^>]*/?>"#).expect("MD_IMG_SRC_ALT_RE should compile"));
static MD_IMG_ALT_SRC_RE: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r#"(?i)<img[^>]*alt=["']([^"']*)["'][^>]*src=["']([^"']+)["'][^>]*/?>"#).expect("MD_IMG_ALT_SRC_RE should compile"));
static MD_P_OPEN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<p[^>]*>").expect("MD_P_OPEN_RE should compile"));
static MD_BR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<br\s*/?>").expect("MD_BR_RE should compile"));
static MD_LI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<li[^>]*>").expect("MD_LI_RE should compile"));
static MD_LIST_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)</?[uo]l[^>]*>").expect("MD_LIST_RE should compile"));
static MD_CODE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<code[^>]*>([^<]*)</code>").expect("MD_CODE_RE should compile"));
static MD_PRE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<pre[^>]*>([^<]*)</pre>").expect("MD_PRE_RE should compile"));
static MD_BLOCKQUOTE_OPEN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<blockquote[^>]*>").expect("MD_BLOCKQUOTE_OPEN_RE should compile"));
static MD_ANY_TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").expect("MD_ANY_TAG_RE should compile"));
static EMPTY_HEADER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#{1,6}\s*$").expect("EMPTY_HEADER should compile"));

pub(crate) fn html_to_markdown(html: &str) -> String {
	let mut result = html.to_string();

	for i in 1..=6 {
		let hashes = "#".repeat(i);
		let close = format!("</h{}>", i);
		result = MD_HEADER_OPEN_RES[i - 1].replace_all(&result, &format!("\n{} ", hashes)).to_string();
		result = result.replace(&close, "\n");
	}

	result = MD_STRONG_RE.replace_all(&result, "**$1**").to_string();
	result = MD_B_RE.replace_all(&result, "**$1**").to_string();
	result = MD_EM_RE.replace_all(&result, "*$1*").to_string();
	result = MD_I_RE.replace_all(&result, "*$1*").to_string();
	result = MD_LINK_RE.replace_all(&result, "[$2]($1)").to_string();
	result = MD_IMG_SRC_ALT_RE.replace_all(&result, "![$2]($1)").to_string();
	result = MD_IMG_ALT_SRC_RE.replace_all(&result, "![$1]($2)").to_string();
	result = MD_P_OPEN_RE.replace_all(&result, "\n\n").to_string();
	result = result.replace("</p>", "\n");
	result = MD_BR_RE.replace_all(&result, "\n").to_string();
	result = MD_LI_RE.replace_all(&result, "\n- ").to_string();
	result = result.replace("</li>", "");
	result = MD_LIST_RE.replace_all(&result, "\n").to_string();
	result = MD_CODE_RE.replace_all(&result, "`$1`").to_string();
	result = MD_PRE_RE.replace_all(&result, "\n```\n$1\n```\n").to_string();
	result = MD_BLOCKQUOTE_OPEN_RE.replace_all(&result, "\n> ").to_string();
	result = result.replace("</blockquote>", "\n");
	result = MD_ANY_TAG_RE.replace_all(&result, "").to_string();
	result = decode_html_entities(&result);
	result = collapse_whitespace(&result);

	result
		.lines()
		.map(|l| l.trim())
		.filter(|l| !l.is_empty() && !EMPTY_HEADER.is_match(l) && !is_junk_line(l))
		.collect::<Vec<_>>()
		.join("\n")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn converts_headings_and_links() {
		let html = "<h1>Title</h1><p><a href='https://example.com'>link</a></p>";
		let markdown = html_to_markdown(html);
		assert!(markdown.contains("# Title"));
		assert!(markdown.contains("[link](https://example.com)"));
	}
}
