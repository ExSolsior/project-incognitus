/*
 * TrieNodePointer — u32
 *
 * A raw byte address into the TrieNode slab. No encoding — the type is always
 * known from context. NULL_POINTER (0x0000_0000) indicates no child.
 */

pub struct TrieNodePointer(u32);

impl From<u32> for TrieNodePointer {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl TrieNodePointer {
    pub fn address(&self) -> u32 {
        self.0 & 0x00FF_FFFF
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0x0000_0000
    }
}

/*
 * TrieLinkNodePointer — u32
 *
*  I think this Encoding is not valid
*  0b0000_0001
*  0b0000_0010
 * Encoding:
 *   upper byte (bits 24–31): type flags
 *     0x0100_0000 — variant points to next TrieLinkNode (chain continues)
 *     0x0200_0000 — variant points to a TrieNode (4th child, last in chain)
 *   lower 3 bytes (bits 0–23): raw byte address into TrieLinkNode slab
 *
 * Exactly one of the type flags must be set. Address is extracted by masking
 * off the upper byte. Max account size is 10mb (0x00A0_0000), so bits 24–31
 * are always free for flag use.
 */

const TRIE_LINK_NODE_FLAG: u32 = 0x0100_0000;
const TRIE_NODE_FLAG: u32 = 0x0200_0000;

pub struct TrieLinkNodePointer(u32);

impl From<u32> for TrieLinkNodePointer {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl TrieLinkNodePointer {
    pub fn address(&self) -> u32 {
        self.0 & 0x00FF_FFFF
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0x0000_0000
    }

    // this is valid, keep it
    pub fn is_trie_link_node(&self) -> bool {
        self.0 & TRIE_LINK_NODE_FLAG != 0
    }

    pub fn is_next_trie_link_node(&self) -> bool {
        self.0 & (TRIE_LINK_NODE_FLAG | NEXT_TRIE_LINK_NODE_FLAG) != 0
    }

    // this I don't think is valid, don't keep
    pub fn is_trie_node(&self) -> bool {
        self.0 & TRIE_NODE_FLAG != 0
    }

    pub fn set_link(address: u32) -> Self {
        Self(address | TRIE_LINK_NODE_FLAG)
    }

    pub fn set_next_link(address: u32) -> Self {
        Self(address | TRIE_LINK_NODE_FLAG | NEXT_TRIE_LINK_NODE_FLAG)
    }
}

/*
 * PriceNodePointer — u32
 *
 * A raw byte address into the PriceNode slab. No encoding — the type is always
 * known from context. NULL_POINTER (0x0000_0000) indicates no price node.
 */

pub struct PriceNodePointer(u32);

impl From<u32> for PriceNodePointer {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl PriceNodePointer {
    pub fn address(&self) -> u32 {
        self.0 && 0x00FF_FFFF
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0x0000_0000
    }
}

/*
 * TrieLinkPointer — enum
 *
 * Resolved pointer type returned by TrieLink::pointer(). Exactly one variant
 * is active, determined by bits 11–13 of TrieLink::flags().
 */

pub enum Pointer {
    TrieNode(TrieNodePointer),
    TrieLinkNode(TrieLinkNodePointer),
    PriceNode(PriceNodePointer),
}

impl Pointer {
    pub fn address(&self) -> u32 {
        match self {
            Pointer::TrieNode(p) => p.address(),
            Pointer::TrieLinkNode(p) => p.address(),
            Pointer::PriceNode(p) => p.address(),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Pointer::TrieNode(p) => p.is_null(),
            Pointer::TrieLinkNode(p) => p.is_null(),
            Pointer::PriceNode(p) => p.is_null(),
        }
    }

    pub fn is_trie_node(&self) -> bool {
        matches!(self, Pointer::TrieNode(_))
    }

    // I'm not sure if this is applicable, since there could be a situation
    // that it's not a trie link node, so maybe trie link node?
    pub fn is_trie_link_node(&self) -> bool {
        matches!(self, Pointer::TrieLinkNode(_))
    }

    pub fn is_price_node(&self) -> bool {
        matches!(self, Pointer::PriceNode(_))
    }
}
