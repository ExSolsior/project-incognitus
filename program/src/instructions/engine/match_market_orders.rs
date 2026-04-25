/// Purpose: match market orders against the order book and settle fills.
/// TODO: finalize matching parameters and settlement inputs.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for MatchMarketOrders.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// MatchMarketOrders instruction wrapper with parsed accounts and data.
pub struct MatchMarketOrders<'a> {
    pub accounts: MatchMarketOrdersAccounts<'a>,
    pub data: MatchMarketOrdersInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for MatchMarketOrders<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = MatchMarketOrdersAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = MatchMarketOrdersInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> MatchMarketOrders<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: match market orders against the order book and settle fills.
        Ok(())
    }
}

/// Accounts expected by MatchMarketOrders.
pub struct MatchMarketOrdersAccounts<'a> {
    /// Market configuration account.
    pub market_config: &'a AccountInfo,
    /// Market order queue state account.
    pub market_order_queue: &'a AccountInfo,
    /// Limit order queue state account.
    pub limit_order_queue: &'a AccountInfo,
    /// Slab allocator backing order book nodes.
    pub slab_allocator: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for MatchMarketOrdersAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            market_config: &accounts[0],
            market_order_queue: &accounts[1],
            limit_order_queue: &accounts[2],
            slab_allocator: &accounts[3],
        })
    }
}

/// Instruction args for MatchMarketOrders.
pub struct MatchMarketOrdersInstructionData {
    /// TODO: define match parameters (batch size, cursor).
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for MatchMarketOrdersInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
