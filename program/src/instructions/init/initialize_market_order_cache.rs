/// Purpose: initialize the market order cache account.
/// TODO: validate signer/writable constraints and finalize cache args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for InitializeMarketOrderCache.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// InitializeMarketOrderCache instruction wrapper with parsed accounts and data.
pub struct InitializeMarketOrderCache<'a> {
    pub accounts: InitializeMarketOrderCacheAccounts<'a>,
    pub data: InitializeMarketOrderCacheInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for InitializeMarketOrderCache<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = InitializeMarketOrderCacheAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = InitializeMarketOrderCacheInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> InitializeMarketOrderCache<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate signer/writable constraints and initialize market order cache state.
        Ok(())
    }
}

/// Accounts expected by InitializeMarketOrderCache.
pub struct InitializeMarketOrderCacheAccounts<'a> {
    /// Authority that seeds the cache PDA.
    pub authority: &'a AccountInfo,
    /// Market order cache account to initialize.
    pub market_order_cache: &'a AccountInfo,
    /// System program for account creation.
    pub system_program: &'a AccountInfo,
    /// Rent sysvar for rent-exempt initialization.
    pub rent_sysvar: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeMarketOrderCacheAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            market_order_cache: &accounts[1],
            system_program: &accounts[2],
            rent_sysvar: &accounts[3],
        })
    }
}

/// Instruction args for InitializeMarketOrderCache.
pub struct InitializeMarketOrderCacheInstructionData {
    /// TODO: define cache initialization args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for InitializeMarketOrderCacheInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
