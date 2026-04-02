#![allow(dead_code)]
use bytemuck::{Pod, Zeroable};

pub trait SlabNode {
    /// Fixed byte size of this node type.
    const SIZE: usize;

    /// Account type discriminator — uniquely identifies this node type.
    /// Written into the header on initialize, checked on every access.
    const DISCRIMINATOR: u32;

    /// Borrow a node at raw byte address `addr` within the account data.
    fn from_bytes(data: &[u8], addr: u32) -> &Self;

    /// Mutably borrow a node at raw byte address `addr` within the account data.
    fn from_bytes_mut(data: &mut [u8], addr: u32) -> &mut Self;
}

pub const INITIAL_ACCOUNT_SIZE: usize = 10_240;
const THRESHOLD_NUMERATOR: u32 = 20;
const THRESHOLD_DENOMINATOR: u32 = 100;

pub struct SlabAllocator(*mut u8);

impl SlabAllocator {
    /* ── construction ────────────────────────────────────────────────────────── */

    #[allow(clippy::missing_safety_doc)]
    #[inline(always)]
    pub unsafe fn new(ptr: *mut u8) -> Self {
        Self(ptr)
    }

    #[inline(always)]
    pub fn from_account_data(data: &mut [u8]) -> Self {
        Self(data.as_mut_ptr())
    }

    /* ── raw slice access ────────────────────────────────────────────────────── */

    #[inline(always)]
    fn data(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.0, self.alloc_size()) }
    }

    #[inline(always)]
    fn data_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.0, self.alloc_size()) }
    }

    #[inline(always)]
    fn alloc_size(&self) -> usize {
        let raw = unsafe { core::slice::from_raw_parts(self.0, 24) };
        Header::from_bytes(raw.try_into().unwrap()).allocator_size() as usize
    }

    // ── typed region accessors ────────────────────────────────────────────────

    #[inline(always)]
    fn header(&self) -> &Header {
        Header::from_bytes(self.data()[0..24].try_into().unwrap())
    }

    #[inline(always)]
    fn header_mut(&mut self) -> &mut Header {
        Header::from_bytes_mut((&mut self.data_mut()[0..24]).try_into().unwrap())
    }

    #[inline(always)]
    fn head_node(&self) -> &StackNode {
        let alloc_size = self.alloc_size();
        let addr = alloc_size - StackNode::SIZE;
        StackNode::at(self.data(), addr as u32)
    }

    #[inline(always)]
    fn head_node_mut(&mut self) -> &mut StackNode {
        let alloc_size = self.alloc_size();
        let addr = alloc_size - StackNode::SIZE;
        StackNode::at_mut(self.data_mut(), addr as u32)
    }

    /* ── address-level read / write ──────────────────────────────────────────── */

    #[inline(always)]
    fn read_addr(&self, addr: u32) -> u32 {
        let dst = addr as usize;
        u32::from_le_bytes(self.data()[dst..dst + 4].try_into().unwrap())
    }

    #[inline(always)]
    fn write_addr(&mut self, addr: u32, val: u32) {
        let dst = addr as usize;
        self.data_mut()[dst..dst + 4].copy_from_slice(&val.to_le_bytes());
    }

    /* ── initialize ──────────────────────────────────────────────────────────── */

    pub fn initialize<N: SlabNode>(&mut self) {
        let data = unsafe { core::slice::from_raw_parts_mut(self.0, INITIAL_ACCOUNT_SIZE) };

        let header = Header::from_bytes_mut((&mut data[0..24]).try_into().unwrap());
        header.set_discriminator(N::DISCRIMINATOR);
        header.set_allocator_size(INITIAL_ACCOUNT_SIZE as u32);

        let first_node = Header::node_section_start(N::SIZE) as u32;

        let stack_top = (INITIAL_ACCOUNT_SIZE - StackNode::SIZE) as u32;
        header.set_stack_pointer(stack_top);
        header.set_still_growing();

        let head = StackNode::at_mut(data, stack_top);
        head.set_next_seq_index(first_node);
    }

    /* ── getters ─────────────────────────────────────────────────────────────── */

    /* --- setters ---- */

    pub fn next_insert_pointer(&mut self) -> u32 {
        let addr = self.header().stack_pointer();
        self.read_addr(addr)
    }

    // WIP <<<<-----
    // decrease stack entry
    // increase stack address
    // update stack
    pub fn set_next_stack_pointer<N: SlabNode>(&mut self) {
        let addr = self.header().stack_pointer();
        let head_node_addr = (self.alloc_size() - StackNode::SIZE) as u32;

        if addr == head_node_addr {
            let next_seq = self.head_node().next_seq_index();
            self.head_node_mut()
                .set_next_seq_index(next_seq + N::SIZE as u32);
            return;
        }

        let addr = addr + 4;

        if !self.header().is_fragmented_stack() {
            self.header_mut().set_stack_pointer(addr);
            return;
        }

        let next_dest = self.read_addr(addr);

        if next_dest != FULL_FLAG {
            self.header_mut().set_stack_pointer(addr);
            return;
        }

        let node_addr = addr - StackNode::SIZE as u32;
        let next_node_ptr = StackNode::at(self.data(), node_addr).next_stack_node_pointer();
        let next_type = StackNode::at(self.data(), next_node_ptr).stack_node_type_flag();

        if next_type != TYPE_HEAD {
            let linked_insert_sp = StackNode::at(self.data(), next_node_ptr).read_u32(8);

            StackNode::at_mut(self.data_mut(), next_node_ptr).write_u32(8, TYPE_TAIL);

            self.header_mut().set_stack_pointer(linked_insert_sp);

            return;
        }

        // Next node is HeadNode — stack collapses back to single-node state.
        // Recover the insert-side stack_pointer that was saved in head-node
        // at offset 12 when fragmentation began.
        let head_insert_sp = StackNode::at(self.data(), next_node_ptr).stack_pointer_or_flag();

        self.head_node_mut().write_u32(4, head_insert_sp);
        self.head_node_mut().write_u32(12, SINGLE_STACK_NODE);

        self.header_mut().set_stack_pointer(head_insert_sp);
        self.header_mut().clear_fragmented_stack();
    }

    pub fn delete() {}

    pub fn increment_stack() {}
}

pub type NodePointer = u32;

pub const NULL_POINTER: u32 = 0x0000_0000;
pub const FULL_FLAG: u32 = 0xFFFF_FFFF;
pub const FLAG_RESIZE_INTERRUPT: u32 = 0xFF00_0000;
pub const FLAG_FRAGMENTED_STACK: u32 = 0x00FF_0000;
pub const FLAG_VALIDATION: u32 = 0x0000_FF00;
pub const FLAG_STILL_GROWING: u32 = 0x0000_00FF;
pub const TYPE_HEAD: u32 = 0x0000_0000;
pub const TYPE_TAIL: u32 = 0xFFFF_FFFF;
pub const SINGLE_STACK_NODE: u32 = 0x0000_0000;

/*
 * Header — 24 bytes, always at offset 0
 *
 *   0 -  3 : discriminator       — account type tag, derived from SlabNode::DISCRIMINATOR
 *   4 -  7 : root_node_pointer   — 0x0000_0000 if node type has no root
 *   8 - 11 : stack_pointer       — raw byte addr of top-of-stack
 *                                  used for deletions when FLAG_FRAGMENTED_STACK is set
 *  12 - 15 : allocator_size      — total account byte size
 *  16 - 19 : num_node_elements   — count of live nodes
 *  20 - 23 : flags               — see flag constants above
 */

// ── flags field byte masks ────────────────────────────────────────────────────
//
//  byte 3  0xFF00_0000  resize interrupt
//              set:     adjust stack first, then clear, then proceed with
//                       original instruction
//
//  byte 2  0x00FF_0000  fragmented stack state
//              set:     min-stack-tracker suspended
//                       header.stack_pointer used for deletions
//                       head_node.stack_pointer used for inserts
//              clear:   single-pointer normal state
//
//  byte 1  0x0000_FF00  validation flag (concept only, not yet implemented)
//              set:     all validation passed, original instruction may proceed
//
//  byte 0  0x0000_00FF  max allocated / still growing state
//              set:     account is still growing, stack-node structure in use
//              clear:   account reached max allocation, default-stack in use
//              note:    once max allocation is reached this flag is cleared
//                       and is never set again

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct Header([u8; 24]);

impl Header {
    pub const SIZE: usize = 24;

    pub fn from_bytes(bytes: &[u8; 24]) -> &Self {
        bytemuck::from_bytes(bytes)
    }

    pub fn from_bytes_mut(bytes: &mut [u8; 24]) -> &mut Self {
        bytemuck::from_bytes_mut(bytes)
    }

    // ── getters ───────────────────────────────────────────────────────────────

    pub fn discriminator(&self) -> u32 {
        u32::from_le_bytes(self.0[0..4].try_into().unwrap())
    }

    pub fn root_node_pointer(&self) -> NodePointer {
        u32::from_le_bytes(self.0[4..8].try_into().unwrap())
    }

    pub fn stack_pointer(&self) -> u32 {
        u32::from_le_bytes(self.0[8..12].try_into().unwrap())
    }

    pub fn allocator_size(&self) -> u32 {
        u32::from_le_bytes(self.0[12..16].try_into().unwrap())
    }

    pub fn num_node_elements(&self) -> u32 {
        u32::from_le_bytes(self.0[16..20].try_into().unwrap())
    }

    pub fn flags(&self) -> u32 {
        u32::from_le_bytes(self.0[20..24].try_into().unwrap())
    }

    // ── setters ───────────────────────────────────────────────────────────────

    pub fn set_discriminator(&mut self, val: u32) {
        self.0[0..4].copy_from_slice(&val.to_le_bytes());
    }

    pub fn set_root_node_pointer(&mut self, val: NodePointer) {
        self.0[4..8].copy_from_slice(&val.to_le_bytes());
    }

    pub fn set_stack_pointer(&mut self, val: u32) {
        self.0[8..12].copy_from_slice(&val.to_le_bytes());
    }

    pub fn set_allocator_size(&mut self, val: u32) {
        self.0[12..16].copy_from_slice(&val.to_le_bytes());
    }

    pub fn set_num_node_elements(&mut self, val: u32) {
        self.0[16..20].copy_from_slice(&val.to_le_bytes());
    }

    pub fn set_flags(&mut self, val: u32) {
        self.0[20..24].copy_from_slice(&val.to_le_bytes());
    }

    // ── flag helpers ──────────────────────────────────────────────────────────

    pub fn is_resize_interrupt(&self) -> bool {
        self.flags() & FLAG_RESIZE_INTERRUPT != 0
    }

    pub fn is_fragmented_stack(&self) -> bool {
        self.flags() & FLAG_FRAGMENTED_STACK != 0
    }

    pub fn is_still_growing(&self) -> bool {
        self.flags() & FLAG_STILL_GROWING != 0
    }

    pub fn set_resize_interrupt(&mut self) {
        let f = self.flags();
        self.set_flags(f | FLAG_RESIZE_INTERRUPT);
    }

    pub fn clear_resize_interrupt(&mut self) {
        let f = self.flags();
        self.set_flags(f & !FLAG_RESIZE_INTERRUPT);
    }

    pub fn set_fragmented_stack(&mut self) {
        let f = self.flags();
        self.set_flags(f | FLAG_FRAGMENTED_STACK);
    }

    pub fn clear_fragmented_stack(&mut self) {
        let f = self.flags();
        self.set_flags(f & !FLAG_FRAGMENTED_STACK);
    }

    pub fn set_still_growing(&mut self) {
        let f = self.flags();
        self.set_flags(f | FLAG_STILL_GROWING);
    }

    pub fn clear_still_growing(&mut self) {
        let f = self.flags();
        self.set_flags(f & !FLAG_STILL_GROWING);
    }

    // ── derived ───────────────────────────────────────────────────────────────

    /// node_section_start: first multiple of node_size that is >= Header::SIZE.
    /// There may be a gap of zeroed bytes between the header and the first node
    /// if node_size does not divide evenly into Header::SIZE.
    pub fn node_section_start(node_size: usize) -> usize {
        let rem = Self::SIZE % node_size;
        if rem == 0 {
            Self::SIZE
        } else {
            Self::SIZE + (node_size - rem)
        }
    }
}

/* ─────────────────────────────────────────────────────────────────────────────
// StackNode — 16 bytes
//
// A single struct used for all four variants.  Which fields are active depends
// on the stack_node_type_flag at offset 8 and the header flags field.
//
// Variant layout summary:
//
// -----------------------------------------------------------------------------
//  TailNode   (header FLAG_FRAGMENTED_STACK set, type_flag = 0xFFFF_FFFF)
//   offset  0 : top_boundary_flag        — 0xFFFF_FFFF
//
//   offset  4 : next_stack_node_pointer  — raw byte addr of next stack-node
//
//   offset  8 : stack_node_type_flag     — 0xFFFF_FFFF
//
//   offset 12 : bottom_boundary_flag     — 0xFFFF_FFFF
//
//
// -----------------------------------------------------------------------------
//  LinkedNode (header FLAG_FRAGMENTED_STACK set, type_flag != 0xFFFF_FFFF
//              AND != 0x0000_0000, offset 12 != 0x0000_0000)
//   offset  0 : top_boundary_flag        — 0xFFFF_FFFF
//
//   offset  4 : next_stack_node_pointer  — raw byte addr of next stack-node
//
//   offset  8 : tail_stack_entry_pointer — raw byte addr of last stack entry
//                                          in this node's stack section
//
//   offset 12 : bottom_boundary_flag     — 0xFFFF_FFFF
//
//
// -----------------------------------------------------------------------------
//  HeadNode   (type_flag = 0x0000_0000)
//   offset  0 : next_seq_index           — raw byte addr of next sequential
//                                          write slot OR a stack-entry
//
//   offset  4 : stack_entry_cycle_pointer— raw byte addr of a slot in the
//                                          head-node stack section
//                                          inactive when FLAG_FRAGMENTED_STACK set
//
//   offset  8 : stack_node_type_flag     — 0x0000_0000
//
//   offset 12 : stack_pointer            — raw byte addr, used on inserts
//                                          when FLAG_STILL_GROWING set
//
//             : single_stack_node_flag   — 0x0000_0000 = only stack-node
//                                          when FLAG_STILL_GROWING clear
//
//
// -----------------------------------------------------------------------------
//  DefaultNode (FLAG_STILL_GROWING clear, FLAG_FRAGMENTED_STACK clear)
//   offset  0 : next_seq_index           — raw byte addr of next sequential
//                                          write slot OR a stack-entry
//
//   offset  4 : min_stack_node_pointer   — raw byte addr into stack section
//
//   offset  8 : (unused — zeroed)
//   offset 12 : (unused — zeroed)
//
// ─────────────────────────────────────────────────────────────────────────────
*/

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct StackNode([u8; 16]);

impl StackNode {
    pub const SIZE: usize = 16;

    // ── constructors ──────────────────────────────────────────────────────────

    pub fn from_bytes(bytes: &[u8; 16]) -> &Self {
        bytemuck::from_bytes(bytes)
    }

    pub fn from_bytes_mut(bytes: &mut [u8; 16]) -> &mut Self {
        bytemuck::from_bytes_mut(bytes)
    }

    /// Borrow the StackNode at raw byte address `addr` within the account data.
    pub fn at(data: &[u8], addr: u32) -> &Self {
        let a = addr as usize;
        bytemuck::from_bytes(&data[a..a + Self::SIZE])
    }

    /// Mutably borrow the StackNode at raw byte address `addr`.
    pub fn at_mut(data: &mut [u8], addr: u32) -> &mut Self {
        let a = addr as usize;
        bytemuck::from_bytes_mut(&mut data[a..a + Self::SIZE])
    }

    // ── raw field access ──────────────────────────────────────────────────────

    fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(self.0[offset..offset + 4].try_into().unwrap())
    }

    fn write_u32(&mut self, offset: usize, val: u32) {
        self.0[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }

    // ── type identification ───────────────────────────────────────────────────

    /// Read the stack_node_type_flag at offset 8.
    pub fn stack_node_type_flag(&self) -> u32 {
        self.read_u32(8)
    }

    pub fn is_head_node(&self) -> bool {
        self.stack_node_type_flag() == TYPE_HEAD
    }

    pub fn is_tail_node(&self) -> bool {
        self.stack_node_type_flag() == TYPE_TAIL && self.read_u32(12) == FULL_FLAG
    }

    pub fn is_linked_node(&self) -> bool {
        self.stack_node_type_flag() == TYPE_TAIL && self.read_u32(12) != FULL_FLAG
    }

    // ── TailNode fields ───────────────────────────────────────────────────────

    /// offset 0 — boundary sentinel (0xFFFF_FFFF).
    pub fn top_boundary_flag(&self) -> u32 {
        self.read_u32(0)
    }

    /// offset 4 — raw byte addr of the next stack-node in the chain.
    /// Valid for TailNode and LinkedNode.
    pub fn next_stack_node_pointer(&self) -> NodePointer {
        self.read_u32(4)
    }

    pub fn set_next_stack_node_pointer(&mut self, val: NodePointer) {
        self.write_u32(4, val);
    }

    /// offset 8 — set to 0xFFFF_FFFF for TailNode and LinkedNode.
    pub fn set_stack_node_type_flag(&mut self, val: u32) {
        self.write_u32(8, val);
    }

    /// offset 12 — boundary sentinel for TailNode (0xFFFF_FFFF).
    pub fn bottom_boundary_flag(&self) -> u32 {
        self.read_u32(12)
    }

    // ── LinkedNode fields ─────────────────────────────────────────────────────

    /// offset 8 — raw byte addr of the last stack entry in this node's section.
    /// Valid only for LinkedNode (type_flag = 0xFFFF_FFFF, offset 12 != 0xFFFF_FFFF).
    pub fn tail_stack_entry_pointer(&self) -> NodePointer {
        self.read_u32(8)
    }

    pub fn set_tail_stack_entry_pointer(&mut self, val: NodePointer) {
        self.write_u32(8, val);
    }

    // ── HeadNode fields ───────────────────────────────────────────────────────

    /// offset 0 — raw byte addr of next sequential write slot, or a stack-entry.
    /// Valid for HeadNode and DefaultNode.
    pub fn next_seq_index(&self) -> NodePointer {
        self.read_u32(0)
    }

    pub fn set_next_seq_index(&mut self, val: NodePointer) {
        self.write_u32(0, val);
    }

    /// offset 4 — cycles through stack entries in the head-node section,
    /// maintaining the minimum free-node address over time.
    ///
    /// Inactive when FLAG_FRAGMENTED_STACK is set in header.flags.
    /// Active when stack_node_type_flag = 0x0000_0000.
    ///
    /// Two behaviors when active:
    ///   1. tail_entry > entry at this pointer → swap, advance pointer by 4.
    ///   2. next_seq_index - 4 == entry at this pointer → move that entry to
    ///      next_seq_index slot, move tail_entry to that slot, advance pointer.
    pub fn stack_entry_cycle_pointer(&self) -> NodePointer {
        self.read_u32(4)
    }

    pub fn set_stack_entry_cycle_pointer(&mut self, val: NodePointer) {
        self.write_u32(4, val);
    }

    /// offset 12 — dual role depending on FLAG_STILL_GROWING in header.flags:
    ///   FLAG_STILL_GROWING set   → stack_pointer: raw byte addr, used on inserts.
    ///   FLAG_STILL_GROWING clear → single_stack_node_flag: 0x0000_0000 means
    ///                              this is the only stack-node.
    pub fn stack_pointer_or_flag(&self) -> u32 {
        self.read_u32(12)
    }

    pub fn set_stack_pointer_or_flag(&mut self, val: u32) {
        self.write_u32(12, val);
    }

    // ── DefaultNode fields ────────────────────────────────────────────────────

    /// offset 4 — raw byte addr into the stack section, used to track the
    /// minimum free-node address.
    /// Valid only for DefaultNode (FLAG_STILL_GROWING clear, FLAG_FRAGMENTED_STACK clear).
    pub fn min_stack_node_pointer(&self) -> NodePointer {
        self.read_u32(4)
    }

    pub fn set_min_stack_node_pointer(&mut self, val: NodePointer) {
        self.write_u32(4, val);
    }

    // ── transition helpers ────────────────────────────────────────────────────

    /// Transition this node into a TailNode.
    /// Sets type flag and both boundary sentinels.
    /// next_stack_node_pointer must be set separately by the caller.
    pub fn become_tail_node(&mut self) {
        self.write_u32(0, FULL_FLAG);
        self.write_u32(8, TYPE_TAIL);
        self.write_u32(12, FULL_FLAG);
    }

    /// Transition this node into a LinkedNode.
    /// Sets type flag and top boundary sentinel.
    /// next_stack_node_pointer and tail_stack_entry_pointer set separately.
    pub fn become_linked_node(&mut self) {
        self.write_u32(0, FULL_FLAG);
        self.write_u32(12, FULL_FLAG);
    }

    /// Clear all TailNode / LinkedNode flags when a tail-node is consumed
    /// (stack shrinks back into the head-node during inserts).
    pub fn clear_node_flags(&mut self) {
        self.write_u32(0, NULL_POINTER);
        self.write_u32(8, NULL_POINTER);
        self.write_u32(12, NULL_POINTER);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut buf = [0u8; 10];
        let ptr = buf.as_mut_ptr();
        let mut alloc = unsafe { SlabAllocator::new(ptr) };

        alloc.data_mut()[0] = 42;
        assert_eq!(alloc.data()[0], 42);
    }

    #[test]
    fn test_data_and_data_mut() {
        let mut buf = [0u8; 10];
        let mut alloc = SlabAllocator::from_account_data(&mut buf);

        // test data_mut write
        alloc.data_mut()[0] = 42;
        alloc.data_mut()[9] = 99;

        // test data read
        assert_eq!(alloc.data()[0], 42);
        assert_eq!(alloc.data()[9], 99);
    }

    #[test]
    fn test_data_mut_all_bytes() {
        let mut buf = [0u8; 10];
        let mut alloc = SlabAllocator::from_account_data(&mut buf);

        for (i, byte) in alloc.data_mut().iter_mut().enumerate() {
            *byte = i as u8;
        }

        for i in 0..10 {
            assert_eq!(alloc.data()[i], i as u8);
        }
    }
}
