use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Discriminator for the AddressLookupTable account.
pub const DISCRIMINATOR: u8 = 0;

/// Address lookup table layout (placeholder).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AddressLookupTable {
    /// 1 when initialized, 0 otherwise.
    pub is_initialized: u8,
    /// Reserved padding for alignment.
    pub _padding: [u8; 7],
    /// Authority controlling the lookup table.
    pub authority: Pubkey,
    /// Market this lookup table belongs to.
    pub market: Pubkey,
    /// Number of stored addresses.
    pub address_count: u64,
    /// Slot of the last update.
    pub last_updated_slot: u64,
}

impl AddressLookupTable {
    /// Byte size of the address lookup table account.
    pub const LEN: usize = core::mem::size_of::<AddressLookupTable>();

    /// Cast immutable account data into a typed view.
    pub fn load(data: &[u8]) -> Result<&AddressLookupTable, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_ptr() as *const AddressLookupTable;
        Ok(unsafe { &*ptr })
    }

    /// Cast mutable account data into a typed view.
    pub fn load_mut(data: &mut [u8]) -> Result<&mut AddressLookupTable, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_mut_ptr() as *mut AddressLookupTable;
        Ok(unsafe { &mut *ptr })
    }
}
