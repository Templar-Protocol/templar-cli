//! HTTP clients for external services.
//!
//! - [`backend`] — Templar backend REST API
//! - [`pyth`] — Pyth/Hermes oracle price feeds
//! - [`relayer`] — Relayer for UA transaction relay (Phase 4)

pub mod backend;
pub mod pyth;
pub mod relayer;
