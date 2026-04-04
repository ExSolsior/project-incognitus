/// Purpose: initialize the account ledger orders slab/account collection.
/// TODO: validate signer/writable constraints and finalize ledger args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for InitializeAccountLedgerOrders.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// InitializeAccountLedgerOrders instruction wrapper with parsed accounts and data.
pub struct InitializeAccountLedgerOrders<'a> {
    pub accounts: InitializeAccountLedgerOrdersAccounts<'a>,
    pub data: InitializeAccountLedgerOrdersInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for InitializeAccountLedgerOrders<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = InitializeAccountLedgerOrdersAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = InitializeAccountLedgerOrdersInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> InitializeAccountLedgerOrders<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate signer/writable constraints and initialize ledger orders state.
        Ok(())
    }
}

/// Accounts expected by InitializeAccountLedgerOrders.
pub struct InitializeAccountLedgerOrdersAccounts<'a> {
    /// Authority that seeds the ledger PDA.
    pub authority: &'a AccountInfo,
    /// Ledger orders account to initialize.
    pub account_ledger_orders: &'a AccountInfo,
    /// System program for account creation.
    pub system_program: &'a AccountInfo,
    /// Rent sysvar for rent-exempt initialization.
    pub rent_sysvar: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeAccountLedgerOrdersAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            account_ledger_orders: &accounts[1],
            system_program: &accounts[2],
            rent_sysvar: &accounts[3],
        })
    }
}

/// Instruction args for InitializeAccountLedgerOrders.
pub struct InitializeAccountLedgerOrdersInstructionData {
    /// TODO: define ledger initialization args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for InitializeAccountLedgerOrdersInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
