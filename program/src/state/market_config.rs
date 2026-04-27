/// MarketConfig — root market account.
///
/// Seeds: ["market-config", base_token_mint, quote_token_mint]
///
/// Fields are WIP and will be iterated on. The discriminator and seed
/// derivation are stable.

/// Discriminator for MarketConfig accounts.
pub const DISCRIMINATOR: u32 = 0x4D4B_5443; // "MKTC"

/// PDA seed prefix.
pub const SEED_PREFIX: &[u8] = b"market-config";

/// Minimum data length for a MarketConfig account.
/// Layout (WIP — field positions will change):
///   [0..4]     discriminator
///   [4..36]    base_asset_token_mint
///   [36..68]   quote_asset_token_mint
///   [68..100]  base_asset_token_pool
///   [100..132] quote_asset_token_pool
///   [132..164] order_book_ledger
///   [164..196] arena: aggregator
///   [196..228] arena: limit_order_queue
///   [228..260] arena: market_order_queue
///   [260..292] slab: order_book
///   [292..324] slab: ledger_status
pub const MIN_DATA_LEN: usize = 324;
