/// ArenaAllocator — linear allocator accounts for queues and aggregation.
///
/// Three arena types exist per market:
///   - Aggregate
///   - LimitOrderQueue
///   - MarketOrderQueue
///
/// Seeds: ["arena-allocator", market_config, arena_type_tag]
///
/// Initialized at 10 KB, grows in 10 KB increments up to 10 MB.

/// Discriminator for ArenaAllocator accounts.
pub const DISCRIMINATOR: u32 = 0x4152_4E41; // "ARNA"

/// PDA seed prefix.
pub const SEED_PREFIX: &[u8] = b"arena-allocator";

/// Arena type tags — used as the final PDA seed byte.
pub const ARENA_TYPE_AGGREGATE: u8 = 0x01;
pub const ARENA_TYPE_LIMIT_ORDER_QUEUE: u8 = 0x02;
pub const ARENA_TYPE_MARKET_ORDER_QUEUE: u8 = 0x03;

/// Header size for an ArenaAllocator account.
/// Layout:
///   [0..4]   discriminator
///   [4..8]   write_offset: u32  — next write position
///   [8..12]  read_offset: u32   — next read position
///   [12..16] capacity: u32      — total data region size
///   [16..20] flags: u32         — state flags (resize interrupt, etc.)
///   [20..24] reserved: u32
pub const HEADER_SIZE: usize = 24;
