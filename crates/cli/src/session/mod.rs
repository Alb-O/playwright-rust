//! Session lifecycle and browser-connection subsystem.
//!
//! This module centralizes session descriptor persistence, acquisition
//! strategy decisions, and shared connect/discover orchestration.

/// Browser connect/discover helpers shared across commands.
pub mod connect;
/// High-level service facade for `connect` command operations.
pub mod connect_service;
/// Backward-compatible re-exports for legacy `session::connector` paths.
pub mod connector;
/// Daemon browser lease helpers.
mod daemon_lease;
/// Persisted session descriptor schema and helpers.
pub mod descriptor;
/// Descriptor lifecycle orchestration for status/clear/persist.
mod descriptor_lifecycle;
/// Session request/manager/handle types and orchestration.
pub mod manager;
/// Active session handle and acquisition result types.
pub mod outcome;
/// Session descriptor repository facade.
pub mod repository;
/// Browser session acquisition helpers used by the manager.
mod session_factory;
/// Session request specification and builder helpers.
pub mod spec;
/// Pure strategy selection for session acquisition.
pub mod strategy;

/// Persisted session descriptor metadata.
pub use descriptor::SessionDescriptor;
/// Session manager and orchestration service.
pub use manager::SessionManager;
/// Session acquisition handle.
pub use outcome::SessionHandle;
/// Session acquisition request specification.
pub use spec::SessionRequest;
