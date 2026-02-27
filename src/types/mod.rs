//! Domain types mirrored from Templar Protocol contracts.
//!
//! These are simplified CLI versions of the on-chain types. They do not use
//! NEAR SDK macros — instead they derive [`serde::Serialize`] and
//! [`serde::Deserialize`] and match the JSON serialization format of the
//! contract types exactly.
//!
//! # Modules
//!
//! - [`number`] — Numeric wrappers (`U64`, `U128`, `Decimal`) and amount newtypes
//! - [`market`] — Lending market configuration and snapshots
//! - [`vault`] — Vault configuration, fees, and access restrictions
//! - [`registry`] — Contract deployment records
//! - [`oracle`] — Pyth oracle price feed types
//! - [`borrow`] — Borrow position and health status types
//! - [`supply`] — Supply position and withdrawal status types
//! - [`universal_account`] — Universal account key types
//! - [`bridge`] — Cross-chain bridge types

pub mod borrow;
pub mod bridge;
pub mod market;
pub mod number;
pub mod oracle;
pub mod registry;
pub mod supply;
pub mod universal_account;
pub mod vault;

// ---------------------------------------------------------------------------
// Re-exports for convenience
// ---------------------------------------------------------------------------

// Numeric primitives
pub use number::{
    Accumulator, BorrowAssetAmount, CollateralAssetAmount, Decimal, FungibleAsset, U128,
    U128String, U64,
};

// Market
pub use market::{
    BorrowAssetMetrics, InterestRateModel, MarketConfiguration, MarketFeeConfig, Snapshot,
    YieldDistribution, YieldWeights,
};

// Vault
pub use vault::{Fee, Fees, Restrictions, VaultConfiguration};

// Registry
pub use registry::{DeployMode, Deployment};

// Oracle
pub use oracle::{OracleResponse, PriceIdentifier, PythPrice};

// Borrow
pub use borrow::{BorrowPosition, BorrowStatus, LiquidationReason};

// Supply
pub use supply::{Deposit, IncomingDeposit, SupplyPosition, WithdrawalRequestStatus};

// Universal account
pub use universal_account::{KeyId, KeyParameters};

// Bridge
pub use bridge::{BridgeIntent, BridgeStatus, ChainId, IntentsChain, TokenInfo};
