//! Structured output envelope and payload models.

#[cfg(test)]
mod tests;

mod data;
mod format;
mod model;
mod result_builder;

pub use data::*;
pub use format::OutputFormat;
pub use model::*;
pub use result_builder::{ResultBuilder, print_error_stderr, print_failure_with_artifacts, print_result};
