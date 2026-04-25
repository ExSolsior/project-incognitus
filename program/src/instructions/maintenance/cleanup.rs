/// Purpose: prune stale queues/caches and reclaim empty accounts.
/// TODO: finalize cleanup parameters and pruning criteria.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for Cleanup.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// Cleanup instruction wrapper with parsed accounts and data.
pub struct Cleanup<'a> {
    pub accounts: CleanupAccounts<'a>,
    pub data: CleanupInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Cleanup<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = CleanupAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = CleanupInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> Cleanup<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: prune stale queues, caches, and close empty accounts.
        Ok(())
    }
}

/// Accounts expected by Cleanup.
pub struct CleanupAccounts<'a> {
    /// Market configuration account.
    pub market_config: &'a AccountInfo,
    /// Authority authorized to perform cleanup.
    pub authority: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for CleanupAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            market_config: &accounts[0],
            authority: &accounts[1],
        })
    }
}

/// Instruction args for Cleanup.
pub struct CleanupInstructionData {
    /// TODO: define cleanup parameters (batch size, cutoff slot).
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for CleanupInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
