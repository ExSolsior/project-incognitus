pub mod arena_allocator;
pub mod asset_balance_ledger;
pub mod asset_order_status;
pub mod market_config;
pub mod slab_allocator;

/// Discriminator assigned to accounts that are being closed.
/// Any account with this discriminator must not be used for normal operations.
pub const CLOSING_DISCRIMINATOR: u32 = 0xFFFF_FFFF;

/// Initial allocation size for Arena and Slab accounts (10 KB).
pub const INITIAL_ACCOUNT_SIZE: u64 = 10 * 1024;

/// Allocation increment size (10 KB).
pub const ALLOCATE_INCREMENT: u64 = 10 * 1024;

/// Maximum account size (10 MB).
pub const MAX_ACCOUNT_SIZE: u64 = 10 * 1024 * 1024;

/// Reads a u32 discriminator from the first 4 bytes of account data.
///
/// # Safety
/// Caller must ensure `data.len() >= 4`.
#[inline(always)]
pub fn read_discriminator(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

/// Writes a u32 discriminator to the first 4 bytes of account data.
///
/// # Safety
/// Caller must ensure `data.len() >= 4`.
#[inline(always)]
pub fn write_discriminator(data: &mut [u8], disc: u32) {
    data[0..4].copy_from_slice(&disc.to_le_bytes());
}
