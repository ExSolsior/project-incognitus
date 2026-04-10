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
        insert_ptr: TrieNodePointer,
        slab: &mut SlabAllocator<TrieLinkNode>,
    ) {
        // size = self.size()
        //
        //
        // index = if side.is_ask()
        //      index = 0
        //      num = 0
        //      loop
        //          if num == digit
        //              break
        //
        //          mask = 0x0000_00001 << num
        //          if self.flags() & mask != 0
        //              index += 1
        //
        //          num += 1
        //
        // else if side.is_bid()
        //      index = 0
        //      num = 9
        //      loop
        //          if num == digit
        //              break
        //
        //          mask = 0x0000_00001 << num
        //          if self.flags() & mask != 0
        //              index += 1
        //
        //          num -= 1
        //
        // else
        //      !unreachable()
        //
        //
        // if size == 0
        //      self.set_pointer(insert_ptr)
        //      self.set_size(size + 1)
        //      mask = 0x0000_0001 << index
        //      self.set_flag(self.flag() | mask)
        //      self.set_flag_as_trie_node_ptr()
        //      return
        //
        //
        // (new_link_node_ptr) = if size == 1 || size == 4 || size == 7
        //          new_link_node_ptr = slab.next_insert()
        //          slab.remove_from_stack()
        //          slab.swap()
        //          new_link_node_ptr
        //      else
        //          0x0000_0000
        //
        //
        // if self.pointer().is_trie_node_ptr()
        //          swap_ptr = self.pointer().address()
        //          self.set_link_pointer(new_link_node_ptr)
        //          node = slab.get_node(new_link_node_ptr)
        //
        //          if index == 0
        //              node.set_pointer(0, insert_ptr)
        //              node.set_pointer(1, swap_ptr)
        //          if index == 1
        //              node.set_pointer(0, swap_ptr)
        //              node.set_pointer(1, insert_ptr)
        //
        //          self.set_size(size + 1)
        //          mask = 0x0000_0001 << index
        //          self.set_flag(self.flag() | mask)
        //          self.set_flag_as_trie_link_node_ptr()
        //          return
        //
        //
        // if (size == 4 || size == 7) && self.pointer().is_trie_link_node_ptr()
        //          link_node_ptr = self.pointer().address()
        //
        //          loop
        //              node = slab.get_node(link_node_ptr)
        //              ptr = node.get_pointer(3)
        //
        //              if ptr.is_trie_link_node_ptr()
        //                  link_node_ptr = ptr.address()
        //                  continue
        //
        //              link_node_ptr = ptr.address()
        //              node = slab.get_node(link_node_ptr)
        //              swap_ptr = node.get_pointer(3).address()
        //              node.set_pointer(3, new_link_node_ptr)
        //
        //              node = slab.get_node(new_link_node_ptr)
        //              node.set_pointer(0, swap_ptr)
        //
        //              break
        //
        // -- traverse, swap, insert
        //
        //      swap_ptr = 0
        //
        //      local = if ((index == 3 && size == 3) || (index == 6 && size == 6) || (index == 9 && size == 9))
        //          3
        //      else
        //          index % 3
        //
        //      start_node_idx = if index > 6 && size >= 7
        //          2
        //      else if index > 3 && size >= 4
        //          1
        //      else
        //          0
        //
        //      end_node_idx = if size >= 7
        //          2
        //      else if size >= 4
        //          1
        //      else
        //          0
        //
        //      link_node_ptr = self.pointer().address()
        //      idx = 0
        //
        //      // performs swap and insert
        //      loop !(idx <= end_node_idx)
        //          if link_node_ptr == 0x0000_0000
        //              break
        //
        //
        //          node = slab.get_node(link_node_ptr)
        //          if !(idx >= start_node_idx)
        //              link_node_ptr = node.get_pointer(3).address()
        //              continue
        //
        //
        //          distance = if size % 3 == 0 && start_node == end_node
        //              4 - local
        //          else
        //              3 - local + 1
        //
        //
        //          count = 0
        //          loop !(count > distance)
        //              swap_ptr = node.get_pointer(local)
        //              node.set_pointer(local, insert_ptr)
        //              insert_ptr = swap_ptr
        //              local += 1
        //              count += 1
        //
        //
        //          link_node_ptr = node.get_pointer(3).address()
        //          local = 0
        //          idx += 1
        //
        //
        //  self.set_size(size + 1)
        //  mask = 0x0000_0001 << index
        //  self.set_flag(self.flag() | mask)
    }
}
