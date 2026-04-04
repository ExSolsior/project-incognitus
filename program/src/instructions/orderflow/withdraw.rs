/// Purpose: withdraw funds from the market vault back to a user.
/// TODO: validate vault accounts and finalize withdraw args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for Withdraw.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// Withdraw instruction wrapper with parsed accounts and data.
pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
    pub data: WithdrawInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = WithdrawAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = WithdrawInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> Withdraw<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate vault accounts and transfer funds out to the user.
        Ok(())
    }
}

/// Accounts expected by Withdraw.
pub struct WithdrawAccounts<'a> {
    /// Withdrawing user (signer, writable for fees).
    pub withdrawer: &'a AccountInfo,
    /// User token account receiving funds.
    pub withdrawer_token: &'a AccountInfo,
    /// Vault token account sending funds.
    pub vault_token: &'a AccountInfo,
    /// Token program for SPL token CPI.
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            withdrawer: &accounts[0],
            withdrawer_token: &accounts[1],
            vault_token: &accounts[2],
            token_program: &accounts[3],
        })
    }
}

/// Instruction args for Withdraw.
pub struct WithdrawInstructionData {
    /// TODO: define withdraw amount and any routing metadata.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for WithdrawInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
