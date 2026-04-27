/// Integration tests using mollusk-svm.
///
/// These test full instruction execution through the SVM — CPI, PDA
/// derivation, account creation, realloc, and close.
///
/// The program binary is loaded by mollusk from the target directory.
/// Build with `cargo build-sbf` before running these tests.
use mollusk_svm::{result::Check, Mollusk};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

use project_incognitus::{
    error::IncognitusError,
    instructions::*,
    state::{
        arena_allocator, asset_balance_ledger, market_config, read_discriminator, slab_allocator,
        CLOSING_DISCRIMINATOR, INITIAL_ACCOUNT_SIZE,
    },
};

/// Program ID used in tests. Arbitrary but deterministic.
fn program_id() -> Pubkey {
    Pubkey::new_from_array([
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ])
}

fn setup() -> Mollusk {
    Mollusk::new(&program_id(), "project_incognitus")
}

/// Rich payer with enough lamports.
fn payer() -> (Pubkey, Account) {
    let key = Pubkey::new_unique();
    let account = Account::new(10_000_000_000, 0, &solana_sdk_ids::system_program::ID);
    (key, account)
}

/// System program keyed account for CPI.
fn system_program_account() -> (Pubkey, Account) {
    mollusk_svm::program::keyed_account_for_system_program()
}

fn custom_err(e: IncognitusError) -> ProgramError {
    ProgramError::Custom(e as u32)
}

// =======================================================================
// Initialize — MarketConfig
// =======================================================================

#[test]
fn initialize_market_config_success() {
    let mollusk = setup();
    let pid = program_id();
    let (payer_key, payer_acc) = payer();
    let base_mint = Pubkey::new_unique();
    let quote_mint = Pubkey::new_unique();

    let (mc_key, bump) = Pubkey::find_program_address(
        &[
            market_config::SEED_PREFIX,
            base_mint.as_ref(),
            quote_mint.as_ref(),
        ],
        &pid,
    );

    let ix_data = vec![IX_INITIALIZE, ACCOUNT_TYPE_MARKET_CONFIG, bump];

    let instruction = Instruction::new_with_bytes(
        pid,
        &ix_data,
        vec![
            AccountMeta::new(payer_key, true),
            AccountMeta::new(mc_key, true),
            AccountMeta::new_readonly(base_mint, false),
            AccountMeta::new_readonly(quote_mint, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    );

    let accounts = vec![
        (payer_key, payer_acc),
        (mc_key, Account::default()),
        (base_mint, Account::default()),
        (quote_mint, Account::default()),
        system_program_account(),
    ];

    let result =
        mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

    // Verify resulting account.
    let mc_result = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| k == &mc_key)
        .expect("market_config not in results");

    assert_eq!(mc_result.1.data.len(), market_config::MIN_DATA_LEN);
    assert_eq!(
        read_discriminator(&mc_result.1.data),
        market_config::DISCRIMINATOR
    );
    assert_eq!(mc_result.1.owner, pid);
}

#[test]
fn initialize_market_config_wrong_pda_fails() {
    let mollusk = setup();
    let pid = program_id();
    let (payer_key, payer_acc) = payer();
    let base_mint = Pubkey::new_unique();
    let quote_mint = Pubkey::new_unique();

    let (mc_key, correct_bump) = Pubkey::find_program_address(
        &[
            market_config::SEED_PREFIX,
            base_mint.as_ref(),
            quote_mint.as_ref(),
        ],
        &pid,
    );
    let wrong_bump = correct_bump.wrapping_sub(1);

    let ix_data = vec![IX_INITIALIZE, ACCOUNT_TYPE_MARKET_CONFIG, wrong_bump];

    let instruction = Instruction::new_with_bytes(
        pid,
        &ix_data,
        vec![
            AccountMeta::new(payer_key, true),
            AccountMeta::new(mc_key, true),
            AccountMeta::new_readonly(base_mint, false),
            AccountMeta::new_readonly(quote_mint, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    );

    let accounts = vec![
        (payer_key, payer_acc),
        (mc_key, Account::default()),
        (base_mint, Account::default()),
        (quote_mint, Account::default()),
        system_program_account(),
    ];

    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(custom_err(IncognitusError::InvalidPda))],
    );
}

#[test]
fn initialize_market_config_already_initialized_fails() {
    let mollusk = setup();
    let pid = program_id();
    let (payer_key, payer_acc) = payer();
    let base_mint = Pubkey::new_unique();
    let quote_mint = Pubkey::new_unique();

    let (mc_key, bump) = Pubkey::find_program_address(
        &[
            market_config::SEED_PREFIX,
            base_mint.as_ref(),
            quote_mint.as_ref(),
        ],
        &pid,
    );

    let mut existing = Account::new(1_000_000, market_config::MIN_DATA_LEN, &pid);
    existing.data[0..4].copy_from_slice(&market_config::DISCRIMINATOR.to_le_bytes());

    let ix_data = vec![IX_INITIALIZE, ACCOUNT_TYPE_MARKET_CONFIG, bump];

    let instruction = Instruction::new_with_bytes(
        pid,
        &ix_data,
        vec![
            AccountMeta::new(payer_key, true),
            AccountMeta::new(mc_key, true),
            AccountMeta::new_readonly(base_mint, false),
            AccountMeta::new_readonly(quote_mint, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    );

    let accounts = vec![
        (payer_key, payer_acc),
        (mc_key, existing),
        (base_mint, Account::default()),
        (quote_mint, Account::default()),
        system_program_account(),
    ];

    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(custom_err(IncognitusError::AlreadyInitialized))],
    );
}

// =======================================================================
// Initialize — AssetBalanceLedger
// =======================================================================

#[test]
fn initialize_asset_balance_ledger_success() {
    let mollusk = setup();
    let pid = program_id();
    let (payer_key, payer_acc) = payer();
    let owner = Pubkey::new_unique();

    let (ledger_key, bump) =
        Pubkey::find_program_address(&[asset_balance_ledger::SEED_PREFIX, owner.as_ref()], &pid);

    let ix_data = vec![IX_INITIALIZE, ACCOUNT_TYPE_ASSET_BALANCE_LEDGER, bump];

    let instruction = Instruction::new_with_bytes(
        pid,
        &ix_data,
        vec![
            AccountMeta::new(payer_key, true),
            AccountMeta::new(ledger_key, true),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    );

    let accounts = vec![
        (payer_key, payer_acc),
        (ledger_key, Account::default()),
        (owner, Account::default()),
        system_program_account(),
    ];

    let result =
        mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

    let ledger = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| k == &ledger_key)
        .unwrap();

    assert_eq!(ledger.1.data.len(), asset_balance_ledger::MIN_DATA_LEN);
    assert_eq!(
        read_discriminator(&ledger.1.data),
        asset_balance_ledger::DISCRIMINATOR,
    );
}

// =======================================================================
// Initialize — ArenaAllocator
// =======================================================================

#[test]
fn initialize_arena_allocator_success() {
    let mollusk = setup();
    let pid = program_id();
    let (payer_key, payer_acc) = payer();
    let base_mint = Pubkey::new_unique();
    let quote_mint = Pubkey::new_unique();

    let (mc_key, _) = Pubkey::find_program_address(
        &[
            market_config::SEED_PREFIX,
            base_mint.as_ref(),
            quote_mint.as_ref(),
        ],
        &pid,
    );

    let mut mc_account = Account::new(1_000_000, market_config::MIN_DATA_LEN, &pid);
    mc_account.data[0..4].copy_from_slice(&market_config::DISCRIMINATOR.to_le_bytes());

    let arena_type = arena_allocator::ARENA_TYPE_AGGREGATE;
    let (arena_key, bump) = Pubkey::find_program_address(
        &[arena_allocator::SEED_PREFIX, mc_key.as_ref(), &[arena_type]],
        &pid,
    );

    let ix_data = vec![
        IX_INITIALIZE,
        ACCOUNT_TYPE_ARENA_ALLOCATOR,
        bump,
        arena_type,
    ];

    let instruction = Instruction::new_with_bytes(
        pid,
        &ix_data,
        vec![
            AccountMeta::new(payer_key, true),
            AccountMeta::new(arena_key, true),
            AccountMeta::new_readonly(mc_key, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    );

    let accounts = vec![
        (payer_key, payer_acc),
        (arena_key, Account::default()),
        (mc_key, mc_account),
        system_program_account(),
    ];

    let result =
        mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

    let arena = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| k == &arena_key)
        .unwrap();
    let data = &arena.1.data;

    assert_eq!(data.len(), INITIAL_ACCOUNT_SIZE as usize);
    assert_eq!(read_discriminator(data), arena_allocator::DISCRIMINATOR);

    let write_off = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let read_off = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let capacity = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

    assert_eq!(write_off, arena_allocator::HEADER_SIZE as u32);
    assert_eq!(read_off, arena_allocator::HEADER_SIZE as u32);
    assert_eq!(
        capacity,
        (INITIAL_ACCOUNT_SIZE as u32) - (arena_allocator::HEADER_SIZE as u32)
    );
}

#[test]
fn initialize_arena_invalid_type_fails() {
    let mollusk = setup();
    let pid = program_id();
    let (payer_key, payer_acc) = payer();
    let arena_key = Pubkey::new_unique();

    let (mc_key, _) =
        Pubkey::find_program_address(&[market_config::SEED_PREFIX, &[1; 32], &[2; 32]], &pid);
    let mut mc_acc = Account::new(1_000_000, market_config::MIN_DATA_LEN, &pid);
    mc_acc.data[0..4].copy_from_slice(&market_config::DISCRIMINATOR.to_le_bytes());

    let ix_data = vec![IX_INITIALIZE, ACCOUNT_TYPE_ARENA_ALLOCATOR, 0, 0xFF];

    let instruction = Instruction::new_with_bytes(
        pid,
        &ix_data,
        vec![
            AccountMeta::new(payer_key, true),
            AccountMeta::new(arena_key, true),
            AccountMeta::new_readonly(mc_key, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    );

    let accounts = vec![
        (payer_key, payer_acc),
        (arena_key, Account::default()),
        (mc_key, mc_acc),
        system_program_account(),
    ];

    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(custom_err(IncognitusError::InvalidInstruction))],
    );
}

// =======================================================================
// Initialize — SlabAllocator
// =======================================================================

#[test]
fn initialize_slab_allocator_success() {
    let mollusk = setup();
    let pid = program_id();
    let (payer_key, payer_acc) = payer();

    let (mc_key, _) =
        Pubkey::find_program_address(&[market_config::SEED_PREFIX, &[1; 32], &[2; 32]], &pid);
    let mut mc_acc = Account::new(1_000_000, market_config::MIN_DATA_LEN, &pid);
    mc_acc.data[0..4].copy_from_slice(&market_config::DISCRIMINATOR.to_le_bytes());

    let slab_type = slab_allocator::SLAB_TYPE_PRICE_TREE;
    let slab_index = 0u8;
    let (slab_key, bump) = Pubkey::find_program_address(
        &[
            slab_allocator::SEED_PREFIX,
            mc_key.as_ref(),
            &[slab_type],
            &[slab_index],
        ],
        &pid,
    );

    let ix_data = vec![
        IX_INITIALIZE,
        ACCOUNT_TYPE_SLAB_ALLOCATOR,
        bump,
        slab_type,
        slab_index,
    ];

    let instruction = Instruction::new_with_bytes(
        pid,
        &ix_data,
        vec![
            AccountMeta::new(payer_key, true),
            AccountMeta::new(slab_key, true),
            AccountMeta::new_readonly(mc_key, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    );

    let accounts = vec![
        (payer_key, payer_acc),
        (slab_key, Account::default()),
        (mc_key, mc_acc),
        system_program_account(),
    ];

    let result =
        mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

    let slab = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| k == &slab_key)
        .unwrap();
    let data = &slab.1.data;
    let total_size = INITIAL_ACCOUNT_SIZE as u32;

    assert_eq!(data.len(), INITIAL_ACCOUNT_SIZE as usize);
    assert_eq!(read_discriminator(data), slab_allocator::DISCRIMINATOR);

    // Verify header.
    let root_ptr = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let stack_ptr = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let alloc_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let num_elements = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    let flags = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);

    let expected_head = total_size - slab_allocator::STACK_NODE_SIZE as u32;

    assert_eq!(root_ptr, 0);
    assert_eq!(stack_ptr, expected_head);
    assert_eq!(alloc_size, total_size);
    assert_eq!(num_elements, 0);
    assert_eq!(flags, 0x0000_00FF); // FLAG_STILL_GROWING

    // HeadNode at end.
    let hn = expected_head as usize;
    let next_seq = u32::from_le_bytes([data[hn], data[hn + 1], data[hn + 2], data[hn + 3]]);
    let type_flag = u32::from_le_bytes([data[hn + 8], data[hn + 9], data[hn + 10], data[hn + 11]]);

    assert_eq!(next_seq, slab_allocator::HEADER_SIZE as u32);
    assert_eq!(type_flag, 0); // head
}

// =======================================================================
// Close — small account (single-tx)
// =======================================================================

#[test]
fn close_small_account_single_tx() {
    let mollusk = setup();
    let pid = program_id();
    let authority = Pubkey::new_unique();
    let target = Pubkey::new_unique();
    let destination = Pubkey::new_unique();

    let target_lamports = 1_000_000u64;
    let dest_lamports = 500_000u64;

    let mut target_acc = Account::new(target_lamports, market_config::MIN_DATA_LEN, &pid);
    target_acc.data[0..4].copy_from_slice(&market_config::DISCRIMINATOR.to_le_bytes());

    let ix_data = vec![IX_CLOSE, ACCOUNT_TYPE_MARKET_CONFIG];

    let instruction = Instruction::new_with_bytes(
        pid,
        &ix_data,
        vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(target, false),
            AccountMeta::new(destination, false),
        ],
    );

    let accounts = vec![
        (authority, Account::default()),
        (target, target_acc),
        (
            destination,
            Account::new(dest_lamports, 0, &solana_sdk_ids::system_program::ID),
        ),
    ];

    let result =
        mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

    let target_res = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| k == &target)
        .unwrap();
    assert_eq!(target_res.1.lamports, 0);
    assert!(target_res.1.data.iter().all(|&b| b == 0));

    let dest_res = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| k == &destination)
        .unwrap();
    assert_eq!(dest_res.1.lamports, dest_lamports + target_lamports);
}

// =======================================================================
// Close — large account (multi-tx chunked zeroing)
// =======================================================================

#[test]
fn close_large_account_multi_tx() {
    let mollusk = setup();
    let pid = program_id();
    let authority = Pubkey::new_unique();
    let target = Pubkey::new_unique();
    let destination = Pubkey::new_unique();

    let data_len = 20_000usize; // > 8KB chunk → 3 calls needed
    let target_lamports = 5_000_000u64;

    let mut target_acc = Account::new(target_lamports, data_len, &pid);
    target_acc.data[0..4].copy_from_slice(&arena_allocator::DISCRIMINATOR.to_le_bytes());
    for b in target_acc.data[4..].iter_mut() {
        *b = 0xAB;
    }

    let ix_data = vec![IX_CLOSE, ACCOUNT_TYPE_ARENA_ALLOCATOR];

    let instruction = Instruction::new_with_bytes(
        pid,
        &ix_data,
        vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(target, false),
            AccountMeta::new(destination, false),
        ],
    );

    let accounts = vec![
        (authority, Account::default()),
        (target, target_acc),
        (
            destination,
            Account::new(100_000, 0, &solana_sdk_ids::system_program::ID),
        ),
    ];

    // Call 1: sets CLOSING disc, zeros [8..8200].
    let r1 = mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

    let a1 = r1
        .resulting_accounts
        .iter()
        .find(|(k, _)| k == &target)
        .unwrap();
    assert_eq!(read_discriminator(&a1.1.data), CLOSING_DISCRIMINATOR);
    let ci1 = u32::from_le_bytes([a1.1.data[4], a1.1.data[5], a1.1.data[6], a1.1.data[7]]);
    assert_eq!(ci1, 8 + 8192); // 8200
    assert_eq!(a1.1.lamports, target_lamports); // still open

    // Call 2: zeros [8200..16392].
    let r2 = mollusk.process_and_validate_instruction(
        &instruction,
        &r1.resulting_accounts,
        &[Check::success()],
    );

    let a2 = r2
        .resulting_accounts
        .iter()
        .find(|(k, _)| k == &target)
        .unwrap();
    let ci2 = u32::from_le_bytes([a2.1.data[4], a2.1.data[5], a2.1.data[6], a2.1.data[7]]);
    assert_eq!(ci2, 8200 + 8192);
    assert_eq!(a2.1.lamports, target_lamports);

    // Call 3: zeros remaining, drains lamports.
    let r3 = mollusk.process_and_validate_instruction(
        &instruction,
        &r2.resulting_accounts,
        &[Check::success()],
    );

    let a3 = r3
        .resulting_accounts
        .iter()
        .find(|(k, _)| k == &target)
        .unwrap();
    assert_eq!(a3.1.lamports, 0);
    assert!(a3.1.data.iter().all(|&b| b == 0));
}

// =======================================================================
// Close — wrong discriminator fails
// =======================================================================

#[test]
fn close_wrong_discriminator_fails() {
    let mollusk = setup();
    let pid = program_id();
    let authority = Pubkey::new_unique();
    let target = Pubkey::new_unique();
    let destination = Pubkey::new_unique();

    let mut target_acc = Account::new(1_000_000, 100, &pid);
    target_acc.data[0..4].copy_from_slice(&slab_allocator::DISCRIMINATOR.to_le_bytes());

    let instruction = Instruction::new_with_bytes(
        pid,
        &[IX_CLOSE, ACCOUNT_TYPE_MARKET_CONFIG],
        vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(target, false),
            AccountMeta::new(destination, false),
        ],
    );

    let accounts = vec![
        (authority, Account::default()),
        (target, target_acc),
        (destination, Account::default()),
    ];

    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(custom_err(
            IncognitusError::InvalidDiscriminator,
        ))],
    );
}

// =======================================================================
// Close — missing signer fails
// =======================================================================

#[test]
fn close_missing_signer_fails() {
    let mollusk = setup();
    let pid = program_id();
    let authority = Pubkey::new_unique();
    let target = Pubkey::new_unique();
    let destination = Pubkey::new_unique();

    let mut target_acc = Account::new(1_000_000, 100, &pid);
    target_acc.data[0..4].copy_from_slice(&market_config::DISCRIMINATOR.to_le_bytes());

    let instruction = Instruction::new_with_bytes(
        pid,
        &[IX_CLOSE, ACCOUNT_TYPE_MARKET_CONFIG],
        vec![
            AccountMeta::new_readonly(authority, false), // NOT signer
            AccountMeta::new(target, false),
            AccountMeta::new(destination, false),
        ],
    );

    let accounts = vec![
        (authority, Account::default()),
        (target, target_acc),
        (destination, Account::default()),
    ];

    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(custom_err(
            IncognitusError::MissingRequiredSigner,
        ))],
    );
}

// =======================================================================
// Close — wrong owner fails
// =======================================================================

#[test]
fn close_wrong_owner_fails() {
    let mollusk = setup();
    let pid = program_id();
    let wrong_owner = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let target = Pubkey::new_unique();
    let destination = Pubkey::new_unique();

    let mut target_acc = Account::new(1_000_000, 100, &wrong_owner);
    target_acc.data[0..4].copy_from_slice(&market_config::DISCRIMINATOR.to_le_bytes());

    let instruction = Instruction::new_with_bytes(
        pid,
        &[IX_CLOSE, ACCOUNT_TYPE_MARKET_CONFIG],
        vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(target, false),
            AccountMeta::new(destination, false),
        ],
    );

    let accounts = vec![
        (authority, Account::default()),
        (target, target_acc),
        (destination, Account::default()),
    ];

    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(custom_err(IncognitusError::InvalidOwner))],
    );
}

// =======================================================================
// Dispatch — edge cases
// =======================================================================

#[test]
fn empty_instruction_data_fails() {
    let mollusk = setup();
    mollusk.process_and_validate_instruction(
        &Instruction::new_with_bytes(program_id(), &[], vec![]),
        &[],
        &[Check::err(custom_err(
            IncognitusError::InstructionDataTooShort,
        ))],
    );
}

#[test]
fn invalid_instruction_discriminator_fails() {
    let mollusk = setup();
    mollusk.process_and_validate_instruction(
        &Instruction::new_with_bytes(program_id(), &[0xFF], vec![]),
        &[],
        &[Check::err(custom_err(IncognitusError::InvalidInstruction))],
    );
}

#[test]
fn initialize_too_few_accounts_fails() {
    let mollusk = setup();
    let pid = program_id();
    let (payer_key, payer_acc) = payer();
    let dummy = Pubkey::new_unique();

    let instruction = Instruction::new_with_bytes(
        pid,
        &[IX_INITIALIZE, ACCOUNT_TYPE_MARKET_CONFIG, 0],
        vec![
            AccountMeta::new(payer_key, true),
            AccountMeta::new(dummy, false),
        ],
    );

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(payer_key, payer_acc), (dummy, Account::default())],
        &[Check::err(custom_err(
            IncognitusError::WrongNumberOfAccounts,
        ))],
    );
}

#[test]
fn initialize_truncated_data_fails() {
    let mollusk = setup();
    mollusk.process_and_validate_instruction(
        &Instruction::new_with_bytes(
            program_id(),
            &[IX_INITIALIZE, ACCOUNT_TYPE_MARKET_CONFIG], // missing bump
            vec![],
        ),
        &[],
        &[Check::err(custom_err(
            IncognitusError::InstructionDataTooShort,
        ))],
    );
}

#[test]
fn initialize_arena_truncated_data_fails() {
    let mollusk = setup();
    mollusk.process_and_validate_instruction(
        &Instruction::new_with_bytes(
            program_id(),
            &[IX_INITIALIZE, ACCOUNT_TYPE_ARENA_ALLOCATOR, 0], // missing arena_type
            vec![],
        ),
        &[],
        &[Check::err(custom_err(
            IncognitusError::InstructionDataTooShort,
        ))],
    );
}

#[test]
fn initialize_slab_truncated_data_fails() {
    let mollusk = setup();
    mollusk.process_and_validate_instruction(
        &Instruction::new_with_bytes(
            program_id(),
            &[IX_INITIALIZE, ACCOUNT_TYPE_SLAB_ALLOCATOR, 0, 0x10], // missing slab_index
            vec![],
        ),
        &[],
        &[Check::err(custom_err(
            IncognitusError::InstructionDataTooShort,
        ))],
    );
}

#[test]
fn allocate_invalid_type_fails() {
    let mollusk = setup();
    mollusk.process_and_validate_instruction(
        &Instruction::new_with_bytes(
            program_id(),
            &[IX_ALLOCATE, ACCOUNT_TYPE_MARKET_CONFIG],
            vec![],
        ),
        &[],
        &[Check::err(custom_err(IncognitusError::InvalidInstruction))],
    );
}
