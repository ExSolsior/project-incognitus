/// Purpose: deposit funds into the market vault for trading.
/// TODO: validate vault accounts and finalize deposit args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for Deposit.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// Deposit instruction wrapper with parsed accounts and data.
pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub data: DepositInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = DepositAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = DepositInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> Deposit<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate vault accounts and transfer funds into the market vault.
        Ok(())
    }
}

/// Accounts expected by Deposit.
pub struct DepositAccounts<'a> {
    /// Depositing user (signer, writable for fees).
    pub depositor: &'a AccountInfo,
    /// User token account funding the deposit.
    pub depositor_token: &'a AccountInfo,
    /// Vault token account receiving funds.
    pub vault_token: &'a AccountInfo,
    /// Token program for SPL token CPI.
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            depositor: &accounts[0],
            depositor_token: &accounts[1],
            vault_token: &accounts[2],
            token_program: &accounts[3],
        })
    }
}

/// Instruction args for Deposit.
pub struct DepositInstructionData {
    /// TODO: define deposit amount and any routing metadata.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
