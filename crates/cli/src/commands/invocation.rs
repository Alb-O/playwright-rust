//! Command invocation adapter for CLI enum variants.
//!
//! Generated from the command graph to avoid duplicated wiring across
//! command registry and CLI parsing.

pub(crate) use crate::commands::graph::from_cli_command;
