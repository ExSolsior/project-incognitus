use crate::pointers::{Pointer, TrieLinkNodePointer, TrieNodePointer};

// the child is not considering the side, I need fix that
impl TrieLink {
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

        // 1. Find the logical index of the child to remove.
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

        // 2. Case: Removing the last child.
        if size == 1 {
            self.set_pointer(0);
            self.set_size(0);
            self.clear_child_digit(digit);
            // Flags bits 11-12 are cleared as size 0 implies PriceNode.
            // need to validate, it should clear flags
            let flags = self.flags() & !(TRIE_NODE_PTR_FLAG | TRIE_LINK_NODE_PTR_FLAG);
            self.set_flags(flags);

            // no more children, should become a pointer to price node
            return;
        }

        // 3. Shift Pointers Left.
        // We traverse the chain and move every pointer after 'index' one slot to the left.
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
        // Correction for 4th slot variants if we are removing from a full node boundary.
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

            // We need to pull the "next" pointer into the current slot.
            let mut cursor = local;
            loop {
                let next_idx = cursor + 1;
                let next_val = if next_idx <= 3 {
                    let ptr = node.pointer(next_idx);
                    if ptr.is_trie_link_node() {
                        // Crossing node boundary: pull from slot 0 of the next node.
                        let next_node = slab.get_node(ptr.address());
                        next_node.pointer(0).address()
                    } else {
                        ptr.address()
                    }
                } else {
                    // Current node is finished, break to advance to next node.
                    break;
                };

                // If setting slot 3, preserve TrieNode encoding.
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

        // 4. Shrink chain if necessary (size transitions 2->1, 5->4, 8->7).
        match size {
            2 => {
                // Collapse single link node back to a direct TrieNodePointer.
                let link_node_ptr = self.pointer().address();
                let node = slab.get_node(link_node_ptr);
                let remaining_ptr = node.pointer(0).address();

                self.set_pointer(remaining_ptr);
                let flags = (self.flags() & !TRIE_LINK_NODE_PTR_FLAG) | TRIE_NODE_PTR_FLAG;
                self.set_flags(flags);

                // Note: slab allocator cleanup logic (freeing link_node_ptr) would go here.
            }
            5 | 8 => {
                // The last node in the chain now has 0 children (it was size 1).
                // We find the parent node and turn its link into a direct TrieNode variant.
                let mut current_ptr = self.pointer().address();
                loop {
                    let node = slab.get_node(current_ptr);
                    let next = node.next_link_ptr();
                    if next.is_trie_link_node() {
                        let next_node = slab.get_node(next.address());
                        if next_node.next_link_ptr().address() == 0 {
                            // This is the second-to-last node.
                            let empty_node_ptr = next.address();
                            let last_child_ptr = next_node.pointer(0).address();

                            // Collapse: convert slot 3 link to a node pointer.
                            node.set(3, TrieLinkNodePointer::from_node(last_child_ptr).0);
                            // slab.free(empty_node_ptr)
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

        // 1. Calculate the logical index based on price priority.
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

        // 2. Case: Single child (direct pointer).
        if size == 1 {
            return match self.pointer() {
                Pointer::TrieNode(p) => Some(p),
                _ => None,
            };
        }

        // 3. Traverse the chain to the specific index.
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
                // The 4th slot is a data pointer (variant).
                return match node.pointer(3) {
                    Pointer::TrieNode(p) => Some(p),
                    _ => None,
                };
            } else {
                return None;
            }
        }
    }
}
