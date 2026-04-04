/// Purpose: initialize the limit order queue account.
/// TODO: validate signer/writable constraints and finalize queue args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for InitializeLimitOrderQueue.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// InitializeLimitOrderQueue instruction wrapper with parsed accounts and data.
pub struct InitializeLimitOrderQueue<'a> {
    pub accounts: InitializeLimitOrderQueueAccounts<'a>,
    pub data: InitializeLimitOrderQueueInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for InitializeLimitOrderQueue<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = InitializeLimitOrderQueueAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = InitializeLimitOrderQueueInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> InitializeLimitOrderQueue<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: validate signer/writable constraints and initialize limit order queue state.
        Ok(())
    }
}

/// Accounts expected by InitializeLimitOrderQueue.
pub struct InitializeLimitOrderQueueAccounts<'a> {
    /// Authority that seeds the queue PDA.
    pub authority: &'a AccountInfo,
    /// Limit order queue account to initialize.
    pub limit_order_queue: &'a AccountInfo,
    /// System program for account creation.
    pub system_program: &'a AccountInfo,
    /// Rent sysvar for rent-exempt initialization.
    pub rent_sysvar: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeLimitOrderQueueAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            limit_order_queue: &accounts[1],
            system_program: &accounts[2],
            rent_sysvar: &accounts[3],
        })
    }
}

/// Instruction args for InitializeLimitOrderQueue.
pub struct InitializeLimitOrderQueueInstructionData {
    /// TODO: define queue initialization args.
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for InitializeLimitOrderQueueInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
