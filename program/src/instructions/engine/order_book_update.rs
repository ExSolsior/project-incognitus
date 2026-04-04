/// Purpose: apply queued updates to the on-chain order book.
/// TODO: finalize update parameters and batching logic.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for OrderBookUpdate.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// OrderBookUpdate instruction wrapper with parsed accounts and data.
pub struct OrderBookUpdate<'a> {
    pub accounts: OrderBookUpdateAccounts<'a>,
    pub data: OrderBookUpdateInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for OrderBookUpdate<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = OrderBookUpdateAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = OrderBookUpdateInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> OrderBookUpdate<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: update order book state from queued changes.
        Ok(())
    }
}

/// Accounts expected by OrderBookUpdate.
pub struct OrderBookUpdateAccounts<'a> {
    /// Market configuration account.
    pub market_config: &'a AccountInfo,
    /// Limit order queue state account.
    pub limit_order_queue: &'a AccountInfo,
    /// Slab allocator backing order book nodes.
    pub slab_allocator: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for OrderBookUpdateAccounts<'a> {
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

/// Instruction args for OrderBookUpdate.
pub struct OrderBookUpdateInstructionData {
    /// TODO: define update parameters (batch limits, cursor).
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for OrderBookUpdateInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
