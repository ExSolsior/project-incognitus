use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Discriminator for the AccountLedgerAddresses account.
pub const DISCRIMINATOR: u8 = 0;

/// Account ledger address layout (placeholder).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AccountLedgerAddresses {
    /// 1 when initialized, 0 otherwise.
    pub is_initialized: u8,
    /// Reserved padding for alignment.
    pub _padding: [u8; 7],
    /// Owner of the ledger.
    pub owner: Pubkey,
    /// Market this ledger belongs to.
    pub market: Pubkey,
    /// Number of address entries.
    pub address_count: u64,
}

impl AccountLedgerAddresses {
    /// Byte size of the account ledger addresses layout.
    pub const LEN: usize = core::mem::size_of::<AccountLedgerAddresses>();

    /// Cast immutable account data into a typed view.
    pub fn load(data: &[u8]) -> Result<&AccountLedgerAddresses, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_ptr() as *const AccountLedgerAddresses;
        Ok(unsafe { &*ptr })
    }

    /// Cast mutable account data into a typed view.
    pub fn load_mut(data: &mut [u8]) -> Result<&mut AccountLedgerAddresses, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_mut_ptr() as *mut AccountLedgerAddresses;
        Ok(unsafe { &mut *ptr })
    }
}
