/// Purpose: allocate lamports/space for an account ledger account.
/// TODO: validate signer/writable constraints and finalize allocation args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for AllocateAccountLedger.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// AllocateAccountLedger instruction wrapper with parsed accounts and data.
pub struct AllocateAccountLedger<'a> {
    pub accounts: AllocateAccountLedgerAccounts<'a>,
    pub data: AllocateAccountLedgerInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for AllocateAccountLedger<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = AllocateAccountLedgerAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = AllocateAccountLedgerInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> AllocateAccountLedger<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: allocate lamports and space for the account ledger account.
        Ok(())
    }
}

/// Accounts expected by AllocateAccountLedger.
pub struct AllocateAccountLedgerAccounts<'a> {
    /// Payer funding the allocation.
    pub payer: &'a AccountInfo,
    /// Account ledger account to allocate.
    pub account_ledger: &'a AccountInfo,
    /// System program for account allocation.
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for AllocateAccountLedgerAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            payer: &accounts[0],
            account_ledger: &accounts[1],
            system_program: &accounts[2],
        })
    }
}

/// Instruction args for AllocateAccountLedger.
pub struct AllocateAccountLedgerInstructionData {
    /// TODO: define allocation size args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for AllocateAccountLedgerInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
