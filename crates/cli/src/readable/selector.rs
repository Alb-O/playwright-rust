//! Selector-based content extraction helpers.

use std::sync::LazyLock;

use regex_lite::Regex;

pub(crate) fn try_extract_by_selector(html: &str, selector: &str) -> Option<String> {
	if let Some(id) = selector.strip_prefix('#') {
		for tag in ["div", "article", "section", "main", "aside"] {
			let pattern = format!(r#"(?is)<{tag}[^>]*id=["']{id}["'][^>]*>(.*?)</{tag}>"#, tag = tag, id = regex_lite::escape(id));
			if let Ok(re) = Regex::new(&pattern) {
				if let Some(caps) = re.captures(html) {
					if let Some(m) = caps.get(1) {
						return Some(m.as_str().to_string());
					}
				}
			}
		}
		None
	} else if let Some(class) = selector.strip_prefix('.') {
		for tag in ["div", "article", "section", "main", "aside"] {
			let pattern = format!(
				r#"(?is)<{tag}[^>]*class=["'][^"']*\b{class}\b[^"']*["'][^>]*>(.*?)</{tag}>"#,
				tag = tag,
				class = regex_lite::escape(class)
			);
			if let Ok(re) = Regex::new(&pattern) {
				if let Some(caps) = re.captures(html) {
					if let Some(m) = caps.get(1) {
						return Some(m.as_str().to_string());
					}
				}
			}
		}
		None
	} else if selector.starts_with('[') && selector.contains("role=") {
		if let Some(role) = selector.strip_prefix("[role=\"").and_then(|s| s.strip_suffix("\"]")) {
			for tag in ["div", "article", "section", "main", "aside"] {
				let pattern = format!(
					r#"(?is)<{tag}[^>]*role=["']{role}["'][^>]*>(.*?)</{tag}>"#,
					tag = tag,
					role = regex_lite::escape(role)
				);
				if let Ok(re) = Regex::new(&pattern) {
					if let Some(caps) = re.captures(html) {
						if let Some(m) = caps.get(1) {
							return Some(m.as_str().to_string());
						}
					}
				}
			}
		}
		None
	} else if selector.chars().all(|c| c.is_alphanumeric()) {
		let pattern = format!(r#"(?is)<{0}[^>]*>(.*?)</{0}>"#, selector);
		if let Ok(re) = Regex::new(&pattern) {
			if let Some(caps) = re.captures(html) {
				if let Some(m) = caps.get(1) {
					return Some(m.as_str().to_string());
				}
			}
		}
		None
	} else {
		None
	}
}

pub(crate) fn extract_body(html: &str) -> Option<String> {
	static BODY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<body[^>]*>(.*)</body>").expect("BODY_RE should compile"));

	BODY_RE.captures(html).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn extracts_by_id_selector() {
		let html = "<main><div id='article'>content</div></main>";
		assert_eq!(try_extract_by_selector(html, "#article"), Some("content".to_string()));
	}

	#[test]
	fn extracts_body_when_present() {
		let html = "<html><body><p>Body text</p></body></html>";
		assert_eq!(extract_body(html), Some("<p>Body text</p>".to_string()));
	}
}
