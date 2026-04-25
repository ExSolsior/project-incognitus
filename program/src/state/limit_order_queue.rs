use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Discriminator for the LimitOrderQueue account.
pub const DISCRIMINATOR: u8 = 0;

/// Limit order queue metadata layout (placeholder).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LimitOrderQueue {
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
    /// Number of queued entries.
    pub count: u64,
}

impl LimitOrderQueue {
    /// Byte size of the limit order queue account.
    pub const LEN: usize = core::mem::size_of::<LimitOrderQueue>();

    /// Cast immutable account data into a typed view.
    pub fn load(data: &[u8]) -> Result<&LimitOrderQueue, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_ptr() as *const LimitOrderQueue;
        Ok(unsafe { &*ptr })
    }

    /// Cast mutable account data into a typed view.
    pub fn load_mut(data: &mut [u8]) -> Result<&mut LimitOrderQueue, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_mut_ptr() as *mut LimitOrderQueue;
        Ok(unsafe { &mut *ptr })
    }
}
