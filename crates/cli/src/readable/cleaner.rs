//! Clutter removal and main-content extraction.

use std::collections::HashSet;

use regex_lite::Regex;

use crate::readable::config::{clutter, partial_pattern_regex};
use crate::readable::entities::collapse_whitespace;
use crate::readable::selector::{extract_body, try_extract_by_selector};

pub(crate) fn remove_clutter(html: &str) -> String {
	let mut result = html.to_string();

	result = remove_tags(&result, &["script", "style", "noscript", "svg"]);
	result = remove_tags(
		&result,
		&["nav", "header", "footer", "aside", "form", "button", "input", "select", "textarea", "iframe"],
	);
	result = remove_elements_by_attribute(&result);
	result = extract_main_content(&result);

	collapse_whitespace(&result).trim().to_string()
}

pub(crate) fn remove_tags(html: &str, tags: &[&str]) -> String {
	let mut result = html.to_string();
	for tag in tags {
		let pattern = format!(r"(?is)<{0}[^>]*>.*?</{0}>|<{0}[^>]*/?>", tag);
		if let Ok(re) = Regex::new(&pattern) {
			result = re.replace_all(&result, "").to_string();
		}
	}
	result
}

fn remove_elements_by_attribute(html: &str) -> String {
	let non_content: HashSet<&str> = clutter().scoring.non_content_patterns.iter().map(|s| s.as_str()).collect();

	let mut result = html.to_string();
	for tag in ["div", "section", "aside", "span", "ul", "ol", "article"] {
		let element_pattern = format!(r#"(?is)<{tag}[^>]*(class|id)=["']([^"']+)["'][^>]*>.*?</{tag}>"#, tag = tag);

		if let Ok(element_re) = Regex::new(&element_pattern) {
			for _ in 0..3 {
				let mut changed = false;
				let new_result = element_re.replace_all(&result, |caps: &regex_lite::Captures| {
					if let Some(attr_value) = caps.get(2) {
						let attr_lower = attr_value.as_str().to_lowercase();
						let should_remove = non_content.iter().any(|p| attr_lower.contains(p)) || partial_pattern_regex().is_match(&attr_lower);
						if should_remove {
							changed = true;
							return String::new();
						}
					}
					caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default()
				});
				result = new_result.into_owned();
				if !changed {
					break;
				}
			}
		}
	}

	result
}

fn extract_main_content(html: &str) -> String {
	for selector in &clutter().content_selectors.selectors {
		if let Some(content) = try_extract_by_selector(html, selector) {
			if content.len() > 100 {
				return content;
			}
		}
	}

	if let Some(body) = extract_body(html) {
		return body;
	}

	html.to_string()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn removes_script_tags() {
		let html = "<div>Before<script>alert('x');</script>After</div>";
		let cleaned = remove_tags(html, &["script"]);
		assert!(!cleaned.contains("alert"));
		assert!(cleaned.contains("Before"));
		assert!(cleaned.contains("After"));
	}

	#[test]
	fn extracts_main_article_over_header_navigation() {
		let html = "<body><header>Top Nav</header><article><p>This is main content that is intentionally long enough to pass the extraction threshold and remain selected.</p></article></body>";
		let cleaned = remove_clutter(html);
		assert!(cleaned.contains("main content"));
		assert!(!cleaned.contains("Top Nav"));
	}
}
