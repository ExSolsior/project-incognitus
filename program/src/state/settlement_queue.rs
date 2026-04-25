use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Discriminator for the SettlementQueue account.
pub const DISCRIMINATOR: u8 = 0;

/// Settlement queue metadata layout (placeholder).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SettlementQueue {
    /// 1 when initialized, 0 otherwise.
    pub is_initialized: u8,
    /// Reserved padding for alignment.
    pub _padding: [u8; 7],
    /// Market this queue belongs to.
    pub market: Pubkey,
    /// Index of the head of the queue.
    pub head: u64,
    /// Index of the tail of the queue.
    pub tail: u64,
    /// Number of queued settlements.
    pub count: u64,
}

impl SettlementQueue {
    /// Byte size of the settlement queue account.
    pub const LEN: usize = core::mem::size_of::<SettlementQueue>();

    /// Cast immutable account data into a typed view.
    pub fn load(data: &[u8]) -> Result<&SettlementQueue, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_ptr() as *const SettlementQueue;
        Ok(unsafe { &*ptr })
    }

    /// Cast mutable account data into a typed view.
    pub fn load_mut(data: &mut [u8]) -> Result<&mut SettlementQueue, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_mut_ptr() as *mut SettlementQueue;
        Ok(unsafe { &mut *ptr })
    }
}
