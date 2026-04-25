use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Discriminator for the MarketOrderCache account.
pub const DISCRIMINATOR: u8 = 0;

/// Market order cache layout (placeholder).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MarketOrderCache {
    /// 1 when initialized, 0 otherwise.
    pub is_initialized: u8,
    /// Reserved padding for alignment.
    pub _padding: [u8; 7],
    /// Market this cache belongs to.
    pub market: Pubkey,
    /// Last processed order id.
    pub last_order_id: u64,
    /// Cached match count.
    pub cached_matches: u64,
}

impl MarketOrderCache {
    /// Byte size of the market order cache account.
    pub const LEN: usize = core::mem::size_of::<MarketOrderCache>();

    /// Cast immutable account data into a typed view.
    pub fn load(data: &[u8]) -> Result<&MarketOrderCache, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_ptr() as *const MarketOrderCache;
        Ok(unsafe { &*ptr })
    }

    /// Cast mutable account data into a typed view.
    pub fn load_mut(data: &mut [u8]) -> Result<&mut MarketOrderCache, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_mut_ptr() as *mut MarketOrderCache;
        Ok(unsafe { &mut *ptr })
    }
}
