/// Purpose: close accounts and reclaim lamports for configured targets.
/// TODO: define close variants and finalize close args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for Close (grouped close_* handlers).
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// Close instruction wrapper with parsed accounts and data.
pub struct Close<'a> {
    pub accounts: CloseAccounts<'a>,
    pub data: CloseInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Close<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = CloseAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = CloseInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> Close<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: route close_* variants and reclaim lamports from the target account.
        Ok(())
    }
}

/// Accounts expected by Close.
pub struct CloseAccounts<'a> {
    /// Authority authorized to close the target.
    pub authority: &'a AccountInfo,
    /// Target account to close.
    pub target_account: &'a AccountInfo,
    /// Recipient that receives reclaimed lamports.
    pub recipient: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for CloseAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            target_account: &accounts[1],
            recipient: &accounts[2],
        })
    }
}

/// Instruction args for Close.
pub struct CloseInstructionData {
    /// TODO: define close variant selector and any extra args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for CloseInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
