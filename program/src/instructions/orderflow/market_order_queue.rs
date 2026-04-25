/// Purpose: enqueue or update market orders for matching.
/// TODO: validate order parameters and finalize market order args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for MarketOrderQueue.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// MarketOrderQueue instruction wrapper with parsed accounts and data.
pub struct MarketOrderQueue<'a> {
    pub accounts: MarketOrderQueueAccounts<'a>,
    pub data: MarketOrderQueueInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for MarketOrderQueue<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = MarketOrderQueueAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = MarketOrderQueueInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> MarketOrderQueue<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: enqueue or update market orders in the market order queue.
        Ok(())
    }
}

/// Accounts expected by MarketOrderQueue.
pub struct MarketOrderQueueAccounts<'a> {
    /// Market configuration account.
    pub market_config: &'a AccountInfo,
    /// Market order queue state account.
    pub market_order_queue: &'a AccountInfo,
    /// Account ledger tracking open orders.
    pub account_ledger_orders: &'a AccountInfo,
    /// Authority or user submitting the order.
    pub authority: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for MarketOrderQueueAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            market_config: &accounts[0],
            market_order_queue: &accounts[1],
            account_ledger_orders: &accounts[2],
            authority: &accounts[3],
        })
    }
}

/// Instruction args for MarketOrderQueue.
pub struct MarketOrderQueueInstructionData {
    /// TODO: define order parameters (side, size, client id).
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for MarketOrderQueueInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
