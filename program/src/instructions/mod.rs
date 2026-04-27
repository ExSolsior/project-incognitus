pub mod allocate;
pub mod close;
pub mod initialize;

/// Instruction discriminators — first byte of instruction data.
pub const IX_INITIALIZE: u8 = 0x00;
pub const IX_ALLOCATE: u8 = 0x01;
pub const IX_CLOSE: u8 = 0x02;

/// Account type tags — second byte of instruction data.
/// Tells the instruction which account type to operate on.
pub const ACCOUNT_TYPE_MARKET_CONFIG: u8 = 0x00;
pub const ACCOUNT_TYPE_ASSET_ORDER_STATUS: u8 = 0x01;
pub const ACCOUNT_TYPE_ASSET_BALANCE_LEDGER: u8 = 0x02;
pub const ACCOUNT_TYPE_ARENA_ALLOCATOR: u8 = 0x03;
pub const ACCOUNT_TYPE_SLAB_ALLOCATOR: u8 = 0x04;
