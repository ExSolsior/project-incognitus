use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Discriminator for the AccountStatus account.
pub const DISCRIMINATOR: u8 = 0;

/// Status flag indicating the account is active.
pub const STATUS_ACTIVE: u8 = 1;

/// Status flag indicating the account is frozen.
pub const STATUS_FROZEN: u8 = 2;

/// Account status metadata layout (placeholder).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AccountStatus {
    /// Status flag for the account.
    pub status: u8,
    /// Reserved padding for alignment.
    pub _padding: [u8; 7],
    /// Owner or authority for the account.
    pub owner: Pubkey,
}

impl AccountStatus {
    /// Byte size of the account status layout.
    pub const LEN: usize = core::mem::size_of::<AccountStatus>();

    /// Cast immutable account data into a typed view.
    pub fn load(data: &[u8]) -> Result<&AccountStatus, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_ptr() as *const AccountStatus;
        Ok(unsafe { &*ptr })
    }

    /// Cast mutable account data into a typed view.
    pub fn load_mut(data: &mut [u8]) -> Result<&mut AccountStatus, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_mut_ptr() as *mut AccountStatus;
        Ok(unsafe { &mut *ptr })
    }
}
