use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Discriminator for the AccountLedgerOrders account.
pub const DISCRIMINATOR: u8 = 0;

/// Account ledger orders layout (placeholder).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AccountLedgerOrders {
    /// 1 when initialized, 0 otherwise.
    pub is_initialized: u8,
    /// Reserved padding for alignment.
    pub _padding: [u8; 7],
    /// Owner of the ledger.
    pub owner: Pubkey,
    /// Market this ledger belongs to.
    pub market: Pubkey,
    /// Number of active orders tracked.
    pub order_count: u64,
}

impl AccountLedgerOrders {
    /// Byte size of the account ledger orders layout.
    pub const LEN: usize = core::mem::size_of::<AccountLedgerOrders>();

    /// Cast immutable account data into a typed view.
    pub fn load(data: &[u8]) -> Result<&AccountLedgerOrders, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_ptr() as *const AccountLedgerOrders;
        Ok(unsafe { &*ptr })
    }

    /// Cast mutable account data into a typed view.
    pub fn load_mut(data: &mut [u8]) -> Result<&mut AccountLedgerOrders, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_mut_ptr() as *mut AccountLedgerOrders;
        Ok(unsafe { &mut *ptr })
    }
}
