//! Metadata extraction from source HTML.

use std::sync::LazyLock;

use regex_lite::Regex;

use crate::readable::entities::decode_html_entities;
use crate::readable::types::PageMetadata;

static META_TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<meta\b[^>]*>").expect("META_TAG_RE should compile"));
static META_ATTR_RE: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r#"(?i)([a-zA-Z_:][-a-zA-Z0-9_:.]*)\s*=\s*["']([^"']*)["']"#).expect("META_ATTR_RE should compile"));
static TITLE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<title[^>]*>([^<]+)</title>").expect("TITLE_RE should compile"));

pub(crate) fn extract_metadata(html: &str, url: Option<&str>) -> PageMetadata {
	let site = extract_meta_content(html, "og:site_name")
		.or_else(|| extract_meta_content(html, "twitter:site"))
		.or_else(|| url.and_then(extract_domain));

	PageMetadata {
		title: extract_meta_content(html, "og:title")
			.or_else(|| extract_meta_content(html, "twitter:title"))
			.or_else(|| extract_title_tag(html)),
		author: extract_meta_content(html, "author").or_else(|| extract_meta_content(html, "article:author")),
		description: extract_meta_content(html, "og:description")
			.or_else(|| extract_meta_content(html, "description"))
			.or_else(|| extract_meta_content(html, "twitter:description")),
		image: extract_meta_content(html, "og:image").or_else(|| extract_meta_content(html, "twitter:image")),
		site,
		published: extract_meta_content(html, "article:published_time").or_else(|| extract_meta_content(html, "datePublished")),
	}
}

fn extract_meta_content(html: &str, name: &str) -> Option<String> {
	let target = name.to_ascii_lowercase();
	for meta_tag in META_TAG_RE.find_iter(html) {
		let mut property = None;
		let mut named = None;
		let mut content = None;

		for caps in META_ATTR_RE.captures_iter(meta_tag.as_str()) {
			let Some(key) = caps.get(1).map(|m| m.as_str()) else {
				continue;
			};
			let Some(value) = caps.get(2).map(|m| m.as_str()) else {
				continue;
			};

			match key.to_ascii_lowercase().as_str() {
				"property" => property = Some(value),
				"name" => named = Some(value),
				"content" => content = Some(value),
				_ => {}
			}
		}

		let Some(content_value) = content else {
			continue;
		};
		let matched =
			property.is_some_and(|value| value.eq_ignore_ascii_case(target.as_str())) || named.is_some_and(|value| value.eq_ignore_ascii_case(target.as_str()));
		if matched {
			return Some(decode_html_entities(content_value));
		}
	}

	None
}

fn extract_title_tag(html: &str) -> Option<String> {
	TITLE_RE.captures(html).and_then(|c| c.get(1)).map(|m| decode_html_entities(m.as_str().trim()))
}

fn extract_domain(url: &str) -> Option<String> {
	let url = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
	let domain = url.split('/').next()?;
	let domain = domain.split(':').next()?;
	Some(domain.trim_start_matches("www.").to_string())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn extracts_title_from_og_then_title_tag() {
		let html = r#"<html><head><meta property="og:title" content="OG Title"><title>Doc Title</title></head></html>"#;
		let meta = extract_metadata(html, None);
		assert_eq!(meta.title, Some("OG Title".to_string()));
	}

	#[test]
	fn extracts_site_from_url_when_meta_missing() {
		let html = r#"<html><head><title>Example</title></head></html>"#;
		let meta = extract_metadata(html, Some("https://www.example.com/path"));
		assert_eq!(meta.site, Some("example.com".to_string()));
	}
}
