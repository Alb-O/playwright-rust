//! Session lifecycle and browser-connection subsystem.
//!
//! This module centralizes session descriptor persistence, acquisition
//! strategy decisions, and shared connect/discover orchestration.

/// High-level service facade for `connect` command operations.
pub mod connect_service;
/// Browser connect/discover helpers shared across commands.
pub mod connector;
/// Persisted session descriptor schema and helpers.
pub mod descriptor;
/// Session request/manager/handle types and orchestration.
pub mod manager;
/// Active session handle and acquisition result types.
pub mod outcome;
/// Session descriptor repository facade.
pub mod repository;
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
