use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Discriminator for the SlabAllocator account.
pub const DISCRIMINATOR: u8 = 0;

/// Slab allocator metadata layout (placeholder).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SlabAllocator {
    /// 1 when initialized, 0 otherwise.
    pub is_initialized: u8,
    /// PDA bump for the allocator.
    pub bump: u8,
    /// Reserved padding for alignment.
    pub _padding: [u8; 6],
    /// Market this allocator belongs to.
    pub market: Pubkey,
    /// Maximum node capacity.
    pub capacity: u64,
    /// Number of allocated nodes.
    pub allocated: u64,
}

impl SlabAllocator {
    /// Byte size of the slab allocator account.
    pub const LEN: usize = core::mem::size_of::<SlabAllocator>();

    /// Cast immutable account data into a typed view.
    pub fn load(data: &[u8]) -> Result<&SlabAllocator, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_ptr() as *const SlabAllocator;
        Ok(unsafe { &*ptr })
    }

    /// Cast mutable account data into a typed view.
    pub fn load_mut(data: &mut [u8]) -> Result<&mut SlabAllocator, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let ptr = data.as_mut_ptr() as *mut SlabAllocator;
        Ok(unsafe { &mut *ptr })
    }
}
