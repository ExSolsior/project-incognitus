/// AssetOrderStatus — per-user per-market order tracking.
///
/// Seeds: ["asset-order-status", market_config, owner, ledger]
///
/// Fields are WIP and will be iterated on. The discriminator and seed
/// derivation are stable.

/// Discriminator for AssetOrderStatus accounts.
pub const DISCRIMINATOR: u32 = 0x4153_4F53; // "ASOS"

/// PDA seed prefix.
pub const SEED_PREFIX: &[u8] = b"asset-order-status";

/// Minimum data length for an AssetOrderStatus account.
/// Layout (WIP — field positions will change):
///   [0..4]     discriminator
///   [4..36]    owner_address
///   [36..68]   asset_balance_ledger_address
///   [68..72]   base_asset_index
///   [72..76]   quote_asset_index
///   [76..80]   ledger_index
///   [80..]     orders (dynamic array)
pub const MIN_DATA_LEN: usize = 80;
