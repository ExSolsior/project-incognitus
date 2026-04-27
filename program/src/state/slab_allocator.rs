/// SlabAllocator — raw byte buffer memory manager for fixed-size nodes.
///
/// Two slab collection types exist per market:
///   - OrderBookCollection (contains: PriceTree, PriceStatusNode,
///     OrderBlockNode, OrderBlockLinkedNode)
///   - LedgerStatusCollection (contains: LedgerStatus, OrderEntryStatus,
///     EntryStatus, OrderEntryStatusLinkedNode, EntryStatusLinkedNode)
///
/// Seeds: ["slab-allocator", market_config, slab_type_tag, slab_index]
///
/// Each slab stores one node type. Multiple slabs of the same type
/// can exist (indexed by slab_index) — this is what makes the order
/// book boundless.
///
/// Initialized at 10 KB, grows in 10 KB increments up to 10 MB.
/// See architecture doc Section 17 for full design.

/// Discriminator for SlabAllocator accounts.
pub const DISCRIMINATOR: u32 = 0x534C_4142; // "SLAB"

/// PDA seed prefix.
pub const SEED_PREFIX: &[u8] = b"slab-allocator";

// -- OrderBookCollection slab type tags --

/// PriceTree nodes (TrieNode, TrieLink, TrieLinkNode).
pub const SLAB_TYPE_PRICE_TREE: u8 = 0x10;

/// PriceStatusNode (PriceNode).
pub const SLAB_TYPE_PRICE_STATUS_NODE: u8 = 0x11;

/// OrderBlockNode (OrderBlock — 408 bytes per node).
pub const SLAB_TYPE_ORDER_BLOCK_NODE: u8 = 0x12;

/// OrderBlockLinkedNode (traversal structure for OrderBlocks).
pub const SLAB_TYPE_ORDER_BLOCK_LINKED_NODE: u8 = 0x13;

// -- LedgerStatusCollection slab type tags --

/// LedgerStatus (AddressIndexNode).
pub const SLAB_TYPE_LEDGER_STATUS: u8 = 0x20;

/// OrderEntryStatus (OrderEntryStatusNode).
pub const SLAB_TYPE_ORDER_ENTRY_STATUS: u8 = 0x21;

/// EntryStatus (EntryChangeLogNode).
pub const SLAB_TYPE_ENTRY_STATUS: u8 = 0x22;

/// OrderEntryStatusLinkedNode.
pub const SLAB_TYPE_ORDER_ENTRY_STATUS_LINKED: u8 = 0x23;

/// EntryStatusLinkedNode.
pub const SLAB_TYPE_ENTRY_STATUS_LINKED: u8 = 0x24;

/// Slab header size — matches architecture doc Section 17.
/// Layout:
///   [0..4]   discriminator: u32
///   [4..8]   root_node_pointer: u32
///   [8..12]  stack_pointer: u32
///   [12..16] allocator_size: u32
///   [16..20] num_node_elements: u32
///   [20..24] flags: u32
pub const HEADER_SIZE: usize = 24;

/// StackNode size — 16 bytes (HeadNode, TailNode, LinkedNode variants).
pub const STACK_NODE_SIZE: usize = 16;
