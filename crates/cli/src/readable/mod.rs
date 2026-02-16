//! Readable content extraction from HTML pages.
//!
//! This module provides a small extraction pipeline used by `page.read`.
//! The pipeline keeps metadata extraction, clutter removal, and rendering
//! stages isolated for easier testing and maintenance.

mod cleaner;
mod config;
mod entities;
mod junk;
mod metadata;
mod pipeline;
mod render_markdown;
mod render_text;
mod selector;
mod types;

pub use pipeline::extract_readable;
pub use types::{PageMetadata, ReadableContent};
