//! Orchestration for readable extraction stages.

use crate::readable::cleaner::remove_clutter;
use crate::readable::metadata::extract_metadata;
use crate::readable::render_markdown::html_to_markdown;
use crate::readable::render_text::html_to_text;
use crate::readable::types::{PageMetadata, ReadableContent};

#[derive(Debug, Clone)]
pub(crate) struct ReadableInput {
	pub(crate) html: String,
	pub(crate) url: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ReadableIntermediate {
	pub(crate) metadata: PageMetadata,
	pub(crate) cleaned_html: String,
}

pub fn extract_readable(html: &str, url: Option<&str>) -> ReadableContent {
	let input = ReadableInput {
		html: html.to_string(),
		url: url.map(ToString::to_string),
	};
	let intermediate = run_pipeline(&input);
	let text = html_to_text(&intermediate.cleaned_html);
	let markdown = Some(html_to_markdown(&intermediate.cleaned_html));

	ReadableContent {
		html: intermediate.cleaned_html,
		text,
		markdown,
		metadata: intermediate.metadata,
	}
}

fn run_pipeline(input: &ReadableInput) -> ReadableIntermediate {
	let metadata = extract_metadata(&input.html, input.url.as_deref());
	let cleaned_html = remove_clutter(&input.html);
	ReadableIntermediate { metadata, cleaned_html }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn extracts_article_and_removes_navigation() {
		let html = "<html><head><title>Story</title></head><body><header>Navigation</header><article><h1>Story</h1><p>This is the primary article content and it is long enough to satisfy the extraction threshold for selecting this block.</p></article><aside>Related links</aside></body></html>";
		let readable = extract_readable(html, Some("https://example.com/story"));
		assert!(readable.text.contains("primary article content"));
		assert!(!readable.text.contains("Navigation"));
		assert_eq!(readable.metadata.site, Some("example.com".to_string()));
	}

	#[test]
	fn falls_back_when_body_is_missing() {
		let html = "<div><p>Bodyless content survives</p></div>";
		let readable = extract_readable(html, None);
		assert!(readable.text.contains("Bodyless content survives"));
	}

	#[test]
	fn tolerates_malformed_html() {
		let html = "<main><h1>Broken<h1><p>Still readable";
		let readable = extract_readable(html, None);
		assert!(readable.text.contains("Still readable"));
	}

	#[test]
	fn extracts_metadata_only_pages() {
		let html = "<html><head><meta property='og:title' content='Meta Title'><meta name='author' content='Ada'><meta property='og:description' content='desc'></head><body></body></html>";
		let readable = extract_readable(html, Some("https://example.org"));
		assert_eq!(readable.metadata.title, Some("Meta Title".to_string()));
		assert_eq!(readable.metadata.author, Some("Ada".to_string()));
		assert_eq!(readable.metadata.description, Some("desc".to_string()));
	}
}
