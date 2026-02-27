//! NEAR RPC client, transaction builder, and contract wrappers.
//!
//! This module provides the core NEAR interaction layer for the Templar CLI:
//!
//! - [`rpc`] — NEAR JSON-RPC client with retry/timeout logic
//! - [`tx_builder`] — Transaction construction helpers (gas, deposit, actions)
//! - [`signer`] — Credential loading and transaction signing
//! - [`contract`] — Typed wrappers for Templar contract view/call methods

pub mod contract;
pub mod rpc;
pub mod signer;
pub mod tx_builder;

// Re-export key types for convenience.
pub use rpc::{NearRpcClient, RpcClient, RpcErrorKind};
pub use signer::NearSigner;
pub use tx_builder::TransactionBuilder;
