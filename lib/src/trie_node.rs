use crate::pointers::{Pointer, PriceNodePointer, TrieLinkNodePointer, TrieNodePointer};

/*
 * TrieNode — 24 bytes
 *
 * Layout:
 *   offset  0: quantity_delta  u64  — sum of all open order quantities in subtree
 *   offset  8: quote_delta     u64  — sum of all open order quotes in subtree
 *   offset 16: (digit, depth)  u8   — low nibble: decimal digit (0–9)
 *                                   — high nibble: depth in trie (0 = ones place, root = highest depth)
 *   offset 17: trie_link       TrieLink (7 bytes, see below)
 */

pub struct TrieNode([u8; 24]);

impl TrieNode {
    pub fn quantity_delta(&self) -> u64 {
        u64::from_le_bytes(self.0[0..8].try_into().unwrap())
    }

    pub fn set_quantity_delta(&mut self, val: u64) {
        self.0[0..8].copy_from_slice(&val.to_le_bytes());
    }

    pub fn quote_delta(&self) -> u64 {
        u64::from_le_bytes(self.0[8..16].try_into().unwrap())
    }

    pub fn set_quote_delta(&mut self, val: u64) {
        self.0[8..16].copy_from_slice(&val.to_le_bytes());
    }

    pub fn digit(&self) -> u8 {
        self.0[16] & 0b0000_1111
    }

    pub fn depth(&self) -> u8 {
        (self.0[16] & 0b1111_0000) >> 4
    }

    /* writes both digit and depth in a single byte write — caller packs the value */
    pub fn set_value(&mut self, val: u8) {
        self.0[16] = val;
    }

    pub fn set_digit(&mut self, val: u8) {
        self.0[16] = (self.0[16] & 0b1111_0000) | (val & 0b0000_1111);
    }

    pub fn set_depth(&mut self, val: u8) {
        self.0[16] = (self.0[16] & 0b0000_1111) | ((val & 0b0000_1111) << 4);
    }

    pub fn trie_link(&self) -> &TrieLink {
        unsafe { &*(self.0[17..].as_ptr() as *const TrieLink) }
    }

    pub fn trie_link_mut(&mut self) -> &mut TrieLink {
        unsafe { &mut *(self.0[17..].as_mut_ptr() as *mut TrieLink) }
    }

    pub fn update_deltas(&mut self, quantity_delta: i64, quote_delta: i64) {
        let new_q = (self.quantity_delta() as i128 + quantity_delta as i128) as u64;
        let new_v = (self.quote_delta() as i128 + quote_delta as i128) as u64;
        self.set_quantity_delta(new_q);
        self.set_quote_delta(new_v);
    }

    // TODO: Implement Price Calculation helper to reconstruct full price or validate digits at depth.
}

/*
 * TrieLink — 7 bytes, embedded in TrieNode at offset 17
 *
 * Layout:
 *   offset 17: size     u8   — number of direct TrieNode children (0–10)
 *   offset 18: flags    u16  — bitmask (see encoding below)
 *   offset 20: pointer  u32  — raw byte address; target type encoded in flags bits 11–13
 *
 * flags encoding (u16):
 *   bits  0–9:  digit bitmask — bit N set means a child with digit=N exists
 *   bit  10:    reserved
 *   bit  11:    pointer → TrieLinkNode  (multiple children)
 *   bit  12:    pointer → TrieNode      (single child)
 *   bit  13:    pointer → PriceNode
 *   bit  14:    is_active — when unset on a parent, entire subtree is inactive
 *   bit  15:    reserved
 *
 * PriceNode is implied by size == 0 — no flag needed.
 * Exactly one of bits 11, 12 must be set when size > 0. Any other combination is invalid.
 */

const TRIE_LINK_NODE_PTR_FLAG: u16 = 0x0800; /* bit 11 */
const TRIE_NODE_PTR_FLAG: u16 = 0x1000; /* bit 12 */
const PRICE_NODE_PTR_FLAG: u16 = 0x2000;
const IS_ACTIVE_FLAG: u16 = 0x4000; /* bit 14 */

pub struct TrieLink([u8; 7]);

impl TrieLink {
    pub fn size(&self) -> u8 {
        self.0[0]
    }

    pub fn set_size(&mut self, val: u8) {
        self.0[0] = val;
    }

    pub fn flags(&self) -> u16 {
        u16::from_le_bytes(self.0[1..3].try_into().unwrap())
    }

    pub fn set_flags(&mut self, val: u16) {
        self.0[1..3].copy_from_slice(&val.to_le_bytes());
    }

    pub fn pointer(&self) -> Pointer {
        let raw = u32::from_le_bytes(self.0[3..7].try_into().unwrap());
        match self.flags() & (TRIE_LINK_NODE_PTR_FLAG | TRIE_NODE_PTR_FLAG | PRICE_NODE_PTR_FLAG) {
            PRICE_NODE_PTR_FLAG => Pointer::PriceNode(PriceNodePointer::from(raw)),
            TRIE_LINK_NODE_PTR_FLAG => Pointer::TrieLinkNode(TrieLinkNodePointer::from(raw)),
            TRIE_NODE_PTR_FLAG => Pointer::TrieNode(TrieNodePointer::from(raw)),
            _ => unreachable!(),
        }
    }

    pub fn next_link_ptr(&self) -> TrieLinkNodePointer {
        let raw_ptr = u32::from_le_bytes(self.0[20..24].try_into().unwrap());
        TrieLinkNodePointer::from(raw_ptr)
    }

    pub fn set_pointer(&mut self, val: u32) {
        self.0[3..7].copy_from_slice(&val.to_le_bytes());
    }

    pub fn set_as_trie_link_node_ptr(&mut self) {
        let flags =
            self.flags() & !(TRIE_NODE_PTR_FLAG | PRICE_NODE_PTR_FLAG) | TRIE_LINK_NODE_PTR_FLAG;
        self.set_flags(flags)
    }

    pub fn set_as_price_node_ptr(&mut self) {
        let flags =
            self.flags() & !(TRIE_NODE_PTR_FLAG | TRIE_LINK_NODE_PTR_FLAG) | PRICE_NODE_PTR_FLAG;
        self.set_flags(flags)
    }

    pub fn set_as_trie_node_ptr(&mut self) {
        let flags =
            self.flags() & !(TRIE_LINK_NODE_PTR_FLAG | PRICE_NODE_PTR_FLAG) | TRIE_NODE_PTR_FLAG;
        self.set_flags(flags)
    }

    pub fn set_price_node(&mut self, ptr: PriceNodePointer) {
        self.set_size(0);
        self.set_pointer(ptr.address());
        self.set_as_price_node_ptr();
    }

    /* --- digit bitmask (bits 0–9) --- */

    pub fn has_child_digit(&self, digit: u8) -> bool {
        self.flags() & (1 << digit) != 0
    }

    pub fn set_child_digit(&mut self, digit: u8) {
        let flags = self.flags() | (1 << digit);
        self.set_flags(flags);
    }

    pub fn clear_child_digit(&mut self, digit: u8) {
        let flags = self.flags() & !(1 << digit);
        self.set_flags(flags);
    }

    /* --- pointer type (bits 11–12) --- */

    pub fn is_trie_link_node_ptr(&self) -> bool {
        self.flags() & TRIE_LINK_NODE_PTR_FLAG != 0
    }

    pub fn is_trie_node_ptr(&self) -> bool {
        self.flags() & TRIE_NODE_PTR_FLAG != 0
    }

    pub fn is_price_node_prt(&self) -> bool {
        self.flags() & PRICE_NODE_PTR_FLAG != 0
    }

    /* --- activity (bit 14) --- */

    // TODO: Implement is_active propagation logic during traversal.

    pub fn is_active(&self) -> bool {
        self.flags() & IS_ACTIVE_FLAG != 0
    }

    pub fn set_active(&mut self, active: bool) {
        let flags = if active {
            self.flags() | IS_ACTIVE_FLAG
        } else {
            self.flags() & !IS_ACTIVE_FLAG
        };
        self.set_flags(flags);
    }

    pub fn insert(
        &mut self,
        side: Side,
        digit: u8,
        mut insert_ptr: TrieNodePointer,
        slab: &mut SlabAllocator<TrieLinkNode>,
    ) {
        let size = self.size();

        // 1. Calculate the insertion index to maintain price priority.
        let mut index = 0;
        match side {
            Side::Ask => {
                for i in 0..10 {
                    if i == digit {
                        break;
                    }
                    if self.has_child_digit(i) {
                        index += 1;
                    }
                }
            }
            Side::Bid => {
                for i in (0..10).rev() {
                    if i == digit {
                        break;
                    }
                    if self.has_child_digit(i) {
                        index += 1;
                    }
                }
            }
        }

        // 2. Case: Empty TrieLink (size 0).
        if size == 0 {
            self.set_pointer(insert_ptr.address());
            self.set_size(1);
            self.set_child_digit(digit);
            let flags = (self.flags() & !TRIE_LINK_NODE_PTR_FLAG) | TRIE_NODE_PTR_FLAG;
            self.set_flags(flags);
            return;
        }

        // 3. Expansion check (at size 1, 4, 7).
        let new_link_node_ptr = match size {
            1 | 4 | 7 => {
                let ptr = slab.next_insert();
                slab.remove_from_stack();
                slab.swap();
                ptr
            }
            _ => 0,
        };

        // 4. Case: Transition from single pointer to a chain (size 1 -> 2).
        if self.is_trie_node_ptr() {
            let swap_ptr = self.pointer().address();
            self.set_pointer(new_link_node_ptr);
            let node = slab.get_node(new_link_node_ptr);

            match index {
                0 => {
                    node.set(0, insert_ptr.address());
                    node.set(1, swap_ptr);
                }
                _ => {
                    node.set(0, swap_ptr);
                    node.set(1, insert_ptr.address());
                }
            }

            self.set_size(size + 1);
            self.set_child_digit(digit);
            let flags = (self.flags() & !TRIE_NODE_PTR_FLAG) | TRIE_LINK_NODE_PTR_FLAG;
            self.set_flags(flags);
            return;
        }

        // 5. Expand the chain if adding a 5th or 8th child.
        if matches!(size, 4 | 7) && self.is_trie_link_node_ptr() {
            let mut current_ptr = self.pointer().address();
            loop {
                let node = slab.get_node(current_ptr);
                let next = node.next_link_ptr();

                if next.is_trie_link_node() {
                    current_ptr = next.address();
                    continue;
                }

                let swap_ptr = next.address();
                node.set(3, TrieLinkNodePointer::from_link(new_link_node_ptr).0);

                let next_node = slab.get_node(new_link_node_ptr);
                next_node.set(0, swap_ptr);
                break;
            }
        }

        // 6. Traverse, Shift, and Insert.
        let mut node_idx = 0;
        let start_node_idx = match index {
            i if i > 6 && size >= 7 => 2,
            i if i > 3 && size >= 4 => 1,
            _ => 0,
        };

        let end_node_idx = match size {
            s if s >= 7 => 2,
            s if s >= 4 => 1,
            _ => 0,
        };

        let mut local = match (index, size) {
            (3, 3) | (6, 6) | (9, 9) => 3,
            _ => (index % 3) as usize,
        };

        let mut current_link_ptr = self.pointer().address();

        while node_idx <= end_node_idx {
            if current_link_ptr == 0 {
                break;
            }
            let node = slab.get_node(current_link_ptr);

            if node_idx < start_node_idx {
                current_link_ptr = node.next_link_ptr().address();
                node_idx += 1;
                continue;
            }

            let is_last_node = node_idx == end_node_idx;
            let distance = match (size % 3 == 0, is_last_node) {
                (true, true) => 4 - local,
                _ => 3 - local,
            };

            for _ in 0..distance {
                let current_val = node.pointer(local).address();
                let val_to_set = if local == 3 {
                    TrieLinkNodePointer::from_node(insert_ptr.address()).0
                } else {
                    insert_ptr.address()
                };
                node.set(local, val_to_set);
                insert_ptr = TrieNodePointer::from(current_val);
                local += 1;
            }

            current_link_ptr = node.next_link_ptr().address();
            local = 0;
            node_idx += 1;
        }

        self.set_size(size + 1);
        self.set_child_digit(digit);
    }

    pub fn remove(&mut self, side: Side, digit: u8, slab: &mut SlabAllocator<TrieLinkNode>) {
        let size = self.size();
        if size == 0 {
            return;
        }

        let mut index = 0;
        match side {
            Side::Ask => {
                for i in 0..digit {
                    if self.has_child_digit(i) {
                        index += 1;
                    }
                }
            }
            Side::Bid => {
                for i in (digit + 1..10).rev() {
                    if self.has_child_digit(i) {
                        index += 1;
                    }
                }
            }
        }

        if size == 1 {
            self.set_pointer(0);
            self.set_size(0);
            self.clear_child_digit(digit);
            let flags = self.flags() & !(TRIE_NODE_PTR_FLAG | TRIE_LINK_NODE_PTR_FLAG);
            self.set_flags(flags);
            return;
        }

        let mut node_idx = 0;
        let start_node_idx = match index {
            i if i >= 7 => 2,
            i if i >= 4 => 1,
            _ => 0,
        };
        let end_node_idx = match size {
            s if s > 7 => 2,
            s if s > 4 => 1,
            _ => 0,
        };

        let mut current_link_ptr = self.pointer().address();
        let mut local = (index % 3) as usize;
        if index == 3 || index == 6 || index == 9 {
            local = 3;
        }

        while node_idx <= end_node_idx {
            let node = slab.get_node(current_link_ptr);
            if node_idx < start_node_idx {
                current_link_ptr = node.next_link_ptr().address();
                node_idx += 1;
                continue;
            }

            let mut cursor = local;
            loop {
                let next_idx = cursor + 1;
                let next_val = if next_idx <= 3 {
                    let ptr = node.pointer(next_idx);
                    if ptr.is_trie_link_node() {
                        let next_node = slab.get_node(ptr.address());
                        next_node.pointer(0).address()
                    } else {
                        ptr.address()
                    }
                } else {
                    break;
                };

                let val_to_set = if cursor == 3 {
                    TrieLinkNodePointer::from_node(next_val).0
                } else {
                    next_val
                };
                node.set(cursor, val_to_set);
                cursor += 1;
                if cursor > 3 {
                    break;
                }
            }

            current_link_ptr = node.next_link_ptr().address();
            if current_link_ptr == 0 {
                break;
            }
            local = 0;
            node_idx += 1;
        }

        match size {
            2 => {
                let link_node_ptr = self.pointer().address();
                let node = slab.get_node(link_node_ptr);
                let remaining_ptr = node.pointer(0).address();

                self.set_pointer(remaining_ptr);
                let flags = (self.flags() & !TRIE_LINK_NODE_PTR_FLAG) | TRIE_NODE_PTR_FLAG;
                self.set_flags(flags);
            }
            5 | 8 => {
                let mut current_ptr = self.pointer().address();
                loop {
                    let node = slab.get_node(current_ptr);
                    let next = node.next_link_ptr();
                    if next.is_trie_link_node() {
                        let next_node = slab.get_node(next.address());
                        if next_node.next_link_ptr().address() == 0 {
                            let last_child_ptr = next_node.pointer(0).address();
                            node.set(3, TrieLinkNodePointer::from_node(last_child_ptr).0);
                            break;
                        }
                        current_ptr = next.address();
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }

        self.set_size(size - 1);
        self.clear_child_digit(digit);
    }

    pub fn get_child(
        &self,
        side: Side,
        digit: u8,
        slab: &SlabAllocator<TrieLinkNode>,
    ) -> Option<TrieNodePointer> {
        if !self.has_child_digit(digit) {
            return None;
        }

        let size = self.size();
        if size == 0 {
            return None;
        }

        let mut index = 0;
        match side {
            Side::Ask => {
                for i in 0..digit {
                    if self.has_child_digit(i) {
                        index += 1;
                    }
                }
            }
            Side::Bid => {
                for i in (digit + 1..10).rev() {
                    if self.has_child_digit(i) {
                        index += 1;
                    }
                }
            }
        }

        if size == 1 {
            return match self.pointer() {
                Pointer::TrieNode(p) => Some(p),
                _ => None,
            };
        }

        let mut current_link_ptr = self.pointer().address();
        let mut cursor = index;

        loop {
            if current_link_ptr == 0 {
                return None;
            }
            let node = slab.get_node(current_link_ptr);

            if cursor < 3 {
                return match node.pointer(cursor as usize) {
                    Pointer::TrieNode(p) => Some(p),
                    _ => None,
                };
            }

            let next = node.next_link_ptr();
            if next.is_trie_link_node() {
                current_link_ptr = next.address();
                cursor -= 3;
            } else if cursor == 3 {
                return match node.pointer(3) {
                    Pointer::TrieNode(p) => Some(p),
                    _ => None,
                };
            } else {
                return None;
            }
        }
    }

    // TODO: Implement Child Iterator for side-aware traversal (Ask: 0->9, Bid: 9->0).
}
