use crate::pointers::{Pointer, TrieLinkNodePointer, TrieNodePointer};

/*
 * TrieLinkNode — 16 bytes
 *
 * Support node for TrieNode when it has more than one child. Chains of
 * TrieLinkNodes store up to 10 child TrieNode addresses (3 + 3 + 4 across
 * a chain of up to 3 nodes).
 *
 * Layout:
 *   offset  0: child[0]  u32  — first child TrieNode raw byte address
 *   offset  4: child[1]  u32  — second child TrieNode raw byte address
 *   offset  8: child[2]  u32  — third child TrieNode raw byte address
 *   offset 12: variant   u32  — next TrieLinkNode OR 4th child TrieNode
 *                               type determined by TrieLinkNodePointer encoding
 *                               of the pointer that led to this node
 *
 * pointer() returns:
 *   index 0–2 → Pointer::TrieNode   (always a child TrieNode)
 *   index   3 → Pointer::TrieLinkNode | Pointer::TrieNode  (resolved via TrieLinkNodePointer encoding)
 */

pub struct TrieLinkNode([u8; 16]);

impl TrieLinkNode {
    // I think this is bugged
    pub fn pointer(&self, index: usize) -> Pointer {
        let offset = index * 4;
        let raw = u32::from_le_bytes(self.0[offset..offset + 4].try_into().unwrap());
        let link_ptr = TrieLinkNodePointer::from(raw);
        match (index, link_ptr.is_trie_link_node()) {
            (0..=2, _) => Pointer::TrieNode(TrieNodePointer::from(raw)),
            (_, true) => Pointer::TrieLinkNode(link_ptr),
            (_, false) => Pointer::TrieNode(TrieNodePointer::from(link_ptr.address())),
        }
    }

    pub fn next_link_ptr(&self) -> TrieLinkNodePointer {
        let raw_ptr = u32::from_le_bytes(self.0[12..16].try_into().unwrap());
        TrieLinkNodePointer::from(raw_ptr)
    }

    pub fn set(&mut self, index: usize, val: u32) {
        let offset = index * 4;
        self.0[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }
}

pub fn insert() {}
