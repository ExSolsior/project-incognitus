#[cfg(feature = "bpf-entrypoint")]
mod entrypoint {
    use pinocchio::{entrypoint, AccountView, Address, ProgramResult};

    entrypoint!(process_instruction);

    pub fn process_instruction(
        program_id: &Address,
        accounts: &mut [AccountView],
        instruction_data: &[u8],
    ) -> ProgramResult {
        crate::processor::process(program_id, accounts, instruction_data)
    }
}

pub mod error;
pub mod instructions;
pub mod processor;
pub mod state;
