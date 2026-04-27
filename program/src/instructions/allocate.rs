use pinocchio::{
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult, Resize,
};
use pinocchio_system::instructions::Transfer;

use crate::error::IncognitusError;
use crate::instructions::{
    ACCOUNT_TYPE_ARENA_ALLOCATOR, ACCOUNT_TYPE_ASSET_BALANCE_LEDGER, ACCOUNT_TYPE_SLAB_ALLOCATOR,
};
use crate::state::{
    arena_allocator, asset_balance_ledger, read_discriminator, slab_allocator, ALLOCATE_INCREMENT,
    MAX_ACCOUNT_SIZE,
};

/// Dispatch allocate (grow) based on account_type byte in instruction_data[1].
///
/// Instruction data layout:
///   [0]    instruction discriminator (0x01 = Allocate)
///   [1]    account type tag
///
/// Only Arena, Slab, and AssetBalanceLedger support allocation.
/// MarketConfig and AssetOrderStatus are fixed-size.
///
/// Arena and Slab grow in 10 KB increments up to 10 MB.
/// AssetBalanceLedger grows by one asset entry (56 bytes).
pub fn process(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.len() < 2 {
        return Err(IncognitusError::InstructionDataTooShort.into());
    }

    let account_type = instruction_data[1];

    match account_type {
        ACCOUNT_TYPE_ASSET_BALANCE_LEDGER => allocate_asset_balance_ledger(program_id, accounts),
        ACCOUNT_TYPE_ARENA_ALLOCATOR => {
            allocate_arena_or_slab(program_id, accounts, arena_allocator::DISCRIMINATOR)
        }
        ACCOUNT_TYPE_SLAB_ALLOCATOR => {
            allocate_arena_or_slab(program_id, accounts, slab_allocator::DISCRIMINATOR)
        }
        _ => Err(IncognitusError::InvalidInstruction.into()),
    }
}

// AssetBalanceLedger — grow by one asset entry
// Accounts:
//   [0] payer             (signer, writable)
//   [1] ledger_account    (writable, program-owned)
//   [2] system_program    (read)

fn allocate_asset_balance_ledger(
    program_id: &Address,
    accounts: &mut [AccountView],
) -> ProgramResult {
    if accounts.len() < 3 {
        return Err(IncognitusError::WrongNumberOfAccounts.into());
    }

    if !accounts[0].is_signer() {
        return Err(IncognitusError::MissingRequiredSigner.into());
    }

    // Validate ownership and discriminator.
    if accounts[1].owner() != program_id {
        return Err(IncognitusError::InvalidOwner.into());
    }
    if accounts[1].data_len() < 4 {
        return Err(IncognitusError::NotInitialized.into());
    }
    let disc = {
        let data = unsafe { accounts[1].borrow_unchecked() };
        read_discriminator(data)
    };
    if disc != asset_balance_ledger::DISCRIMINATOR {
        return Err(IncognitusError::InvalidDiscriminator.into());
    }

    let current_len = accounts[1].data_len();
    let new_len = current_len + asset_balance_ledger::ASSET_ENTRY_SIZE;

    // Compute additional rent needed.
    let rent = Rent::get()?;
    let current_rent = rent.try_minimum_balance(current_len)?;
    let new_rent = rent.try_minimum_balance(new_len)?;
    let rent_diff = new_rent.saturating_sub(current_rent);

    // Transfer additional lamports if needed.
    if rent_diff > 0 {
        Transfer {
            from: &accounts[0],
            to: &accounts[1],
            lamports: rent_diff,
        }
        .invoke()?;
    }

    // Resize account using pinocchio's Resize trait.
    accounts[1].resize(new_len)?;

    Ok(())
}

// Arena / Slab — grow by 10 KB increment

// Accounts:
//   [0] payer             (signer, writable)
//   [1] target_account    (writable, program-owned)
//   [2] system_program    (read)

fn allocate_arena_or_slab(
    program_id: &Address,
    accounts: &mut [AccountView],
    expected_discriminator: u32,
) -> ProgramResult {
    if accounts.len() < 3 {
        return Err(IncognitusError::WrongNumberOfAccounts.into());
    }

    if !accounts[0].is_signer() {
        return Err(IncognitusError::MissingRequiredSigner.into());
    }

    // Validate ownership and discriminator.
    if accounts[1].owner() != program_id {
        return Err(IncognitusError::InvalidOwner.into());
    }
    if accounts[1].data_len() < 4 {
        return Err(IncognitusError::NotInitialized.into());
    }
    let disc = {
        let data = unsafe { accounts[1].borrow_unchecked() };
        read_discriminator(data)
    };
    if disc != expected_discriminator {
        return Err(IncognitusError::InvalidDiscriminator.into());
    }

    let current_len = accounts[1].data_len() as u64;
    let new_len = current_len + ALLOCATE_INCREMENT;

    // Enforce max size.
    if new_len > MAX_ACCOUNT_SIZE {
        return Err(IncognitusError::ExceedsMaxAccountSize.into());
    }

    // Compute additional rent needed.
    let rent = Rent::get()?;
    let current_rent = rent.try_minimum_balance(current_len as usize)?;
    let new_rent = rent.try_minimum_balance(new_len as usize)?;
    let rent_diff = new_rent.saturating_sub(current_rent);

    if rent_diff > 0 {
        Transfer {
            from: &accounts[0],
            to: &accounts[1],
            lamports: rent_diff,
        }
        .invoke()?;
    }

    // Resize the account.
    accounts[1].resize(new_len as usize)?;

    // For slab accounts: update the allocator_size field in the header
    // and handle HeadNode relocation.
    if expected_discriminator == slab_allocator::DISCRIMINATOR {
        let data = unsafe { accounts[1].borrow_unchecked_mut() };
        let old_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let new_size = new_len as u32;

        // Update allocator_size.
        data[12..16].copy_from_slice(&new_size.to_le_bytes());

        // The old HeadNode was at old_size - STACK_NODE_SIZE.
        // New HeadNode goes at new_size - STACK_NODE_SIZE.
        // Old HeadNode transitions to TailNode.
        let old_head_addr = (old_size - slab_allocator::STACK_NODE_SIZE as u32) as usize;
        let new_head_addr = (new_size - slab_allocator::STACK_NODE_SIZE as u32) as usize;

        // Read old HeadNode's next_seq_index before overwriting.
        let old_next_seq = u32::from_le_bytes([
            data[old_head_addr],
            data[old_head_addr + 1],
            data[old_head_addr + 2],
            data[old_head_addr + 3],
        ]);

        // Convert old HeadNode → TailNode.
        // TailNode: [0xFFFFFFFF, next_stack_node_pointer, 0xFFFFFFFF, 0xFFFFFFFF]
        data[old_head_addr..old_head_addr + 4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        data[old_head_addr + 4..old_head_addr + 8]
            .copy_from_slice(&(new_head_addr as u32).to_le_bytes());
        data[old_head_addr + 8..old_head_addr + 12].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        data[old_head_addr + 12..old_head_addr + 16].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());

        // Write new HeadNode at end.
        // HeadNode: [next_seq_index, 0, 0x00000000, stack_pointer]
        data[new_head_addr..new_head_addr + 4].copy_from_slice(&old_next_seq.to_le_bytes());
        data[new_head_addr + 4..new_head_addr + 8].copy_from_slice(&0u32.to_le_bytes());
        data[new_head_addr + 8..new_head_addr + 12].copy_from_slice(&0u32.to_le_bytes());
        data[new_head_addr + 12..new_head_addr + 16]
            .copy_from_slice(&(old_head_addr as u32).to_le_bytes());

        // Update header stack_pointer to point to new head.
        data[8..12].copy_from_slice(&(new_head_addr as u32).to_le_bytes());

        // Set FLAG_FRAGMENTED_STACK (byte 2) since we now have a tail + head chain.
        let flags = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let new_flags = flags | 0x00FF_0000;
        data[20..24].copy_from_slice(&new_flags.to_le_bytes());

        // If new size == MAX_ACCOUNT_SIZE, clear FLAG_STILL_GROWING (byte 0).
        if new_len == MAX_ACCOUNT_SIZE {
            let flags = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
            let cleared = flags & !0x0000_00FF;
            data[20..24].copy_from_slice(&cleared.to_le_bytes());
        }
    }

    // For arena accounts: update the capacity field.
    if expected_discriminator == arena_allocator::DISCRIMINATOR {
        let data = unsafe { accounts[1].borrow_unchecked_mut() };
        let new_capacity = (new_len as u32) - (arena_allocator::HEADER_SIZE as u32);
        data[12..16].copy_from_slice(&new_capacity.to_le_bytes());
    }

    Ok(())
}
