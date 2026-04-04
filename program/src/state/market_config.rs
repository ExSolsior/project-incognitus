use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Discriminator for the MarketConfig account.
pub const DISCRIMINATOR: u8 = 0;

/// Market configuration account layout (placeholder).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MarketConfig {
    /// 1 when initialized, 0 otherwise.
    pub is_initialized: u8,
    /// Version or bump for PDA derivation.
    pub version: u8,
    /// Reserved padding for alignment.
    pub _padding: [u8; 6],
    /// Authority that controls the market.
    pub authority: Pubkey,
    /// Base mint configured for the market.
    pub base_mint: Pubkey,
    /// Quote mint configured for the market.
    pub quote_mint: Pubkey,
}

impl MarketConfig {
    /// Byte size of the market config account.
    pub const LEN: usize = core::mem::size_of::<MarketConfig>();

    /// Cast immutable account data into a typed view.
    pub fn load(data: &[u8]) -> Result<&MarketConfig, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_ptr() as *const MarketConfig;
        Ok(unsafe { &*ptr })
    }

    /// Cast mutable account data into a typed view.
    pub fn load_mut(data: &mut [u8]) -> Result<&mut MarketConfig, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_mut_ptr() as *mut MarketConfig;
        Ok(unsafe { &mut *ptr })
    }
}
