/// Purpose: enqueue or update limit orders for the order book.
/// TODO: validate order parameters and finalize limit order args.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for LimitOrderQueue.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// LimitOrderQueue instruction wrapper with parsed accounts and data.
pub struct LimitOrderQueue<'a> {
    pub accounts: LimitOrderQueueAccounts<'a>,
    pub data: LimitOrderQueueInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for LimitOrderQueue<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = LimitOrderQueueAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = LimitOrderQueueInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> LimitOrderQueue<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: enqueue or update limit orders in the limit order queue.
        Ok(())
    }
}

/// Accounts expected by LimitOrderQueue.
pub struct LimitOrderQueueAccounts<'a> {
    /// Market configuration account.
    pub market_config: &'a AccountInfo,
    /// Limit order queue state account.
    pub limit_order_queue: &'a AccountInfo,
    /// Account ledger tracking open orders.
    pub account_ledger_orders: &'a AccountInfo,
    /// Authority or user submitting the order.
    pub authority: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for LimitOrderQueueAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            market_config: &accounts[0],
            limit_order_queue: &accounts[1],
            account_ledger_orders: &accounts[2],
            authority: &accounts[3],
        })
    }
}

/// Instruction args for LimitOrderQueue.
pub struct LimitOrderQueueInstructionData {
    /// TODO: define order parameters (side, price, size, client id).
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for LimitOrderQueueInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
