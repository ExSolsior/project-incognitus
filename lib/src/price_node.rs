/*
 * PriceNode — 32 bytes
 *
 * Maps a specific price level to its collection of OrderBlocks.
 * Tracks insertion and match positions across blocks.
 *
 * Layout:
 *   offset  0: last_slot_update            u64  — slot when this node was last modified
 *   offset  8: price                       u64  — the price value this node represents
 *   offset 16: parent_pointer              u32  — raw byte address of parent TrieNode
 *   offset 20: order_block_pointer         u32  — raw byte address of OrderBlock collection
 *   offset 24: next_entry_offset           u8   — offset within last block for next insert (0–15)
 *   offset 25: next_match_offset           u8   — offset within first block for next match (0–15)
 *   offset 26: num_order_blocks_produced   u16  — NOBP: total blocks ever created
 *   offset 28: num_order_blocks_consumed   u16  — NOBC1: blocks fully matched and removed
 *   offset 30: num_order_blocks_compacted  u16  — NOBC2: blocks merged via compaction
 */

pub struct PriceNode([u8; 32]);

impl PriceNode {
    pub fn last_slot_update(&self) -> u64 {
        u64::from_le_bytes(self.0[0..8].try_into().unwrap())
    }

    pub fn set_last_slot_update(&mut self, val: u64) {
        self.0[0..8].copy_from_slice(&val.to_le_bytes());
    }

    pub fn price(&self) -> u64 {
        u64::from_le_bytes(self.0[8..16].try_into().unwrap())
    }

    pub fn set_price(&mut self, val: u64) {
        self.0[8..16].copy_from_slice(&val.to_le_bytes());
    }

    pub fn parent_pointer(&self) -> u32 {
        u32::from_le_bytes(self.0[16..20].try_into().unwrap())
    }

    pub fn set_parent_pointer(&mut self, val: u32) {
        self.0[16..20].copy_from_slice(&val.to_le_bytes());
    }

    pub fn order_block_pointer(&self) -> u32 {
        u32::from_le_bytes(self.0[20..24].try_into().unwrap())
    }

    pub fn set_order_block_pointer(&mut self, val: u32) {
        self.0[20..24].copy_from_slice(&val.to_le_bytes());
    }

    pub fn next_entry_offset(&self) -> u8 {
        self.0[24]
    }

    pub fn set_next_entry_offset(&mut self, val: u8) {
        self.0[24] = val;
    }

    pub fn next_match_offset(&self) -> u8 {
        self.0[25]
    }

    pub fn set_next_match_offset(&mut self, val: u8) {
        self.0[25] = val;
    }

    pub fn num_order_blocks_produced(&self) -> u16 {
        u16::from_le_bytes(self.0[26..28].try_into().unwrap())
    }

    pub fn set_num_order_blocks_produced(&mut self, val: u16) {
        self.0[26..28].copy_from_slice(&val.to_le_bytes());
    }

    pub fn num_order_blocks_consumed(&self) -> u16 {
        u16::from_le_bytes(self.0[28..30].try_into().unwrap())
    }

    pub fn set_num_order_blocks_consumed(&mut self, val: u16) {
        self.0[28..30].copy_from_slice(&val.to_le_bytes());
    }

    pub fn num_order_blocks_compacted(&self) -> u16 {
        u16::from_le_bytes(self.0[30..32].try_into().unwrap())
    }

    pub fn set_num_order_blocks_compacted(&mut self, val: u16) {
        self.0[30..32].copy_from_slice(&val.to_le_bytes());
    }

    /* Derived value: NOBP - (NOBC1 + NOBC2) */
    pub fn num_order_blocks_live(&self) -> u16 {
        self.num_order_blocks_produced()
            .saturating_sub(self.num_order_blocks_consumed() + self.num_order_blocks_compacted())
    }
}
