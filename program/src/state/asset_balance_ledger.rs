/// AssetBalanceLedger — per-user cross-market balance tracking.
///
/// Seeds: ["asset-balance-ledger", owner]
///
/// Uses double-entry bookkeeping: balance, credit, debit per asset.
/// Balance is the settled amount. Credit/debit track transitional state.
/// Settlement reconciles credit/debit into balance.
///
/// Dynamic size — grows as user trades new assets.

/// Discriminator for AssetBalanceLedger accounts.
pub const DISCRIMINATOR: u32 = 0x4142_4C47; // "ABLG"

/// PDA seed prefix.
pub const SEED_PREFIX: &[u8] = b"asset-balance-ledger";

/// Minimum data length for an AssetBalanceLedger account.
/// Layout (WIP — field positions will change):
///   [0..4]     discriminator
///   [4..36]    owner_address
///   [36..]     assets (dynamic array)
///              each asset entry:
///                [0..32]  mint_address
///                [32..40] balance: u64
///                [40..48] credit: u64
///                [48..56] debit: u64
pub const MIN_DATA_LEN: usize = 36;

/// Size of a single asset entry within the ledger.
pub const ASSET_ENTRY_SIZE: usize = 56;
