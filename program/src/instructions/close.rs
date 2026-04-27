use pinocchio::{AccountView, Address, ProgramResult};

use crate::error::IncognitusError;
use crate::instructions::{
    ACCOUNT_TYPE_ARENA_ALLOCATOR, ACCOUNT_TYPE_ASSET_BALANCE_LEDGER,
    ACCOUNT_TYPE_ASSET_ORDER_STATUS, ACCOUNT_TYPE_MARKET_CONFIG, ACCOUNT_TYPE_SLAB_ALLOCATOR,
};
use crate::state::{
    arena_allocator, asset_balance_ledger, asset_order_status, market_config, read_discriminator,
    slab_allocator, write_discriminator, CLOSING_DISCRIMINATOR,
};

/// Maximum number of bytes to zero per close transaction.
/// Large accounts (up to 10 MB) cannot be zeroed in a single tx due to CU limits.
/// Each close call zeros up to this many bytes, tracked by a close_index.
const ZERO_CHUNK_SIZE: usize = 8 * 1024; // 8 KB per tx — conservative for CU budget

/// Dispatch close based on account_type byte in instruction_data[1].
///
/// Instruction data layout:
///   [0]    instruction discriminator (0x02 = Close)
///   [1]    account type tag
///
/// Close is a multi-step process for large accounts:
///   1. First call: sets CLOSING_DISCRIMINATOR (0xFFFF_FFFF), begins zeroing.
///   2. Subsequent calls: continue zeroing from where the last call left off.
///   3. Final call: all bytes zeroed, lamports transferred to destination,
///      account data zeroed and closed.
///
/// Safe close pattern per Helius guide:
///   - Set discriminator to CLOSING (prevents reuse in same tx)
///   - Zero all data before transferring lamports
///   - Transfer all lamports to destination
///   - The runtime garbage-collects zero-lamport accounts
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
        ACCOUNT_TYPE_MARKET_CONFIG => {
            close_program_account(program_id, accounts, market_config::DISCRIMINATOR)
        }
        ACCOUNT_TYPE_ASSET_ORDER_STATUS => {
            close_program_account(program_id, accounts, asset_order_status::DISCRIMINATOR)
        }
        ACCOUNT_TYPE_ASSET_BALANCE_LEDGER => {
            close_program_account(program_id, accounts, asset_balance_ledger::DISCRIMINATOR)
        }
        ACCOUNT_TYPE_ARENA_ALLOCATOR => {
            close_program_account(program_id, accounts, arena_allocator::DISCRIMINATOR)
        }
        ACCOUNT_TYPE_SLAB_ALLOCATOR => {
            close_program_account(program_id, accounts, slab_allocator::DISCRIMINATOR)
        }
        _ => Err(IncognitusError::InvalidInstruction.into()),
    }
}

// Unified close logic — works for all account types
// Accounts:
//   [0] authority        (signer)
//   [1] target_account   (writable, program-owned)
//   [2] destination      (writable — receives lamports)
//
// The close_index is stored at bytes [4..8] of the account data once
// the CLOSING_DISCRIMINATOR is set. This repurposes the first field
// after the discriminator as a progress tracker.
//
// Close index layout (inside target_account data, only when closing):
//   [0..4]   CLOSING_DISCRIMINATOR (0xFFFF_FFFF)
//   [4..8]   close_index: u32 — byte offset of next chunk to zero

fn close_program_account(
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

    // Validate ownership.
    if accounts[1].owner() != program_id {
        return Err(IncognitusError::InvalidOwner.into());
    }

    let data_len = accounts[1].data_len();
    if data_len < 8 {
        return Err(IncognitusError::InvalidDataLength.into());
    }

    let data = unsafe { accounts[1].borrow_unchecked_mut() };
    let disc = read_discriminator(data);

    // Determine if this is the first close call or a continuation.
    let close_index: usize;

    if disc == CLOSING_DISCRIMINATOR {
        // Continuation — read close_index from [4..8].
        close_index = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    } else if disc == expected_discriminator {
        // First close call — mark as closing and start zeroing from byte 8.
        write_discriminator(data, CLOSING_DISCRIMINATOR);
        close_index = 8;
        data[4..8].copy_from_slice(&(close_index as u32).to_le_bytes());
    } else {
        return Err(IncognitusError::InvalidDiscriminator.into());
    }

    // Zero the next chunk.
    let zero_end = core::cmp::min(close_index + ZERO_CHUNK_SIZE, data_len);

    // Zero bytes in the chunk range.
    for i in close_index..zero_end {
        data[i] = 0;
    }

    if zero_end < data_len {
        // Not done yet — update close_index and return.
        data[4..8].copy_from_slice(&(zero_end as u32).to_le_bytes());
        return Ok(());
    }

    // All data bytes after the header are zeroed. Now zero the header itself.
    for i in 0..8 {
        data[i] = 0;
    }

    // Transfer all lamports to destination.
    // Data is already zeroed — even if this account is referenced again
    // in the same tx, the zero discriminator prevents valid interpretation.
    let lamports = accounts[1].lamports();
    accounts[1].set_lamports(0);
    accounts[2].set_lamports(accounts[2].lamports() + lamports);

    // The Solana runtime garbage-collects zero-lamport accounts
    // at the end of the transaction.

    Ok(())
}
