/// Purpose: precompute market order matches and cache results.
/// TODO: finalize precompute parameters and caching layout.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for PrecomputeMarketOrders.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// PrecomputeMarketOrders instruction wrapper with parsed accounts and data.
pub struct PrecomputeMarketOrders<'a> {
    pub accounts: PrecomputeMarketOrdersAccounts<'a>,
    pub data: PrecomputeMarketOrdersInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for PrecomputeMarketOrders<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = PrecomputeMarketOrdersAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = PrecomputeMarketOrdersInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> PrecomputeMarketOrders<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: precompute market order matches and cache the results.
        Ok(())
    }
}

/// Accounts expected by PrecomputeMarketOrders.
pub struct PrecomputeMarketOrdersAccounts<'a> {
    /// Market configuration account.
    pub market_config: &'a AccountInfo,
    /// Market order queue state account.
    pub market_order_queue: &'a AccountInfo,
    /// Market order cache for precomputed results.
    pub market_order_cache: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for PrecomputeMarketOrdersAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            market_config: &accounts[0],
            market_order_queue: &accounts[1],
            market_order_cache: &accounts[2],
        })
    }
}

/// Instruction args for PrecomputeMarketOrders.
pub struct PrecomputeMarketOrdersInstructionData {
    /// TODO: define precompute parameters (batch size, cursor).
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for PrecomputeMarketOrdersInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
