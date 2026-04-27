/// Unit tests for state module — discriminators, seeds, constants, helpers.
///
/// These are pure Rust tests — no SVM, no accounts.
use project_incognitus::state::{
    arena_allocator, asset_balance_ledger, asset_order_status, market_config, read_discriminator,
    slab_allocator, write_discriminator, ALLOCATE_INCREMENT, CLOSING_DISCRIMINATOR,
    INITIAL_ACCOUNT_SIZE, MAX_ACCOUNT_SIZE,
};

// Discriminator helpers
#[test]
fn write_then_read_discriminator_roundtrip() {
    let mut buf = [0u8; 8];
    let disc: u32 = 0xDEAD_BEEF;

    write_discriminator(&mut buf, disc);
    let got = read_discriminator(&buf);

    assert_eq!(got, disc);
}

#[test]
fn discriminator_is_little_endian() {
    let mut buf = [0u8; 4];
    write_discriminator(&mut buf, 0x0102_0304);

    assert_eq!(buf, [0x04, 0x03, 0x02, 0x01]);
}

#[test]
fn read_discriminator_from_zeroed_buffer() {
    let buf = [0u8; 4];
    assert_eq!(read_discriminator(&buf), 0);
}

#[test]
fn closing_discriminator_is_all_ones() {
    assert_eq!(CLOSING_DISCRIMINATOR, 0xFFFF_FFFF);

    let mut buf = [0u8; 4];
    write_discriminator(&mut buf, CLOSING_DISCRIMINATOR);
    assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
}

// All discriminators are unique
#[test]
fn discriminators_are_unique() {
    let discs = [
        market_config::DISCRIMINATOR,
        asset_order_status::DISCRIMINATOR,
        asset_balance_ledger::DISCRIMINATOR,
        arena_allocator::DISCRIMINATOR,
        slab_allocator::DISCRIMINATOR,
        CLOSING_DISCRIMINATOR,
    ];

    for i in 0..discs.len() {
        for j in (i + 1)..discs.len() {
            assert_ne!(
                discs[i], discs[j],
                "discriminators at index {} and {} collide: {:#010X}",
                i, j, discs[i]
            );
        }
    }
}

#[test]
fn no_discriminator_is_zero() {
    // Zero discriminator = uninitialized. No account type should use it.
    assert_ne!(market_config::DISCRIMINATOR, 0);
    assert_ne!(asset_order_status::DISCRIMINATOR, 0);
    assert_ne!(asset_balance_ledger::DISCRIMINATOR, 0);
    assert_ne!(arena_allocator::DISCRIMINATOR, 0);
    assert_ne!(slab_allocator::DISCRIMINATOR, 0);
}

// Seed prefixes are non-empty and unique
#[test]
fn seed_prefixes_are_nonempty() {
    assert!(!market_config::SEED_PREFIX.is_empty());
    assert!(!asset_order_status::SEED_PREFIX.is_empty());
    assert!(!asset_balance_ledger::SEED_PREFIX.is_empty());
    assert!(!arena_allocator::SEED_PREFIX.is_empty());
    assert!(!slab_allocator::SEED_PREFIX.is_empty());
}

#[test]
fn seed_prefixes_are_unique() {
    let seeds: [&[u8]; 5] = [
        market_config::SEED_PREFIX,
        asset_order_status::SEED_PREFIX,
        asset_balance_ledger::SEED_PREFIX,
        arena_allocator::SEED_PREFIX,
        slab_allocator::SEED_PREFIX,
    ];

    for i in 0..seeds.len() {
        for j in (i + 1)..seeds.len() {
            assert_ne!(
                seeds[i], seeds[j],
                "seed prefixes at index {} and {} collide",
                i, j
            );
        }
    }
}

// Account size constants
#[test]
fn initial_account_size_is_10kb() {
    assert_eq!(INITIAL_ACCOUNT_SIZE, 10 * 1024);
}

#[test]
fn allocate_increment_is_10kb() {
    assert_eq!(ALLOCATE_INCREMENT, 10 * 1024);
}

#[test]
fn max_account_size_is_10mb() {
    assert_eq!(MAX_ACCOUNT_SIZE, 10 * 1024 * 1024);
}

#[test]
fn max_divides_evenly_by_increment() {
    assert_eq!(MAX_ACCOUNT_SIZE % ALLOCATE_INCREMENT, 0);
}

#[test]
fn initial_size_divides_evenly_into_max() {
    assert_eq!(MAX_ACCOUNT_SIZE % INITIAL_ACCOUNT_SIZE, 0);
}

// Arena allocator types
#[test]
fn arena_types_are_distinct() {
    let types = [
        arena_allocator::ARENA_TYPE_AGGREGATE,
        arena_allocator::ARENA_TYPE_LIMIT_ORDER_QUEUE,
        arena_allocator::ARENA_TYPE_MARKET_ORDER_QUEUE,
    ];
    for i in 0..types.len() {
        for j in (i + 1)..types.len() {
            assert_ne!(types[i], types[j]);
        }
    }
}

#[test]
fn arena_header_size_is_24() {
    assert_eq!(arena_allocator::HEADER_SIZE, 24);
}

// Slab allocator types
#[test]
fn slab_types_are_distinct() {
    let types = [
        slab_allocator::SLAB_TYPE_PRICE_TREE,
        slab_allocator::SLAB_TYPE_PRICE_STATUS_NODE,
        slab_allocator::SLAB_TYPE_ORDER_BLOCK_NODE,
        slab_allocator::SLAB_TYPE_ORDER_BLOCK_LINKED_NODE,
        slab_allocator::SLAB_TYPE_LEDGER_STATUS,
        slab_allocator::SLAB_TYPE_ORDER_ENTRY_STATUS,
        slab_allocator::SLAB_TYPE_ENTRY_STATUS,
        slab_allocator::SLAB_TYPE_ORDER_ENTRY_STATUS_LINKED,
        slab_allocator::SLAB_TYPE_ENTRY_STATUS_LINKED,
    ];
    for i in 0..types.len() {
        for j in (i + 1)..types.len() {
            assert_ne!(
                types[i], types[j],
                "slab types at index {} and {} collide: {:#04X}",
                i, j, types[i]
            );
        }
    }
}

#[test]
fn slab_header_size_is_24() {
    assert_eq!(slab_allocator::HEADER_SIZE, 24);
}

#[test]
fn slab_stack_node_size_is_16() {
    assert_eq!(slab_allocator::STACK_NODE_SIZE, 16);
}

#[test]
fn slab_header_plus_stack_node_fits_in_initial_size() {
    let overhead = slab_allocator::HEADER_SIZE + slab_allocator::STACK_NODE_SIZE;
    assert!(overhead < INITIAL_ACCOUNT_SIZE as usize);
}

// MarketConfig layout
#[test]
fn market_config_min_data_len_accommodates_all_fields() {
    // 4 (disc) + 32*4 (mints+pools) + 32 (order_book_ledger) + 32*5 (arena+slab ptrs)
    // = 4 + 128 + 32 + 160 = 324
    assert_eq!(market_config::MIN_DATA_LEN, 324);
}

// AssetBalanceLedger layout
#[test]
fn asset_balance_ledger_entry_size() {
    // 32 (mint) + 8 (balance) + 8 (credit) + 8 (debit) = 56
    assert_eq!(asset_balance_ledger::ASSET_ENTRY_SIZE, 56);
}

#[test]
fn asset_balance_ledger_min_is_header_only() {
    // 4 (disc) + 32 (owner) = 36
    assert_eq!(asset_balance_ledger::MIN_DATA_LEN, 36);
}
