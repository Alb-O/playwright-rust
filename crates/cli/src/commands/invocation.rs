//! Command invocation adapter for CLI enum variants.
//!
//! Generated from the command catalog to avoid duplicated wiring across
//! command registry and CLI parsing.

pub(crate) use crate::commands::catalog::from_cli_command;
