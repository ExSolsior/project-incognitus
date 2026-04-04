/// Purpose: initialize a slab allocator account for a market node type.
/// TODO: validate signer/writable constraints and finalize allocator args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for InitializeSlabAllocator.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// InitializeSlabAllocator instruction wrapper with parsed accounts and data.
pub struct InitializeSlabAllocator<'a> {
    pub accounts: InitializeSlabAllocatorAccounts<'a>,
    pub data: InitializeSlabAllocatorInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for InitializeSlabAllocator<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = InitializeSlabAllocatorAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = InitializeSlabAllocatorInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> InitializeSlabAllocator<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate signer/writable constraints and initialize slab allocator state.
        Ok(())
    }
}

/// Accounts expected by InitializeSlabAllocator.
pub struct InitializeSlabAllocatorAccounts<'a> {
    /// Authority that seeds the allocator PDA.
    pub authority: &'a AccountInfo,
    /// Slab allocator account to initialize.
    pub slab_allocator: &'a AccountInfo,
    /// System program for account creation.
    pub system_program: &'a AccountInfo,
    /// Rent sysvar for rent-exempt initialization.
    pub rent_sysvar: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeSlabAllocatorAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            slab_allocator: &accounts[1],
            system_program: &accounts[2],
            rent_sysvar: &accounts[3],
        })
    }
}

/// Instruction args for InitializeSlabAllocator.
pub struct InitializeSlabAllocatorInstructionData {
    /// TODO: define allocator initialization args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for InitializeSlabAllocatorInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
