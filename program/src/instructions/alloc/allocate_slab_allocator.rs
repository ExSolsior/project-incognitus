/// Purpose: allocate lamports/space for a slab allocator account.
/// TODO: validate signer/writable constraints and finalize allocation args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for AllocateSlabAllocator.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// AllocateSlabAllocator instruction wrapper with parsed accounts and data.
pub struct AllocateSlabAllocator<'a> {
    pub accounts: AllocateSlabAllocatorAccounts<'a>,
    pub data: AllocateSlabAllocatorInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for AllocateSlabAllocator<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = AllocateSlabAllocatorAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = AllocateSlabAllocatorInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> AllocateSlabAllocator<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: allocate lamports and space for the slab allocator account.
        Ok(())
    }
}

/// Accounts expected by AllocateSlabAllocator.
pub struct AllocateSlabAllocatorAccounts<'a> {
    /// Payer funding the allocation.
    pub payer: &'a AccountInfo,
    /// Slab allocator account to allocate.
    pub slab_allocator: &'a AccountInfo,
    /// System program for account allocation.
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for AllocateSlabAllocatorAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            payer: &accounts[0],
            slab_allocator: &accounts[1],
            system_program: &accounts[2],
        })
    }
}

/// Instruction args for AllocateSlabAllocator.
pub struct AllocateSlabAllocatorInstructionData {
    /// TODO: define allocation size args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for AllocateSlabAllocatorInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
