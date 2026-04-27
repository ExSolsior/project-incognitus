/// Unit tests for instruction dispatch constants and wire format.
///
/// These verify the instruction data layout without invoking the SVM.
use project_incognitus::instructions::{
    ACCOUNT_TYPE_ARENA_ALLOCATOR, ACCOUNT_TYPE_ASSET_BALANCE_LEDGER,
    ACCOUNT_TYPE_ASSET_ORDER_STATUS, ACCOUNT_TYPE_MARKET_CONFIG, ACCOUNT_TYPE_SLAB_ALLOCATOR,
    IX_ALLOCATE, IX_CLOSE, IX_INITIALIZE,
};

// Instruction discriminators
#[test]
fn instruction_discriminators_sequential() {
    assert_eq!(IX_INITIALIZE, 0x00);
    assert_eq!(IX_ALLOCATE, 0x01);
    assert_eq!(IX_CLOSE, 0x02);
}

#[test]
fn instruction_discriminators_unique() {
    let discs = [IX_INITIALIZE, IX_ALLOCATE, IX_CLOSE];
    for i in 0..discs.len() {
        for j in (i + 1)..discs.len() {
            assert_ne!(discs[i], discs[j]);
        }
    }
}

// Account type tags
#[test]
fn account_type_tags_sequential() {
    assert_eq!(ACCOUNT_TYPE_MARKET_CONFIG, 0x00);
    assert_eq!(ACCOUNT_TYPE_ASSET_ORDER_STATUS, 0x01);
    assert_eq!(ACCOUNT_TYPE_ASSET_BALANCE_LEDGER, 0x02);
    assert_eq!(ACCOUNT_TYPE_ARENA_ALLOCATOR, 0x03);
    assert_eq!(ACCOUNT_TYPE_SLAB_ALLOCATOR, 0x04);
}

#[test]
fn account_type_tags_unique() {
    let tags = [
        ACCOUNT_TYPE_MARKET_CONFIG,
        ACCOUNT_TYPE_ASSET_ORDER_STATUS,
        ACCOUNT_TYPE_ASSET_BALANCE_LEDGER,
        ACCOUNT_TYPE_ARENA_ALLOCATOR,
        ACCOUNT_TYPE_SLAB_ALLOCATOR,
    ];
    for i in 0..tags.len() {
        for j in (i + 1)..tags.len() {
            assert_ne!(tags[i], tags[j]);
        }
    }
}

// Wire format construction helpers — verify expected byte layouts
#[test]
fn initialize_market_config_wire_format() {
    // [0]=init, [1]=market_config, [2]=bump
    let bump = 254u8;
    let data = [IX_INITIALIZE, ACCOUNT_TYPE_MARKET_CONFIG, bump];

    assert_eq!(data[0], 0x00);
    assert_eq!(data[1], 0x00);
    assert_eq!(data[2], 254);
    assert_eq!(data.len(), 3); // minimum length for non-arena/slab init
}

#[test]
fn initialize_arena_wire_format() {
    // [0]=init, [1]=arena, [2]=bump, [3]=arena_type
    let bump = 255u8;
    let arena_type = 0x02u8; // LIMIT_ORDER_QUEUE
    let data = [
        IX_INITIALIZE,
        ACCOUNT_TYPE_ARENA_ALLOCATOR,
        bump,
        arena_type,
    ];

    assert_eq!(data.len(), 4); // minimum for arena init
    assert_eq!(data[3], 0x02);
}

#[test]
fn initialize_slab_wire_format() {
    // [0]=init, [1]=slab, [2]=bump, [3]=slab_type, [4]=slab_index
    let bump = 253u8;
    let slab_type = 0x10u8; // PRICE_TREE
    let slab_index = 0u8;
    let data = [
        IX_INITIALIZE,
        ACCOUNT_TYPE_SLAB_ALLOCATOR,
        bump,
        slab_type,
        slab_index,
    ];

    assert_eq!(data.len(), 5); // minimum for slab init
    assert_eq!(data[3], 0x10);
    assert_eq!(data[4], 0);
}

#[test]
fn allocate_wire_format() {
    // [0]=allocate, [1]=account_type
    let data = [IX_ALLOCATE, ACCOUNT_TYPE_ARENA_ALLOCATOR];

    assert_eq!(data.len(), 2); // minimum for allocate
    assert_eq!(data[0], 0x01);
}

#[test]
fn close_wire_format() {
    // [0]=close, [1]=account_type
    let data = [IX_CLOSE, ACCOUNT_TYPE_SLAB_ALLOCATOR];

    assert_eq!(data.len(), 2); // minimum for close
    assert_eq!(data[0], 0x02);
}
