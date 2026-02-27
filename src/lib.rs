//! # Templar CLI
//!
//! The Templar Protocol command-line interface — Cypher Lending, Be Your Own Bank.
//!
//! This crate provides typed wrappers for all Templar Protocol contract
//! interactions, cross-chain bridge operations, and a branded cypherpunk
//! terminal experience.
//!
//! ## Modules
//!
//! - [`config`] — Configuration profiles, network settings, and credential paths
//! - [`error`] — Unified error types for all CLI operations
//! - [`types`] — Domain types mirrored from Templar contracts
//! - [`near`] — NEAR RPC client, transaction builder, and contract wrappers
//! - [`client`] — HTTP clients for backend API, relayer, and Pyth oracle
//! - [`commands`] — CLI command implementations
//! - [`display`] — Themed terminal output, banners, spinners, and frames

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod commands;
pub mod config;
pub mod client;
pub mod display;
pub mod error;
pub mod near;
pub mod types;
