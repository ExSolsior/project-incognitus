#![no_std]
#![allow(unexpected_cfgs)]

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    nostd_panic_handler,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

pub mod instructions;
pub mod state;

/// Program id (placeholder for local development).
pub const ID: Pubkey = [0u8; 32];

entrypoint!(process_instruction);
nostd_panic_handler!();

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Split leading discriminator byte from instruction payload.
    let (_discriminator, _data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // TODO: wire instruction dispatch once discriminators are finalized.
    Err(ProgramError::InvalidInstructionData)
}
