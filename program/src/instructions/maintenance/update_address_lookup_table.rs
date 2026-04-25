/// Purpose: update an address lookup table with new entries.
/// TODO: validate authority and finalize update args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for UpdateAddressLookupTable.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// UpdateAddressLookupTable instruction wrapper with parsed accounts and data.
pub struct UpdateAddressLookupTable<'a> {
    pub accounts: UpdateAddressLookupTableAccounts<'a>,
    pub data: UpdateAddressLookupTableInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for UpdateAddressLookupTable<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = UpdateAddressLookupTableAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = UpdateAddressLookupTableInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> UpdateAddressLookupTable<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate authority and update the address lookup table entries.
        Ok(())
    }
}

/// Accounts expected by UpdateAddressLookupTable.
pub struct UpdateAddressLookupTableAccounts<'a> {
    /// Authority allowed to update the lookup table.
    pub authority: &'a AccountInfo,
    /// Address lookup table account to update.
    pub address_lookup_table: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for UpdateAddressLookupTableAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            address_lookup_table: &accounts[1],
        })
    }
}

/// Instruction args for UpdateAddressLookupTable.
pub struct UpdateAddressLookupTableInstructionData {
    /// TODO: define address update arguments.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for UpdateAddressLookupTableInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
