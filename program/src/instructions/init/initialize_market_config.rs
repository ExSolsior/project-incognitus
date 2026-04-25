/// Purpose: initialize the market configuration account.
/// TODO: validate signer/writable constraints and finalize config args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for InitializeMarketConfig.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// InitializeMarketConfig instruction wrapper with parsed accounts and data.
pub struct InitializeMarketConfig<'a> {
    pub accounts: InitializeMarketConfigAccounts<'a>,
    pub data: InitializeMarketConfigInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for InitializeMarketConfig<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = InitializeMarketConfigAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = InitializeMarketConfigInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> InitializeMarketConfig<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate signer/writable constraints and initialize market config state.
        Ok(())
    }
}

/// Accounts expected by InitializeMarketConfig.
pub struct InitializeMarketConfigAccounts<'a> {
    /// Authority that seeds and configures the market.
    pub authority: &'a AccountInfo,
    /// Market config PDA to initialize.
    pub market_config: &'a AccountInfo,
    /// System program for account creation.
    pub system_program: &'a AccountInfo,
    /// Rent sysvar for rent-exempt initialization.
    pub rent_sysvar: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeMarketConfigAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            market_config: &accounts[1],
            system_program: &accounts[2],
            rent_sysvar: &accounts[3],
        })
    }
}

/// Instruction args for InitializeMarketConfig.
pub struct InitializeMarketConfigInstructionData {
    /// TODO: define market config initialization args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for InitializeMarketConfigInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
