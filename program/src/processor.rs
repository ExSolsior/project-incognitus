use pinocchio::{AccountView, Address, ProgramResult};

use crate::error::IncognitusError;
use crate::instructions::{IX_ALLOCATE, IX_CLOSE, IX_INITIALIZE};

/// Top-level instruction router.
///
/// Instruction data[0] determines the instruction type:
///   0x00 = Initialize
///   0x01 = Allocate
///   0x02 = Close
pub fn process(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(IncognitusError::InstructionDataTooShort.into());
    }

    match instruction_data[0] {
        IX_INITIALIZE => {
            crate::instructions::initialize::process(program_id, accounts, instruction_data)
        }
        IX_ALLOCATE => {
            crate::instructions::allocate::process(program_id, accounts, instruction_data)
        }
        IX_CLOSE => crate::instructions::close::process(program_id, accounts, instruction_data),
        _ => Err(IncognitusError::InvalidInstruction.into()),
    }
}
