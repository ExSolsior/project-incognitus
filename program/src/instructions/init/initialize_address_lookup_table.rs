/// Purpose: initialize the address lookup table account.
/// TODO: validate signer/writable constraints and finalize lookup table args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for InitializeAddressLookupTable.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// InitializeAddressLookupTable instruction wrapper with parsed accounts and data.
pub struct InitializeAddressLookupTable<'a> {
    pub accounts: InitializeAddressLookupTableAccounts<'a>,
    pub data: InitializeAddressLookupTableInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for InitializeAddressLookupTable<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = InitializeAddressLookupTableAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = InitializeAddressLookupTableInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> InitializeAddressLookupTable<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate signer/writable constraints and initialize address lookup table state.
        Ok(())
    }
}

/// Accounts expected by InitializeAddressLookupTable.
pub struct InitializeAddressLookupTableAccounts<'a> {
    /// Authority that seeds the lookup table PDA.
    pub authority: &'a AccountInfo,
    /// Address lookup table account to initialize.
    pub address_lookup_table: &'a AccountInfo,
    /// System program for account creation.
    pub system_program: &'a AccountInfo,
    /// Rent sysvar for rent-exempt initialization.
    pub rent_sysvar: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeAddressLookupTableAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            address_lookup_table: &accounts[1],
            system_program: &accounts[2],
            rent_sysvar: &accounts[3],
        })
    }
}

/// Instruction args for InitializeAddressLookupTable.
pub struct InitializeAddressLookupTableInstructionData {
    /// TODO: define address lookup table initialization args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for InitializeAddressLookupTableInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
