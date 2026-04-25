/// Purpose: aggregate queued limit orders into the order book.
/// TODO: finalize aggregation parameters and batching logic.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for LimitOrderAggregation.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// LimitOrderAggregation instruction wrapper with parsed accounts and data.
pub struct LimitOrderAggregation<'a> {
    pub accounts: LimitOrderAggregationAccounts<'a>,
    pub data: LimitOrderAggregationInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for LimitOrderAggregation<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = LimitOrderAggregationAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = LimitOrderAggregationInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> LimitOrderAggregation<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: aggregate limit orders into order book state.
        Ok(())
    }
}

/// Accounts expected by LimitOrderAggregation.
pub struct LimitOrderAggregationAccounts<'a> {
    /// Market configuration account.
    pub market_config: &'a AccountInfo,
    /// Limit order queue state account.
    pub limit_order_queue: &'a AccountInfo,
    /// Slab allocator backing order book nodes.
    pub slab_allocator: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for LimitOrderAggregationAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            market_config: &accounts[0],
            limit_order_queue: &accounts[1],
            slab_allocator: &accounts[2],
        })
    }
}

/// Instruction args for LimitOrderAggregation.
pub struct LimitOrderAggregationInstructionData {
    /// TODO: define aggregation parameters (batch size, cursor).
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for LimitOrderAggregationInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
