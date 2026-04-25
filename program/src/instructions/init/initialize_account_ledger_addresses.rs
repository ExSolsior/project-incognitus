/// Purpose: initialize the account ledger address slab/account collection.
/// TODO: validate signer/writable constraints and finalize ledger args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for InitializeAccountLedgerAddresses.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// InitializeAccountLedgerAddresses instruction wrapper with parsed accounts and data.
pub struct InitializeAccountLedgerAddresses<'a> {
    pub accounts: InitializeAccountLedgerAddressesAccounts<'a>,
    pub data: InitializeAccountLedgerAddressesInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for InitializeAccountLedgerAddresses<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = InitializeAccountLedgerAddressesAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = InitializeAccountLedgerAddressesInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> InitializeAccountLedgerAddresses<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate signer/writable constraints and initialize ledger addresses state.
        Ok(())
    }
}

/// Accounts expected by InitializeAccountLedgerAddresses.
pub struct InitializeAccountLedgerAddressesAccounts<'a> {
    /// Authority that seeds the ledger PDA.
    pub authority: &'a AccountInfo,
    /// Ledger addresses account to initialize.
    pub account_ledger_addresses: &'a AccountInfo,
    /// System program for account creation.
    pub system_program: &'a AccountInfo,
    /// Rent sysvar for rent-exempt initialization.
    pub rent_sysvar: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeAccountLedgerAddressesAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            account_ledger_addresses: &accounts[1],
            system_program: &accounts[2],
            rent_sysvar: &accounts[3],
        })
    }
}

/// Instruction args for InitializeAccountLedgerAddresses.
pub struct InitializeAccountLedgerAddressesInstructionData {
    /// TODO: define ledger initialization args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for InitializeAccountLedgerAddressesInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
