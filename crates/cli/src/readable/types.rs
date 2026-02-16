//! Public types returned by readable extraction.

/// Metadata extracted from the source page.
#[derive(Debug, Default, Clone)]
pub struct PageMetadata {
	pub title: Option<String>,
	pub author: Option<String>,
	pub published: Option<String>,
	pub description: Option<String>,
	pub image: Option<String>,
	pub site: Option<String>,
}

/// Result of readable content extraction.
#[derive(Debug)]
pub struct ReadableContent {
	/// Cleaned HTML content after clutter removal and content extraction.
	pub html: String,
	/// Plain text rendering of `html`.
	pub text: String,
	/// Markdown rendering of `html`.
	pub markdown: Option<String>,
	/// Metadata extracted from the original source HTML.
	pub metadata: PageMetadata,
}
