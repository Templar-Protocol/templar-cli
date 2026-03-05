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
// --- Allowed pedantic lints (Phase 1/2 scaffold) ---
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unused_async)]
#![allow(clippy::unused_self)]

pub mod client;
pub mod commands;
pub mod config;
pub mod display;
pub mod error;
pub mod near;
pub mod types;
