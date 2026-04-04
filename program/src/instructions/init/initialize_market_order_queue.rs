/// Purpose: initialize the market order queue account.
/// TODO: validate signer/writable constraints and finalize queue args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for InitializeMarketOrderQueue.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// InitializeMarketOrderQueue instruction wrapper with parsed accounts and data.
pub struct InitializeMarketOrderQueue<'a> {
    pub accounts: InitializeMarketOrderQueueAccounts<'a>,
    pub data: InitializeMarketOrderQueueInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for InitializeMarketOrderQueue<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = InitializeMarketOrderQueueAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = InitializeMarketOrderQueueInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> InitializeMarketOrderQueue<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate signer/writable constraints and initialize market order queue state.
        Ok(())
    }
}

/// Accounts expected by InitializeMarketOrderQueue.
pub struct InitializeMarketOrderQueueAccounts<'a> {
    /// Authority that seeds the queue PDA.
    pub authority: &'a AccountInfo,
    /// Market order queue account to initialize.
    pub market_order_queue: &'a AccountInfo,
    /// System program for account creation.
    pub system_program: &'a AccountInfo,
    /// Rent sysvar for rent-exempt initialization.
    pub rent_sysvar: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeMarketOrderQueueAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            market_order_queue: &accounts[1],
            system_program: &accounts[2],
            rent_sysvar: &accounts[3],
        })
    }
}

/// Instruction args for InitializeMarketOrderQueue.
pub struct InitializeMarketOrderQueueInstructionData {
    /// TODO: define queue initialization args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for InitializeMarketOrderQueueInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
