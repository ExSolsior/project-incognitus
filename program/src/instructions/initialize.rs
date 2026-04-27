use pinocchio::{
    cpi::{Seed, Signer},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::error::IncognitusError;
use crate::instructions::{
    ACCOUNT_TYPE_ARENA_ALLOCATOR, ACCOUNT_TYPE_ASSET_BALANCE_LEDGER,
    ACCOUNT_TYPE_ASSET_ORDER_STATUS, ACCOUNT_TYPE_MARKET_CONFIG, ACCOUNT_TYPE_SLAB_ALLOCATOR,
};
use crate::state::{
    arena_allocator, asset_balance_ledger, asset_order_status, market_config, read_discriminator,
    slab_allocator, write_discriminator, INITIAL_ACCOUNT_SIZE,
};

/// Dispatch initialize based on account_type byte in instruction_data[1].
///
/// Instruction data layout:
///   [0]    instruction discriminator (0x00 = Initialize)
///   [1]    account type tag
///   [2]    bump seed
///   [3..]  type-specific params (e.g. arena_type, slab_type, slab_index)
pub fn process(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.len() < 3 {
        return Err(IncognitusError::InstructionDataTooShort.into());
    }

    let account_type = instruction_data[1];
    let bump = instruction_data[2];

    match account_type {
        ACCOUNT_TYPE_MARKET_CONFIG => initialize_market_config(program_id, accounts, bump),
        ACCOUNT_TYPE_ASSET_ORDER_STATUS => {
            initialize_asset_order_status(program_id, accounts, bump)
        }
        ACCOUNT_TYPE_ASSET_BALANCE_LEDGER => {
            initialize_asset_balance_ledger(program_id, accounts, bump)
        }
        ACCOUNT_TYPE_ARENA_ALLOCATOR => {
            if instruction_data.len() < 4 {
                return Err(IncognitusError::InstructionDataTooShort.into());
            }
            initialize_arena_allocator(program_id, accounts, bump, instruction_data[3])
        }
        ACCOUNT_TYPE_SLAB_ALLOCATOR => {
            if instruction_data.len() < 5 {
                return Err(IncognitusError::InstructionDataTooShort.into());
            }
            initialize_slab_allocator(
                program_id,
                accounts,
                bump,
                instruction_data[3],
                instruction_data[4],
            )
        }
        _ => Err(IncognitusError::InvalidInstruction.into()),
    }
}

// MarketConfig

// Accounts:
//   [0] payer          (signer, writable)
//   [1] market_config  (writable, PDA)
//   [2] base_mint      (read)
//   [3] quote_mint     (read)
//   [4] system_program (read)

fn initialize_market_config(
    program_id: &Address,
    accounts: &mut [AccountView],
    bump: u8,
) -> ProgramResult {
    if accounts.len() < 5 {
        return Err(IncognitusError::WrongNumberOfAccounts.into());
    }

    if !accounts[0].is_signer() {
        return Err(IncognitusError::MissingRequiredSigner.into());
    }

    let base_mint_key = *accounts[2].address();
    let quote_mint_key = *accounts[3].address();

    // Derive and validate PDA.
    let expected = Address::derive_address(
        &[
            market_config::SEED_PREFIX,
            base_mint_key.as_ref(),
            quote_mint_key.as_ref(),
        ],
        Some(bump),
        program_id,
    );
    if accounts[1].address() != &expected {
        return Err(IncognitusError::InvalidPda.into());
    }

    // Guard against re-initialization.
    if accounts[1].data_len() > 0 {
        return Err(IncognitusError::AlreadyInitialized.into());
    }

    let space = market_config::MIN_DATA_LEN as u64;
    let bump_bytes = [bump];
    let seeds: [Seed; 4] = [
        Seed::from(market_config::SEED_PREFIX),
        Seed::from(base_mint_key.as_ref()),
        Seed::from(quote_mint_key.as_ref()),
        Seed::from(bump_bytes.as_ref()),
    ];
    let signer = Signer::from(&seeds);

    CreateAccount::with_minimum_balance(&accounts[0], &accounts[1], space, program_id, None)?
        .invoke_signed(&[signer])?;

    // Write discriminator.
    let data = unsafe { accounts[1].borrow_unchecked_mut() };
    write_discriminator(data, market_config::DISCRIMINATOR);

    Ok(())
}

// AssetOrderStatus

// Accounts:
//   [0] payer                (signer, writable)
//   [1] asset_order_status   (writable, PDA)
//   [2] market_config        (read)
//   [3] owner                (signer)
//   [4] ledger               (read)
//   [5] system_program       (read)

fn initialize_asset_order_status(
    program_id: &Address,
    accounts: &mut [AccountView],
    bump: u8,
) -> ProgramResult {
    if accounts.len() < 6 {
        return Err(IncognitusError::WrongNumberOfAccounts.into());
    }

    if !accounts[0].is_signer() {
        return Err(IncognitusError::MissingRequiredSigner.into());
    }
    if !accounts[3].is_signer() {
        return Err(IncognitusError::MissingRequiredSigner.into());
    }

    // Validate market_config is owned by this program and initialized.
    if accounts[2].owner() != program_id {
        return Err(IncognitusError::InvalidOwner.into());
    }
    if accounts[2].data_len() < 4 {
        return Err(IncognitusError::NotInitialized.into());
    }
    let mc_disc = {
        let d = unsafe { accounts[2].borrow_unchecked() };
        read_discriminator(d)
    };
    if mc_disc != market_config::DISCRIMINATOR {
        return Err(IncognitusError::InvalidDiscriminator.into());
    }

    let mc_key = *accounts[2].address();
    let owner_key = *accounts[3].address();
    let ledger_key = *accounts[4].address();

    // Derive PDA.
    let expected = Address::derive_address(
        &[
            asset_order_status::SEED_PREFIX,
            mc_key.as_ref(),
            owner_key.as_ref(),
            ledger_key.as_ref(),
        ],
        Some(bump),
        program_id,
    );
    if accounts[1].address() != &expected {
        return Err(IncognitusError::InvalidPda.into());
    }

    if accounts[1].data_len() > 0 {
        return Err(IncognitusError::AlreadyInitialized.into());
    }

    let space = asset_order_status::MIN_DATA_LEN as u64;
    let bump_bytes = [bump];
    let seeds: [Seed; 5] = [
        Seed::from(asset_order_status::SEED_PREFIX),
        Seed::from(mc_key.as_ref()),
        Seed::from(owner_key.as_ref()),
        Seed::from(ledger_key.as_ref()),
        Seed::from(bump_bytes.as_ref()),
    ];
    let signer = Signer::from(&seeds);

    CreateAccount::with_minimum_balance(&accounts[0], &accounts[1], space, program_id, None)?
        .invoke_signed(&[signer])?;

    let data = unsafe { accounts[1].borrow_unchecked_mut() };
    write_discriminator(data, asset_order_status::DISCRIMINATOR);

    Ok(())
}

// AssetBalanceLedger

// Accounts:
//   [0] payer                  (signer, writable)
//   [1] asset_balance_ledger   (writable, PDA)
//   [2] owner                  (signer)
//   [3] system_program         (read)

fn initialize_asset_balance_ledger(
    program_id: &Address,
    accounts: &mut [AccountView],
    bump: u8,
) -> ProgramResult {
    if accounts.len() < 4 {
        return Err(IncognitusError::WrongNumberOfAccounts.into());
    }

    if !accounts[0].is_signer() {
        return Err(IncognitusError::MissingRequiredSigner.into());
    }
    if !accounts[2].is_signer() {
        return Err(IncognitusError::MissingRequiredSigner.into());
    }

    let owner_key = *accounts[2].address();

    let expected = Address::derive_address(
        &[asset_balance_ledger::SEED_PREFIX, owner_key.as_ref()],
        Some(bump),
        program_id,
    );
    if accounts[1].address() != &expected {
        return Err(IncognitusError::InvalidPda.into());
    }

    if accounts[1].data_len() > 0 {
        return Err(IncognitusError::AlreadyInitialized.into());
    }

    let space = asset_balance_ledger::MIN_DATA_LEN as u64;
    let bump_bytes = [bump];
    let seeds: [Seed; 3] = [
        Seed::from(asset_balance_ledger::SEED_PREFIX),
        Seed::from(owner_key.as_ref()),
        Seed::from(bump_bytes.as_ref()),
    ];
    let signer = Signer::from(&seeds);

    CreateAccount::with_minimum_balance(&accounts[0], &accounts[1], space, program_id, None)?
        .invoke_signed(&[signer])?;

    let data = unsafe { accounts[1].borrow_unchecked_mut() };
    write_discriminator(data, asset_balance_ledger::DISCRIMINATOR);

    Ok(())
}

// ArenaAllocator

// Accounts:
//   [0] payer             (signer, writable)
//   [1] arena_account     (writable, PDA)
//   [2] market_config     (read)
//   [3] system_program    (read)

fn initialize_arena_allocator(
    program_id: &Address,
    accounts: &mut [AccountView],
    bump: u8,
    arena_type: u8,
) -> ProgramResult {
    if accounts.len() < 4 {
        return Err(IncognitusError::WrongNumberOfAccounts.into());
    }

    if !accounts[0].is_signer() {
        return Err(IncognitusError::MissingRequiredSigner.into());
    }

    // Validate market_config.
    if accounts[2].owner() != program_id {
        return Err(IncognitusError::InvalidOwner.into());
    }
    if accounts[2].data_len() < 4 {
        return Err(IncognitusError::NotInitialized.into());
    }
    let mc_disc = {
        let d = unsafe { accounts[2].borrow_unchecked() };
        read_discriminator(d)
    };
    if mc_disc != market_config::DISCRIMINATOR {
        return Err(IncognitusError::InvalidDiscriminator.into());
    }

    // Validate arena_type.
    match arena_type {
        arena_allocator::ARENA_TYPE_AGGREGATE
        | arena_allocator::ARENA_TYPE_LIMIT_ORDER_QUEUE
        | arena_allocator::ARENA_TYPE_MARKET_ORDER_QUEUE => {}
        _ => return Err(IncognitusError::InvalidInstruction.into()),
    }

    let mc_key = *accounts[2].address();
    let arena_type_bytes = [arena_type];

    let expected = Address::derive_address(
        &[
            arena_allocator::SEED_PREFIX,
            mc_key.as_ref(),
            &arena_type_bytes,
        ],
        Some(bump),
        program_id,
    );
    if accounts[1].address() != &expected {
        return Err(IncognitusError::InvalidPda.into());
    }

    if accounts[1].data_len() > 0 {
        return Err(IncognitusError::AlreadyInitialized.into());
    }

    let space = INITIAL_ACCOUNT_SIZE;
    let bump_bytes = [bump];
    let seeds: [Seed; 4] = [
        Seed::from(arena_allocator::SEED_PREFIX),
        Seed::from(mc_key.as_ref()),
        Seed::from(arena_type_bytes.as_ref()),
        Seed::from(bump_bytes.as_ref()),
    ];
    let signer = Signer::from(&seeds);

    CreateAccount::with_minimum_balance(&accounts[0], &accounts[1], space, program_id, None)?
        .invoke_signed(&[signer])?;

    // Write header.
    let data = unsafe { accounts[1].borrow_unchecked_mut() };
    write_discriminator(data, arena_allocator::DISCRIMINATOR);
    // write_offset = HEADER_SIZE
    data[4..8].copy_from_slice(&(arena_allocator::HEADER_SIZE as u32).to_le_bytes());
    // read_offset = HEADER_SIZE
    data[8..12].copy_from_slice(&(arena_allocator::HEADER_SIZE as u32).to_le_bytes());
    // capacity = space - HEADER_SIZE
    let capacity = (space as u32) - (arena_allocator::HEADER_SIZE as u32);
    data[12..16].copy_from_slice(&capacity.to_le_bytes());
    // flags = 0
    data[16..20].copy_from_slice(&0u32.to_le_bytes());
    // reserved = 0
    data[20..24].copy_from_slice(&0u32.to_le_bytes());

    Ok(())
}

// SlabAllocator

// Accounts:
//   [0] payer             (signer, writable)
//   [1] slab_account      (writable, PDA)
//   [2] market_config     (read)
//   [3] system_program    (read)

fn initialize_slab_allocator(
    program_id: &Address,
    accounts: &mut [AccountView],
    bump: u8,
    slab_type: u8,
    slab_index: u8,
) -> ProgramResult {
    if accounts.len() < 4 {
        return Err(IncognitusError::WrongNumberOfAccounts.into());
    }

    if !accounts[0].is_signer() {
        return Err(IncognitusError::MissingRequiredSigner.into());
    }

    // Validate market_config.
    if accounts[2].owner() != program_id {
        return Err(IncognitusError::InvalidOwner.into());
    }
    if accounts[2].data_len() < 4 {
        return Err(IncognitusError::NotInitialized.into());
    }
    let mc_disc = {
        let d = unsafe { accounts[2].borrow_unchecked() };
        read_discriminator(d)
    };
    if mc_disc != market_config::DISCRIMINATOR {
        return Err(IncognitusError::InvalidDiscriminator.into());
    }

    // Validate slab_type.
    match slab_type {
        slab_allocator::SLAB_TYPE_PRICE_TREE
        | slab_allocator::SLAB_TYPE_PRICE_STATUS_NODE
        | slab_allocator::SLAB_TYPE_ORDER_BLOCK_NODE
        | slab_allocator::SLAB_TYPE_ORDER_BLOCK_LINKED_NODE
        | slab_allocator::SLAB_TYPE_LEDGER_STATUS
        | slab_allocator::SLAB_TYPE_ORDER_ENTRY_STATUS
        | slab_allocator::SLAB_TYPE_ENTRY_STATUS
        | slab_allocator::SLAB_TYPE_ORDER_ENTRY_STATUS_LINKED
        | slab_allocator::SLAB_TYPE_ENTRY_STATUS_LINKED => {}
        _ => return Err(IncognitusError::InvalidInstruction.into()),
    }

    let mc_key = *accounts[2].address();
    let slab_type_bytes = [slab_type];
    let slab_index_bytes = [slab_index];

    let expected = Address::derive_address(
        &[
            slab_allocator::SEED_PREFIX,
            mc_key.as_ref(),
            &slab_type_bytes,
            &slab_index_bytes,
        ],
        Some(bump),
        program_id,
    );
    if accounts[1].address() != &expected {
        return Err(IncognitusError::InvalidPda.into());
    }

    if accounts[1].data_len() > 0 {
        return Err(IncognitusError::AlreadyInitialized.into());
    }

    let space = INITIAL_ACCOUNT_SIZE;
    let bump_bytes = [bump];
    let signer_seeds: [Seed; 5] = [
        Seed::from(slab_allocator::SEED_PREFIX),
        Seed::from(mc_key.as_ref()),
        Seed::from(slab_type_bytes.as_ref()),
        Seed::from(slab_index_bytes.as_ref()),
        Seed::from(bump_bytes.as_ref()),
    ];
    let signer = Signer::from(&signer_seeds);

    CreateAccount::with_minimum_balance(&accounts[0], &accounts[1], space, program_id, None)?
        .invoke_signed(&[signer])?;

    // Write slab header per Section 17 of architecture doc.
    let data = unsafe { accounts[1].borrow_unchecked_mut() };
    let total_size = space as u32;

    // [0..4] discriminator
    write_discriminator(data, slab_allocator::DISCRIMINATOR);
    // [4..8] root_node_pointer = 0x0000_0000 (no root yet)
    data[4..8].copy_from_slice(&0u32.to_le_bytes());
    // [8..12] stack_pointer → HeadNode at end of account
    let head_node_addr = total_size - (slab_allocator::STACK_NODE_SIZE as u32);
    data[8..12].copy_from_slice(&head_node_addr.to_le_bytes());
    // [12..16] allocator_size
    data[12..16].copy_from_slice(&total_size.to_le_bytes());
    // [16..20] num_node_elements = 0
    data[16..20].copy_from_slice(&0u32.to_le_bytes());
    // [20..24] flags = 0x0000_00FF (FLAG_STILL_GROWING)
    data[20..24].copy_from_slice(&0x0000_00FFu32.to_le_bytes());

    // Write HeadNode at end of account (16 bytes).
    let hn = head_node_addr as usize;
    let header_size = slab_allocator::HEADER_SIZE as u32;
    data[hn..hn + 4].copy_from_slice(&header_size.to_le_bytes()); // next_seq_index
    data[hn + 4..hn + 8].copy_from_slice(&0u32.to_le_bytes()); // stack_entry_cycle_pointer
    data[hn + 8..hn + 12].copy_from_slice(&0u32.to_le_bytes()); // stack_node_type_flag (head)
    data[hn + 12..hn + 16].copy_from_slice(&0u32.to_le_bytes()); // single_stack_node_flag

    Ok(())
}
