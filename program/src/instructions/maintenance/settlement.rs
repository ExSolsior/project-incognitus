/// Purpose: settle matched orders and update ledger balances.
/// TODO: finalize settlement parameters and ledger updates.
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

/// Instruction discriminator for Settlement.
/// TODO: replace placeholder once the instruction set is finalized (currently 0).
pub const DISCRIMINATOR: u8 = 0;

/// Settlement instruction wrapper with parsed accounts and data.
pub struct Settlement<'a> {
    pub accounts: SettlementAccounts<'a>,
    pub data: SettlementInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Settlement<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse and validate account list ordering.
        let accounts = SettlementAccounts::try_from(accounts)?;
        // Parse and validate instruction data.
        let data = SettlementInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }
}

impl<'a> Settlement<'a> {
    pub fn process(&mut self) -> ProgramResult {
        // TODO: settle matched orders and update ledger balances.
        Ok(())
    }
}

/// Accounts expected by Settlement.
pub struct SettlementAccounts<'a> {
    /// Market configuration account.
    pub market_config: &'a AccountInfo,
    /// Settlement queue state account.
    pub settlement_queue: &'a AccountInfo,
    /// Account ledger tracking balances.
    pub account_ledger_addresses: &'a AccountInfo,
    /// Authority authorized to finalize settlement.
    pub authority: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for SettlementAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // TODO: enforce precise ordering and signer/writable checks.
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            market_config: &accounts[0],
            settlement_queue: &accounts[1],
            account_ledger_addresses: &accounts[2],
            authority: &accounts[3],
        })
    }
}

/// Instruction args for Settlement.
pub struct SettlementInstructionData {
    /// TODO: define settlement parameters (batch size, cursor).
    pub _reserved: [u8; 0],
}

impl TryFrom<&[u8]> for SettlementInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: parse instruction args once the layout is finalized.
        if !data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { _reserved: [] })
    }
}
