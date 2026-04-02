# Incognitus — Core Systems Deep Dive

> **Version:** 1.8
> **Status:** DRAFT — being iterated on. WIP sections are explicitly marked.
> Structures and behaviors will be updated as implementation progresses.
> **Changelog:**
> - v0.1 — Initial draft
> - v0.2 — Part 1 (Slab Allocator) fully rewritten: raw byte addresses (not slot indices),
>           correct header structure with flags layout, correct stack-node variants with
>           byte layouts, correct insert/delete operations from implementation, validate
>           memory and resize described, discriminator on SlabNode trait noted, traversal
>           algorithm as the connector between slabs noted. Removed all slot index
>           references, removed hallucinated invariants, removed incorrect "slab is ignorant
>           of node type" claim.
> - v0.3 — 1.1 insert return value statement removed, no slot indices claim scoped to slab,
>           arrow direction fixed in account layout diagram
> - v0.4 — 2.1 rewritten: TrieLink removed as own slab, order book framed as composed of
>           two sides, Sealevel reference removed, matching/update context added. 2.2
>           hierarchy diagram updated with TrieLink as embedded, three-way branching from
>           TrieLink and PriceNode, OrderBlockLinkNode and OrderBlockTreeNode added.
>           OrderTreeNode renamed to OrderBlockTreeNode throughout. All slot index references
>           replaced with raw byte addresses in 2.4 and 2.5.
> - v0.5 — 2.3 fully updated: concept framing paragraph added, depth direction corrected
>           (root at highest depth, leaf at depth 0), trailing zeros behavior added with
>           example, slab_get_mut removed from delta propagation, bit offset note added to
>           structure table, WIP notes added to traversal and delta propagation
> - v0.6 — 2.4 updated: notes added on size field (may become padding/flag), parent_trie_node
>           (path stack vs parent pointer CU decision), is_root (may be redundant)
> - v0.7 — 2.5 updated: 0 children clarified as leaf case, 1–10 TrieNode children for
>           internal nodes, assumption on child count noted, chain capacity corrected to
>           3+3+4=10, variant field description clarified
> - v0.8 — 2.4 pointer cases diagram added showing all three TrieLink resolution options
> - v0.9 — 2.6 updated: notes added on order_block_pointer three-way resolution, parent_pointer
>           CU cost decision, num_order_blocks may be derivable, WIP note on structure,
>           compaction potential if parent_pointer removed
> - v1.0 — 2.7 updated: batching benefit clarified, memory locality noted as assumption
>           needing benchmarking, Approach 1 eliminated, Approach 2 expanded with delta
>           fields, Approaches 2 and 3 to be benchmarked, header prefix removed from
>           structure, OrderBlock does not get additional pointer fields
> - v1.1 — 2.8 fixed: slot indices → raw byte addresses in coordinate description.
>           2.9 fixed: Approach 1 reference removed, only Approach 2 as alternative to 3
> - v1.2 — 2.9 fully rewritten: correct traversal concept, delta-guided and stack-based
>           traversal, insertion left to right, null pointer cases (matching and compaction),
>           depth-based leaf encoding, raw byte addresses. 2.10 added: OrderBlockLinkNode
>           with delta fields, same encoding concept as TrieLinkNode
> - v1.3 — 2.10 structure finalized: 32 bytes, 3 OrderBlock pointers + variant field,
> - v1.4 — Part 3
> - v1.5 — 2.6 PriceNode updated
> - v1.6 — Part 3 WIP note added
> - v1.7 — Part 4 updated: 4.3 slot indices → raw byte addresses, 4.6 i64 → u64 and
>           size corrected to 32 bytes, 4.7 link nodes corrected to variant field pattern,
>           4.8 Stage 8 references removed and reframed around order events, 4.9 capacity
>           table updated for 32-byte OrderEntryChangeLogNode
> - v1.8 — Appendix updated: A.1 stage references removed, A.2 slot indices → raw byte
>           addresses, A.3 slab_respond_to_allocation_event removed, resize framed as
>           separate instruction not a function call at top, 3.17 added documenting undefined source accounts,
>           cache structure, and order-event-queue structure as not yet designed: next_entry/match_offset shrunk to 1 byte, num_order_blocks
>           replaced with produced/consumed/compacted fields, num_order_blocks_live as derived.
>           2.7 OrderBlock updated: 408 bytes, block_id start/end for compaction tracking,
>           24-byte header. Technical spec 1.7 and 1.8 updated to match. (Matching Engine) fully rewritten: 3.3 replaced with three traversal
>           methods, 3.4 order of operations added with all 10 operations, 3.5 cache and
>           order-event-queue added, 3.6 market order processing and batching rules added,
>           cleanup flagging reframed as traversal gate, section numbers updated, 3.15
>           TestSlab and 3.16 Interface expanded with deeper explanations and WIP notes
>           same pattern as TrieLinkNode
>
> **Purpose:** A single reference document for the four core systems of Incognitus:
> the Slab Allocator, the Order Book collection, the Matching Engine, and the Ledger
> collection. This document goes deeper than the overview and broader than the technical
> spec. It is the document you keep open while building.
>
> **What this document is NOT:**
> - A replacement for the technical spec (`06-technical-spec-v2.md`) — go there for exact
>   byte layouts and PDA seeds
> - A replacement for the algorithm design doc (`07-algorithm-design-v2.md`) — go there
>   for pseudocode and worked examples
> - A pipeline description — see `00-overview.md` Section 9 for that
>
> **How to use this document:**
> Read Part 1 (Slab Allocator) first — everything else builds on it.
> Then read whichever part is relevant to your current work.

---

## Table of Contents

1. [Part 1 — Slab Allocator](#part-1--slab-allocator)
2. [Part 2 — Order Book](#part-2--order-book)
3. [Part 3 — Matching Engine](#part-3--matching-engine)
4. [Part 4 — Ledger](#part-4--ledger)
5. [Appendix — Open Integration Questions](#appendix--open-integration-questions)

---

---

# Part 1 — Slab Allocator

---

## 1.1 Mental Model

A slab is a **flat byte array** — a contiguous account data buffer — that manages fixed-size nodes using raw byte addresses. That's it.

Within the slab, pointers are raw byte addresses — u32 values that are literal byte offsets into the account data buffer. There are no multiplications. There is just a byte address and the account data buffer.

> **Note:** In other parts of the system, slot indices and logical indices do exist. That context will be covered when we get there. This section describes the slab allocator specifically.

**Why this maps to Solana:**

Solana accounts are fixed in size at creation. A slab allocator maps perfectly to this — all dynamic behavior (inserting and deleting nodes) happens within a fixed-size byte buffer. Nodes are written sequentially until the account fills up or deleted addresses are available for reuse. When free space drops below a threshold, the account grows. The slab manages this entirely within the account.

**What the slab knows:**

Each slab stores one node type. The node type is encoded in a `discriminator` constant on the `SlabNode` trait — for example TrieNode has `DISCRIMINATOR = 0x5452_4945` (ASCII "TRIE"). The slab uses this to identify what it is managing.

The slab does not manage relationships between slab accounts. TrieNode and TrieLinkNode slabs work together — but that relationship is managed by the traversal algorithm, not by the slab itself. The slab provides raw node storage. The traversal algorithm knows which slab to look in when dereferencing a pointer.

---

## 1.2 Account Layout

```
┌──────────────────────────────────────────────┐  ← byte 0 (start of account)
│                                              │
│  Header  (24 bytes)                          │
│  discriminator, root_node_pointer,           │
│  stack_pointer, allocator_size,              │
│  num_node_elements, flags                    │
│                                              │
├──────────────────────────────────────────────┤  ← header_size (aligned to node_size)
│                                              │
│  Node Section                                │
│                                              │
│  nodes written sequentially from here        │
│  each node accessed by its raw byte address  │
│                                              │
│  grows ↓                                     │
│                                              │
│  (free space)                                │
│                                              │
│  Stack Section                               │
│  u32 raw byte addresses of deleted nodes     │
│  grows ↑ toward node section                 │
│                                              │
├──────────────────────────────────────────────┤  ← end - StackNode::SIZE
│                                              │
│  Stack-Node  (16 bytes)                      │
│  HeadNode — always at end of account         │
│                                              │
└──────────────────────────────────────────────┘  ← end of account
```

Accounts start at 10kb and grow in 10kb increments up to a maximum of 10mb. Growth is triggered by the `validate_memory` check and handled by a separate resize instruction. While growing, the `FLAG_STILL_GROWING` flag is set and the stack-node structure is active.

---

## 1.3 Header — 24 bytes

```
offset  0: discriminator       u32   node type identifier (e.g. 0x5452_4945 for TrieNode)
offset  4: root_node_pointer   u32   raw byte address of root node
                                     initialized to 0x0000_0000
                                     updated by the node layer when a new root is established
offset  8: stack_pointer       u32   raw byte address — top of stack
                                     used for deletes in normal state
                                     used for deletes in fragmented state (FLAG_FRAGMENTED_STACK)
offset 12: allocator_size      u32   total account byte size
offset 16: num_node_elements   u32   count of live nodes currently in the slab
offset 20: flags               u32   bitmask — see flag layout below
```

**Flag layout:**

```
byte 3 (0xFF00_0000) — resize interrupt
                       resize must be handled before insert/delete can proceed
byte 2 (0x00FF_0000) — fragmented stack state
                       set after account growth while stack sections are being merged
                       min-swap optimization suspended while set
byte 1 (0x0000_FF00) — validation flag (concept, not yet implemented)
byte 0 (0x0000_00FF) — still growing / max allocated state
                       set while account is below max size
                       stack-node structure is active while this is set
                       once account reaches max size, flag is cleared
                       when cleared and fragmented stack = 0, stack-node structure
                       is no longer used and a simpler stack structure applies
```

---

## 1.4 Stack-Node — 16 bytes, four variants

The stack-node lives at the end of the account. It manages the free address stack and tracks where the next sequential write goes. Four variants exist depending on the account's state.

**Stack-node relationships:**

```
[head-node]                                    ← normal / default state
[tail-node] → [head-node]                      ← typical transitional state during growth
[tail-node] → [linked-node; n] → [head-node]  ← rare, rapid growth fallback
```

The head-node is always at the end of the account. When the account grows, a new head-node is written at the new end, and the old head-node transitions into a tail-node or linked-node. The chain adapts as the account expands.

---

**HeadNode — always at end of account**

```
offset  0: next_seq_index            u32   raw byte address of next sequential write
offset  4: stack_entry_cycle_pointer u32   cycles through stack entries for min-swap optimization
                                           active when stack_node_type_flag = 0x0000_0000
                                           and flags byte 2 = 0x00
offset  8: stack_node_type_flag      u32   0x0000_0000 = head
offset 12: stack_pointer             u32   used for inserts when flags byte 0 = 0xFF (still growing)
           single_stack_node_flag    u32   0x0000_0000 = only stack-node (when flags byte 0 = 0x00)
```

---

**TailNode — exists when flags byte 2 = 0xFF (fragmented stack)**

```
offset  0: top_boundary_flag         u32   0xFFFF_FFFF
offset  4: next_stack_node_pointer   u32   raw byte address of next stack-node in chain
offset  8: stack_node_type_flag      u32   0xFFFF_FFFF = tail
offset 12: bottom_boundary_flag      u32   0xFFFF_FFFF
```

---

**LinkedNode — exists when flags byte 2 = 0xFF (rare, rapid growth)**

```
offset  0: top_boundary_flag         u32   0xFFFF_FFFF
offset  4: next_stack_node_pointer   u32   raw byte address of next stack-node in chain
offset  8: tail_stack_entry_pointer  u32   raw byte address of last stack entry in this node's section
offset 12: bottom_boundary_flag      u32   0xFFFF_FFFF
```

---

**DefaultNode — final state when fully grown and not fragmented (8 bytes)**

```
offset  0: next_seq_index            u32
offset  4: min_stack_node_pointer    u32
```

---

## 1.5 Insert

```rust
pub fn insert<N: SlabNode>(&mut self, node_bytes: &[u8]) -> Result<u32, SlabError>
```

Returns the raw byte address where the node was written.

**Routing:**

```
if resize_interrupt flag set:
  → return ResizeInterrupt

sp        = header.stack_pointer
head_addr = allocator_size - StackNode::SIZE

if sp < head_addr:
  // recycled free addresses exist on the stack — pop path
  dest_addr = pop_insert(sp)
else:
  // stack is empty — sequential write path
  dest_addr = seq_insert()

write node_bytes to data[dest_addr .. dest_addr + N::SIZE]
header.num_node_elements += 1
return dest_addr
```

**Pop path — recycled address available:**

```
pop_insert(sp):
  entry = read_addr(sp)

  if entry == FULL_FLAG:
    // hit a boundary sentinel from a stack-node transition
    remove_tail_stack_node(sp)
    new_sp = header.stack_pointer   // updated by remove_tail
    dest   = read_addr(new_sp)
    header.stack_pointer = new_sp + 4
    return dest
  else:
    header.stack_pointer = sp + 4
    return entry
```

**Sequential path — no recycled addresses:**

```
seq_insert():
  next_seq  = head_node.next_seq_index
  stack_top = header.stack_pointer

  if next_seq + N::SIZE > stack_top:
    → return OutOfSpace

  head_node.next_seq_index = next_seq + N::SIZE
  validate_memory()
  return next_seq
```

---

## 1.6 Delete

```rust
pub fn delete<N: SlabNode>(&mut self, node_addr: u32) -> Result<(), SlabError>
```

Takes the raw byte address of the node to remove.

```
if resize_interrupt flag set:
  → return ResizeInterrupt

sp     = header.stack_pointer
new_sp = sp - 4

if read_addr(new_sp) == FULL_FLAG:
  merge_stack_node(new_sp)   // merge tail into head on boundary hit

write_addr(new_sp, node_addr)   // push raw byte address onto stack
header.stack_pointer = new_sp

if not fragmented_stack:
  maybe_swap_min(node_addr, new_sp)   // min-swap optimization
  advance_cycle_pointer()

zero node bytes at node_addr
header.num_node_elements -= 1
validate_memory()
```

**Min-swap optimization:**

On each delete, the `stack_entry_cycle_pointer` advances one position through the stack. It compares the newly pushed address against the address at the current cycle position. If the new address is lower (closer to the node section start), they are swapped. Over time this ensures lower addresses bubble toward the top of the stack and get reused first, keeping the node section compact.

---

## 1.7 Validate Memory

Called after every sequential insert and every delete. Checks whether free space has dropped below a threshold. If the threshold is crossed, sets the resize interrupt flag — no further inserts or deletes can proceed until a separate resize instruction handles the account growth.

> **WIP:** The exact threshold formula is still being iterated on. The intent is to detect when the ratio of used space to total space crosses a defined limit and trigger account growth before the slab runs out of space entirely.

---

## 1.8 Resize

Account growth is handled in a separate instruction — not inline with insert or delete. When `validate_memory` sets the resize interrupt flag:

1. Normal operations halt — insert and delete both return `ResizeInterrupt`
2. The resize instruction runs: grows the account, writes the new HeadNode at the new end, transitions the old HeadNode into a TailNode or LinkedNode, sets `FLAG_FRAGMENTED_STACK`
3. During `FLAG_FRAGMENTED_STACK`: inserts use the new HeadNode's stack region, deletes use `header.stack_pointer` into the old stack region, the min-swap optimization is suspended
4. As the two stack sections are merged over subsequent operations, `FLAG_FRAGMENTED_STACK` is cleared and normal operation resumes

This ignorance is a design choice. It keeps the slab simple, testable, and composable. The layer above the slab — the matching engine and the Solana program — provides all the context.

---

---

# Part 2 — Order Book

---

## 2.1 The Order Book as a Collection

The order book is not a single data structure. It is a **collection of slab accounts**, each holding one type of node. The nodes together form the complete order book structure.

The order book is composed of two sides — bid and ask. Each side has its own set of slab accounts. Bid and ask never share an account.

```
Order Book
  ├── Bid Side
  │     ├── slab[TrieNode][bid][market][0]        ← price trie nodes
  │     ├── slab[TrieLinkNode][bid][market][0]    ← child pointer chains
  │     ├── slab[PriceNode][bid][market][0]       ← price level records
  │     ├── slab[OrderBlock][bid][market][0]      ← order entry batches
  │     └── slab[OrderBlockTreeNode][bid][market][0]  ← tree traversal (if chosen)
  │
  └── Ask Side
        └── (same structure, separate accounts)
```

During matching, a market order is matched against its corresponding side — a market buy matches against the ask side, a market sell matches against the bid side. During order book updates, work can be batched by side and by action (open or cancel).

**The order book is persistent state.** It survives across pipeline batches. It is not a transient queue — it is the live state of all resting limit orders.

---

## 2.2 The Hierarchy

The order book data structures form a hierarchy. TrieLink is embedded inside TrieNode — it is not a separate slab node. It is the decision point that determines what a TrieNode points to. PriceNode similarly has three options for how it manages its OrderBlock collection depending on the traversal approach chosen.

```
TrieNode (price index)
  └── trie_link: TrieLink (embedded — not a separate slab node)
        ├── → TrieNode (single child)
        ├── → TrieLinkNode (multiple children)
        │     └── → TrieNode (each child)
        └── → PriceNode (leaf)
              ├── → OrderBlock (single block)
              │     └── OrderEntry (×16, within block)
              ├── → OrderBlockLinkNode (multiple blocks — link approach)
              │     └── → OrderBlock
              │           └── OrderEntry (×16, within block)
              └── → OrderBlockTreeNode (multiple blocks — tree approach)
                    └── → OrderBlock
                          └── OrderEntry (×16, within block)
```

Traversal flows from the top down. Navigation uses raw byte address pointers — u32 values that are literal byte offsets into a specific slab account's data. These are not Solana addresses.

Each component serves a distinct purpose:
- **TrieNode** — represents one decimal digit at one depth in the price trie. Stores aggregate deltas for the subtree below it.
- **TrieLink** — embedded in TrieNode. Encodes what the node points to and carries child tracking flags and the cleanup flag.
- **TrieLinkNode** — support node for TrieNode with multiple children. Chains up to 3 child pointers per node.
- **PriceNode** — manages the collection of entries at one price level. Tracks insertion and match positions.
- **OrderBlock** — batches up to 16 OrderEntry records. Stores aggregate deltas for entries within it.
- **OrderBlockLinkNode** — support node for PriceNode with multiple blocks (link approach). Under evaluation.
- **OrderBlockTreeNode** — support node for PriceNode with multiple blocks (tree approach). Under evaluation.
- **OrderEntry** — the atomic unit. One resting limit order, fully specified.

---

## 2.3 TrieNode — The Price Index

### Concept

The order book needs to answer one question efficiently: given a market order, what is the best available price that can fill it?

The price trie is the answer. It organizes all resting limit orders by price using the decimal digits of the price as the key. Each level of the trie represents one decimal position. Traversing the trie from the root downward narrows in on a specific price level one digit at a time.

What makes this more than a standard trie is the delta fields on every node. Each TrieNode stores the sum of all open quantities and quotes in its entire subtree. This means the trie can answer not just "does price X exist?" but "which price level can fill a quantity of N?" without scanning every price level.

The order book is a price trie — a prefix tree keyed on the decimal digits of a price. Each `TrieNode` represents one decimal digit at one depth. The root is at the highest depth and traversal descends toward lower depths. Depth 0 is the least significant digit (ones place).

For price 1234:
- digit 1 is at depth 3 (1 × 10^3)
- digit 2 is at depth 2 (2 × 10^2)
- digit 3 is at depth 1 (3 × 10^1)
- digit 4 is at depth 0 (4 × 10^0)

**Trailing zeros:** Traversal stops at the last non-zero digit. That TrieNode points to the PriceNode — no nodes are created for trailing zeros. For price 1200, traversal stops at digit 2 at depth 2, and that node points to the PriceNode for price 1200.

```
Price: 1234

Root
  └── TrieNode(digit=1, depth=3)   delta_quantity=500, delta_quote=617,000
        └── TrieNode(digit=2, depth=2)   delta_quantity=500, delta_quote=617,000
              └── TrieNode(digit=3, depth=1)   delta_quantity=500, delta_quote=617,000
                    └── TrieNode(digit=4, depth=0) [leaf → PriceNode]
                          └── PriceNode(price=1234)

Price: 1200

Root
  └── TrieNode(digit=1, depth=3)
        └── TrieNode(digit=2, depth=2) [leaf → PriceNode]
              └── PriceNode(price=1200)
```

Each TrieNode stores `delta_quantity_total` and `delta_quote_total` — the **sum of all open order quantities and quotes in its entire subtree**. This is the key insight that enables three-dimensional traversal.

### Three-Dimensional Traversal

> **Note:** The exact traversal approach may change during implementation as we identify more precisely how it would work in practice.

Because every TrieNode carries subtree delta sums, the matching engine can answer three different navigation questions in a single downward pass:

**Dimension 1 — By price:** Walk digit by digit to find a specific price level. Standard trie traversal.

**Dimension 2 — By quantity:** At each node, check `delta_quantity_total >= target_quantity`. If the subtree doesn't have enough, don't enter it. Descend only into subtrees that can fill the order. No price-level scanning.

**Dimension 3 — By quote:** Same as by quantity but using `delta_quote_total`. Useful for orders specified in quote currency rather than base quantity.

This is what distinguishes the Incognitus matching engine. Every existing on-chain CLOB can only answer "what is the best price?" The delta fields make it possible to also answer "what is the best price at which I can fill 10,000 units?" without scanning all price levels.

### Structure — 32 bytes

> **Note:** Offsets are in bits using the Rust bit offset convention from the source. Byte offsets: delta_quantity_total=0, delta_quote_total=8, value=16, trie_link=20.

```
Field                  Size   Offset (bits)   Description
delta_quantity_total   8      0               Sum of all open quantity in subtree
delta_quote_total      8      64              Sum of all open quote in subtree
value: (u16, u16)      4      128             (decimal digit, depth)
trie_link: TrieLink    12     160             Child pointer and flags (see 2.4)
```

**On `value`:** The two u16 fields pack the decimal digit (0–9) and depth into a single 4-byte field. Depth 0 is the least significant digit (ones place). The root is at the highest depth.

**On `delta_quantity_total` and `delta_quote_total`:** These are always kept in sync with reality. Every insert, fill, cancel, and cleanup propagates delta changes upward through the path from the affected leaf to the root. The delta invariant must hold at all times:

```
For every TrieNode:
  node.delta_quantity_total == sum of direct_children[i].delta_quantity_total

For every leaf TrieNode:
  node.delta_quantity_total == PriceNode(pointed to).delta_quantity_total
```

If this invariant is violated, the matching engine will make wrong navigation decisions — filling orders incorrectly or missing available liquidity.

### Delta Propagation

> **Note:** The exact delta propagation approach may change during implementation as we identify more precisely how it would work in practice.

When an entry is inserted, filled, or removed, deltas must propagate from the affected leaf up to the root. The path is tracked during downward traversal using a path stack — each TrieNode raw byte address is pushed as we descend, then the stack is iterated on the way back up:

```
propagate_delta(path_stack, quantity_change, quote_change):
  for each addr in path_stack (bottom to top):
    node = &mut data[addr]
    node.delta_quantity_total += quantity_change   // negative for fills/cancels
    node.delta_quote_total    += quote_change
```

This gives O(log n) propagation where n is the number of decimal digits in the price — effectively constant for practical prices.

### Cleanup Flagging

When all orders at a price level are exhausted — fully matched or cancelled — the trie path above it needs cleanup. Rather than immediately traversing and freeing every ancestor, the matching engine uses the `is_active` flag (bit 14 in TrieLink.flags) to lazily signal that a subtree is no longer active.

When the top-level parent node that no longer has active children is flagged as inactive, all of its children are implicitly inactive — no need to traverse them. The cleanup stage uses this flag to efficiently identify what to free.

---

## 2.4 TrieLink — Child Pointer and Flags

### Concept

`TrieLink` is embedded inside every `TrieNode`. It serves three roles simultaneously:
1. It encodes what the node points to — a single child TrieNode, a TrieLinkNode chain, or a PriceNode
2. It tracks how many children this node has and which digits they represent
3. It carries the `is_active` cleanup flag

### Structure — 12 bytes

> **Note:** This structure may shrink to 8 bytes if the path stack approach is chosen for delta propagation — see `parent_trie_node` note below.

```
Field                  Size   Byte Offset   Description
flags: u16             2      0             Bitmask — see flag layout below
padding: u8            1      2             —
size: u8               1      3             Number of children (max 10)
parent_trie_node: u32  4      4             Pointer to parent TrieNode
pointer: u32           4      8             Raw byte address into slab identified by bits 11–13
```

**On `size`:** Tracks the number of children. The digit bitmask in bits 0–9 makes this derivable by counting set bits, but keeping it as an explicit field avoids that computation on every access. If more flags are needed in the future, this field may become padding or be repurposed as additional flags.

**On `parent_trie_node`:** Stores a raw byte address pointing back to the parent TrieNode. This enables upward delta propagation without a path stack. However, if the path stack approach is chosen for delta propagation, this field is not needed — the structure would shrink from 12 bytes to 8 bytes. The decision between the two approaches is based on which has the least CU cost and will be determined during benchmarking.

**Flag layout (u16 at offset 0):**

```
bits 0–9:   digit bitmask
              bit N is set if a child with digit=N exists
              e.g. children with digits 0, 3, 7 → 0b0000_0010_1001

bit 10:     is_root
              under consideration — may be redundant
              root can be identified by parent_trie_node == 0x0000_0000
              may be removed in a future iteration

bit 11:     is TrieLinkNodePointer
              pointer field — raw byte address of a TrieLinkNode

bit 12:     is TrieNodePointer
              pointer field — raw byte address of a TrieNode (single-child optimization)

bit 13:     is PriceNodePointer
              pointer field — raw byte address of a PriceNode (leaf)

bit 14:     is_active
              when unset on a parent, all children are implicitly inactive
              set for all active nodes, cleared during cleanup

bit 15:     reserved
```

**Pointer type encoding (RESOLVED — R-021):**

Exactly one of bits 11, 12, 13 must be active. The slab type is inferred from the flags — not encoded in the pointer value. If none or more than one are set, the state is invalid.

The `pointer` field is a 4-byte raw byte address. To dereference it, the caller reads bits 11–13 to determine which slab account to look in. The pointer is just a number — the slab context comes from the flags.

```
TrieNode.trie_link.pointer:
  ├── → TrieNode          (bit 12 set — single child)
  ├── → TrieLinkNode[0]   (bit 11 set — multiple children)
  │     [ptr_a, ptr_b, ptr_c, variant]
  │                         ↓
  │    TrieLinkNode[1] (if needed)
  │     [ptr_d, ptr_e, ptr_f, variant]
  │                         ↓
  │    TrieLinkNode[2] (last node)
  │     [ptr_g, ptr_h, ptr_i, ptr_j]
  └── → PriceNode          (bit 13 set — leaf)
```

**Why this design:** Keeping the pointer as a bare raw byte address keeps the TrieLink compact. The flag bits are already there for child tracking. Reusing them for pointer type costs nothing and avoids inflating the pointer field. The caller is already reading the flags to make traversal decisions — reading two extra bits adds negligible cost.

---

## 2.5 TrieLinkNode — Child Pointer Chain

### Concept

A `TrieNode` can have 1 to 10 TrieNode children — one per decimal digit (0–9). When a TrieNode has no TrieNode children, it is a leaf and its TrieLink pointer points to a PriceNode instead (bit 13 set). Storing all 10 possible child pointers inline would add 40 bytes to every TrieNode, bloating the slab.

The solution is a two-tier child storage strategy:

**Tier 1 — Single child:** When `trie_link.size == 1`, the pointer field points directly to the child TrieNode. No TrieLinkNode needed. Bit 12 (is TrieNodePointer) is set.

**Tier 2 — Multiple children:** When `trie_link.size > 1`, the pointer field points to the head of a TrieLinkNode chain. Bit 11 (is TrieLinkNodePointer) is set.

> **Note:** In practice the number of children per node is unlikely to be maxed out. At the top of the trie (fewer shared price prefixes) and the bottom (final digits narrow quickly) there tend to be fewer children. The middle section — where prices cluster within a similar range — could see more children. This is an assumption that will be validated during implementation.

```
Single child (size=1):
  TrieNode.trie_link.pointer → TrieNode (bit 12 set)

Multiple children (size=2–10):
  TrieNode.trie_link.pointer → TrieLinkNode[0]  (bit 11 set)
                                  [ptr_a, ptr_b, ptr_c, variant]
                                                    ↓
                               TrieLinkNode[1] (if needed)
                                  [ptr_d, ptr_e, ptr_f, variant]
                                                    ↓
                               TrieLinkNode[2] (last node)
                                  [ptr_g, ptr_h, ptr_i, ptr_j]
                                  (variant = 4th TrieNodePointer on last node)
```

**Chain capacity:** Each TrieLinkNode holds 3 TrieNodePointers + 1 variant field. On intermediate nodes the variant is a pointer to the next TrieLinkNode. On the last node in the chain, the variant is a 4th TrieNodePointer — determined by the encoding in the pointer that led to this node. Chain of three: 3 + 3 + 4 = 10 children total.

### Structure — 16 bytes

```
Field                  Size   Byte Offset   Description
TrieNodePointer[0]     4      0             First child raw byte address
TrieNodePointer[1]     4      4             Second child raw byte address
TrieNodePointer[2]     4      8             Third child raw byte address
variant: u32           4      12            Next TrieLinkNode | 4th TrieNodePointer
                                            encoding determined by the pointer that
                                            led to this TrieLinkNode
```

**On the variant field:** Whether the last field is a pointer to the next TrieLinkNode or a 4th TrieNodePointer is known before reading it — the flag lives in the pointer structure of the node that points to this TrieLinkNode. This keeps the TrieLinkNode itself to exactly 16 bytes with no ambiguity at read time.

---

## 2.6 PriceNode — Price Level Record

### Concept

When a TrieNode trie traversal reaches a leaf — a TrieNode whose `trie_link` carries bit 13 (is PriceNodePointer) — it arrives at a `PriceNode`. The PriceNode is the record for one specific price level. It holds everything needed to navigate the order entries at that price.

The PriceNode's job is to manage the **collection of OrderBlocks at its price level** and to track two independent cursors:

- **The entry cursor (`next_entry_offset`)** — points to where the next new limit order entry should be written within the most recently allocated OrderBlock. This cursor is at the tail of the block collection — the newest block.

- **The match cursor (`next_match_offset`)** — points to where the next match should be read from within the oldest unconsumed OrderBlock. This cursor is at the head of the block collection — the oldest block.

These two cursors move in the same direction (forward through entries within a block, 0 → 1 → 2 → ... → 15) but they operate on **different blocks**. The entry cursor is always on the newest block. The match cursor is always on the oldest unconsumed block. In a live market with continuous order flow, there may be many blocks between the two.

### The Two Cursors in Detail

**Entry cursor behavior (`next_entry_offset`):**

```
New entry arrives → write to current last block at position next_entry_offset
next_entry_offset advances: 0 → 1 → 2 → ... → 15

When next_entry_offset reaches 16:
  → reset next_entry_offset to 0
  → allocate new OrderBlock
  → new block becomes the last block
  → link new block into the collection
```

**Match cursor behavior (`next_match_offset`):**

```
Match needed → read entry from current first block at position next_match_offset
next_match_offset advances: 0 → 1 → 2 → ... → 15

When next_match_offset reaches 16:
  → reset next_match_offset to 0
  → mark current first block for cleanup
  → advance to next block in collection (it becomes the new first block)
```

**FIFO guarantee:** Because the entry cursor writes to the newest block and the match cursor reads from the oldest block, entries are always matched in the order they were inserted. This is the FIFO guarantee within a price level — the first limit order to arrive is the first to be matched.

### Structure — 32 bytes

> **WIP:** Fields look valid but may change during implementation as we discover things we haven't thought about yet. If `parent_pointer` is removed (not needed if path stack approach is used for traversal) that space becomes padding and the structure remains 32 bytes.

```
Field                       Size   Byte Offset   Description
last_slot_update            8      0             Slot when this node was last modified
price                       8      8             The price value this node represents
parent_pointer              4      16            Pointer back to parent TrieNode
                                                 Not needed if path stack approach chosen —
                                                 becomes padding in that case
order_block_pointer         4      20            Pointer to OrderBlock collection
                                                 Resolves to one of three cases:
                                                   → single OrderBlock (if only one exists)
                                                   → OrderBlockLinkNode (link approach)
                                                   → OrderBlockTreeNode (tree approach)
                                                 May need flags to encode which case
next_entry_offset           1      24            Offset within last block for next insert (0–15)
                                                 Resets to 0 and creates new block after 16
next_match_offset           1      25            Offset within first block for next match (0–15)
                                                 Resets to 0 and advances block after 16
num_order_blocks_produced   2      26            NOBP — total blocks ever created at this price
num_order_blocks_consumed   2      28            NOBC1 — blocks fully matched and removed
num_order_blocks_compacted  2      30            NOBC2 — blocks merged via compaction
```

**`num_order_blocks_live` is a derived value — never stored:**
```
num_order_blocks_live = NOBP - (NOBC1 + NOBC2)
```

---

## 2.7 OrderBlock — Entry Batch

### Concept

An `OrderBlock` is a fixed-capacity batch of `OrderEntry` records. Batching entries into a block allows multiple entries to be written in a single operation — more efficient than handling each entry individually. The 16 entries within a block are contiguous by structure, which is good for reading them sequentially during matching.

> **Note:** 16 entries per block is a valid educated assumption. The optimal batch size needs to be validated through benchmarking.

The block stores aggregate delta totals for all entries within it, mirroring the pattern of TrieNode storing subtree totals. This enables efficient navigation when multiple blocks exist at a price level.

Each entry in the block stores **pre-computed exchange amounts** — the quantity to send and the quantity to receive at the given price, computed off-chain by the client and validated on-chain. The matching engine does not recompute these at match time. It finds the right entry and reads the values directly.

### Traversal Approach — Under Evaluation

When the matching engine arrives at a PriceNode with multiple OrderBlocks, it needs a way to navigate them. Three approaches were considered — only two will be benchmarked:

**Approach 1 — Doubly Linked List (eliminated)**

Each OrderBlock stores `next` and `prev` pointers. Traversal is linear from head to tail. This approach is eliminated — it takes too much space and there is no meaningful way to aggregate delta totals across blocks.

**Approach 2 — OrderBlockLinkNode Batching**

Works on the same concept as TrieLinkNode with the same encoding approach. Each OrderBlockLinkNode batches multiple OrderBlock pointers. The head of the structure includes `delta_quantity_total` and `delta_quote_total` for the blocks it contains, providing delta aggregation at the link node level.

**Approach 3 — Tree with Delta Aggregation (primary candidate)**

A binary tree of OrderBlockTreeNode records over the OrderBlock collection. The root node carries the sum of all deltas across all blocks. Each subtree node carries the sum of its own subtree. This mirrors the TrieNode pattern at the price level — the same delta-guided navigation is available at the block level.

```
PriceNode
  └── OrderBlockTreeNode (root, delta=sum of all blocks)
         ├── OrderBlockTreeNode (delta=sum of its subtree)
         │    └── OrderBlocks
         └── OrderBlockTreeNode (delta=sum of its subtree)
              └── OrderBlocks
```

**Why Approach 3 is the primary candidate:**

Without per-block delta, the matching engine cannot make directed decisions at the block level — it must always scan linearly. With the tree, the same delta-guided navigation that TrieNode enables at the price level is available at the block level:

```
TrieNode           → navigate to best price level by delta
OrderBlockTreeNode → navigate to best OrderBlock within price level by delta
OrderBlock         → consume OrderEntry records in FIFO order
```

The architectural coherence of the three-level delta hierarchy is the primary argument. The counter-argument is complexity and space overhead. Approaches 2 and 3 will be benchmarked — the decision is made with data. See D-013 in `02-decision-log-v2.md`.

### Structure — 408 bytes

> **WIP:** The OrderBlock itself does not get additional pointer fields — the traversal structure (OrderBlockLinkNode or OrderBlockTreeNode) lives in separate slab nodes that point to OrderBlocks.

**Header (24 bytes):**
```
Field                  Size   Byte Offset   Description
block_id_start: u16    2      0             This block's ID (or start of compacted range)
block_id_end: u16      2      2             0x0000 = no compaction
                                            0x_xxxx = end of compacted block ID range
                                            if end != 0x0000 then start < end
_padding               4      4             alignment
delta_quantity_total   8      8             Sum of all entry quantity_amounts in this block
delta_quote_total      8      16            Sum of all entry quote_amounts in this block
```

**Entries (384 bytes):**
```
entries: [OrderEntry; 16]   ← 16 × 24 bytes = 384 bytes
```

**On `block_id`:** The start/end range encodes compaction history. A block with `end == 0x0000` has never been compacted — it represents only itself. A block with `end != 0x0000` is the result of compaction — it now represents the range `[start, end]` of original blocks. This allows the matching engine and cleanup stage to know immediately whether a block is compacted and what it represents.

---

## 2.8 OrderEntry — The Atomic Unit

### Concept

`OrderEntry` is the smallest unit in the order book — a single resting limit order. It is stored inside an `OrderBlock`, accessed via the PriceNode's match cursor.

An `OrderEntry` is deliberately minimal. The 32-byte owner public key is **not** stored here. Instead, the entry carries an 8-byte coordinate into the Ledger — two 4-byte pointers that identify who placed this order and where their order status lives. This coordinate system reduces per-entry size from 32+ bytes to 8 bytes and is resolved during settlement, not during matching.

The entry also stores pre-computed exchange amounts rather than storing only price and quantity. This is a CU optimization — the matching engine reads these values directly at match time rather than recomputing them.

### Pre-Computed Exchange Amounts

When a trader places a limit order, the client computes:
- `quantity_amount` — the quantity of base token to be exchanged
- `quote_amount` — the equivalent in quote token (`price × quantity`)

These are validated on-chain before the order enters the book:
```
quote % price    == 0   (quote is exactly divisible by price)
quote % quantity == 0   (quote is exactly divisible by quantity)
```

If either check fails, the instruction is rejected. If both pass, the pre-computed values are stored on the entry and trusted. The matching engine never recomputes them — it finds the entry and reads directly.

This shifts computation to the client (cheap) and reduces CU cost on the hot path (expensive).

### Structure — 24 bytes

```
Field                       Size   Byte Offset   Description
quantity_amount: u64        8      0             Base token amount (pre-computed)
quote_amount: u64           8      8             Quote token amount (pre-computed, = price × quantity)
address_index_pointer: u32  4      16            Coordinate → owner's AddressIndexNode in Ledger
order_entry_status_pointer: u32  4  20           Coordinate → order's status node in Ledger
```

**The 8-byte coordinate:** `address_index_pointer` and `order_entry_status_pointer` together form the ledger coordinate for this entry. They are raw byte addresses into ledger slab accounts. They are not Solana addresses. The settlement stage uses them to look up the trader's identity and update their order status.

> **WIP:** The exact encoding and how the settlement stage resolves them to the correct slab accounts is still being designed. See Q-021 in `11-open-questions.md`.

---

## 2.9 OrderBlockTreeNode — Block Navigation Structure (Under Evaluation)

> **Note:** This structure is only implemented if Approach 3 is chosen during benchmarking. Details may change during implementation.

### Concept

`OrderBlockTreeNode` collectively batches all `OrderBlock`s at a given price level into a binary tree structure. The tree enables delta-guided traversal — each node stores the cumulative `delta_quantity_total` and `delta_quote_total` for all OrderBlocks in its subtree, allowing the matching engine to navigate directly to the right block without scanning linearly.

**Traversal:**

Two options for traversal:

1. **Delta-guided descent** — at each node, check the right subtree's delta total. Subtract it from the parent total to get the left subtree's total. If the amount being searched for is less than the right subtree total, traverse right. If more, traverse left. Repeat until depth 0.

2. **Stack-based traversal** — use a path stack to track visited nodes, similar to the TrieNode traversal approach.

**Insertion order:**

OrderBlocks are inserted left to right. The right child may not exist yet — `0x0000_0000` indicates nothing is there. It is also valid for left or right to become `0x0000_0000` during the block's lifetime.

**Leaf encoding:**

Depth-based. The depth of the tree is derivable from the total number of OrderBlocks. At depth 0 you know you are at a leaf node pointing to an OrderBlock — no special encoding needed in the fields.

**Null pointers during lifetime:**

`0x0000_0000` on left or right can occur in two situations:
- **Matching** — when an OrderBlock is fully consumed and removed, its pointer in the tree becomes `0x0000_0000`
- **Compaction** — a merge of neighboring OrderBlocks when all their entries can fit into a single OrderBlock. After a merge the now-empty block's pointer becomes `0x0000_0000`. The full design of compaction has not yet been discussed and will be designed during implementation.

### Structure — 24 bytes

```
Field                    Size   Byte Offset   Description
delta_quantity_total: u64  8    0             Subtree quantity sum
delta_quote_total: u64     8    8             Subtree quote sum
left: u32                  4    16            Left child — raw byte address of OrderBlockTreeNode
                                              or OrderBlock at depth 0
                                              0x0000_0000 = nothing here
right: u32                 4    20            Right child — raw byte address of OrderBlockTreeNode
                                              or OrderBlock at depth 0
                                              0x0000_0000 = nothing here
```

---

## 2.10 OrderBlockLinkNode — Block Link Structure (Under Evaluation)

> **Note:** This structure is only implemented if Approach 2 is chosen during benchmarking. Details may change during implementation.

### Concept

`OrderBlockLinkNode` works on the same concept as `TrieLinkNode` with the same encoding approach. It batches multiple `OrderBlock` raw byte address pointers together in a chain. Unlike the doubly linked list approach (Approach 1, eliminated), the head of each `OrderBlockLinkNode` includes `delta_quantity_total` and `delta_quote_total` for the OrderBlocks it contains — providing delta aggregation at the link node level.

The chain structure mirrors TrieLinkNode — each node holds multiple OrderBlock pointers plus a variant field that is either a pointer to the next OrderBlockLinkNode in the chain or a final OrderBlock pointer, determined by the encoding in the pointer that led to this node.

### Structure — 32 bytes

```
Field                    Size   Byte Offset   Description
delta_quantity_total: u64  8    0             Sum of quantity_amounts in blocks this node contains
delta_quote_total: u64     8    8             Sum of quote_amounts in blocks this node contains
OrderBlockPointer[0]: u32  4    16            First OrderBlock raw byte address
OrderBlockPointer[1]: u32  4    20            Second OrderBlock raw byte address
OrderBlockPointer[2]: u32  4    24            Third OrderBlock raw byte address
variant: u32               4    28            Next OrderBlockLinkNode | 4th OrderBlock pointer
                                              encoding determined by the pointer that led here
```

---

---

# Part 3 — Matching Engine

> **WIP Note:** Part 3 describes the matching engine operations, traversal methods, and the order book (destination) account structures. The source account structures — the producer/consumer transient accounts that feed into the operations — are not yet defined. The cache structure and the order-event-queue structure are also not yet defined. These will be designed as implementation progresses.

---

## 3.1 What the Matching Engine Is

The matching engine is the algorithmic core of Incognitus. It is a pure Rust library — no `solana-program` dependency. It operates on raw byte slices. It has no knowledge of:

- Solana accounts, PDAs, or the account model
- The pipeline, slot timing, or the crank
- The ledger or trader identities
- Fees, balances, or token transfers

Its only concern is: given the current state of the order book (as byte slices) and a queue of market orders, execute the sequence of operations that updates the order book and produces fill records.

Everything outside that scope is the Solana program's responsibility.

### Why a Pure Rust Library

Building the matching engine as a separate crate with no Solana dependency means:

- **Fast testing.** `cargo test` in milliseconds. No validator needed.
- **Accurate benchmarking.** `criterion` benchmarks measure algorithm cost without network or Solana overhead.
- **Clear interface.** The boundary between the matching library and the Solana program is explicit and enforced by the type system.
- **Parallel development.** The library can be built and tested while account structs are still being finalized.

The matching engine is the most complex part of the system. It deserves the most test coverage and the most careful benchmarking. Building it as a pure library gives it both.

---

## 3.2 Inputs and Outputs

### Inputs

The matching engine receives:
- A queue of market orders to process
- The raw byte slices of the relevant slab accounts (TrieNode, TrieLinkNode, PriceNode, OrderBlock slabs)
- A cache — a transient account carrying state between operations (see 3.5)
- A pre-computed list of which accounts are needed (from the Pre-Compute stage)

### Outputs

The matching engine produces:
- Fill records written to the order-event-queue — each describing one matched entry (see 3.4)
- Updated delta values propagated through the TrieNode tree and OrderBlock collection
- Cleanup flags set on exhausted nodes — gates for the traversal algorithm
- Updated cache state for downstream operations

### What It Does Not Produce

The matching engine does not:
- Update the Ledger
- Transfer tokens
- Update AccountStatus balances
- Write to settlement constructs directly

---

## 3.3 Traversal Methods

The matching engine uses three traversal methods to navigate the price trie. All three use a path stack to track visited nodes. The child traversal order is side-dependent — this is how price priority is enforced structurally:

- **Bid side:** traverse children 9 → 8 → 7 → 6 → 5 → 4 → 3 → 2 → 1 → 0 (highest price first)
- **Ask side:** traverse children 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 (lowest price first)

> **Note:** These traversal methods describe the general idea and direction. Details may change during implementation.

---

### Method 1 — Price Traversal (exact price lookup)

Used when you know the exact price you are looking for.

```
price_traversal(root, target_digit, target_depth, side):
  path_stack = []
  path_stack.push(root)

  loop:
    traverse children in side order:
      if child.digit == target_digit AND child.depth == target_depth:
        found child-node
        path_stack.push(child)
        loop:
          traverse children in side order
          stop when leaf node reached
        proceed to next child-node
```

---

### Method 2 — Delta Traverse Down (find best price by delta)

Used when you have a target quantity or quote to fill and need to find the best price level that can satisfy it. Accumulates child deltas in side order until the target is exceeded — that child is the one to descend into.

```
delta_traverse_down(root, target_delta, side):
  path_stack = []
  path_stack.push(root)
  total_delta = 0

  loop:
    traverse children in side order:
      loop:
        add child.delta to total_delta
        if target_delta < total_delta:
          found child-node
          path_stack.push(child)
          loop:
            traverse children in side order
            stop when leaf node reached
        else:
          proceed to next child-node
```

---

### Method 3 — Delta Traverse Up (resume from partial position)

Used to resume traversal from a previously visited position — popping up the path stack and accumulating delta until enough context exists to descend again.

```
delta_traverse_up(path_stack, target_delta):
  from leaf → pop pointer from stack, get node
  update total_delta

  if target_delta > total_delta:
    repeat pop pointer from stack

  if target_delta < total_delta:
    proceed with delta_traverse_down
```

---

## 3.4 Order of Operations

Operations are executed in a defined sequence. Operations sharing the same letter can run concurrently. Operations of different letters must run sequentially.

> **Note:** Operation 10 is incomplete and will be broken down into more detailed sub-operations in a future iteration.

```
(a) — batch delete PriceNodes
(a) — batch delete TrieNodes
(a) — batch update TrieNodes
(a) — batch insert OrderEntries → OrderBlockNodes

(b) — batch entry delete (OrderEntry → OrderBlockNode)

(c) — batch entry compaction (OrderBlockNode)

(d) — batch update PriceNodes

(e) — batch insert PriceNodes

(f) — batch insert TrieNodes

(g) — batch matched orders (matching process)
```

---

### Operation 1 — Batch Delete PriceNodes

**Inputs:** source, destination
**Action:** Delete PriceNodes from source → destination

---

### Operation 2 — Batch Delete TrieNodes

**Inputs:** source, destination
**Action:** Delete TrieNodes from source → destination

---

### Operation 3 — Batch Update TrieNodes

**Inputs:** source, destination
**Action:** Update TrieNodes source → destination:
- `delta_quantity_total`
- `delta_quote_total`
- `trie_link` — remove deleted children

---

### Operation 4 — Batch Insert OrderEntries → OrderBlockNodes

**Inputs:** source, destination, cache, order-event-queue

**Process:**
- Check flag → update destination stack if needed
- Insert `OrderEntry` records (grouped by price) into destination
- Create new `OrderBlockNode` records as needed
- Create new `OrderBlockLinkNode` or `OrderBlockTreeNode` as needed
- Write each entry at `next_entry_offset`, advance offset
- Find head node pointer (`OrderBlockLinkNode` | `OrderBlockTreeNode` | `OrderBlockNode`)

**Update cache** (grouped by price):
- `num_order_blocks_produced_delta`
- `next_insert_offset`
- head node pointer

**Update order-event-queue:**
- entry insert event

---

### Operation 5 — Batch Entry Delete (OrderEntry → OrderBlockNode)

**Inputs:** source, destination, cache, order-event-queue

**Process:**
- Delete `OrderEntry` records (grouped by price) from destination
- Flag entry as empty (tombstone)
- Update `OrderBlock` — increment tombstone count

**Update cache** (indexed by price):
- Add impacted `OrderBlockNode` by pointer and head node pointer (insert once)

**Update order-event-queue:**
- entry delete event

---

### Operation 6 — Batch Entry Compaction (OrderBlockNode)

**Inputs:** cache, order-event-queue, destination

**Process:**
- For each `OrderBlockNode` with 16 tombstone entries:
  - Clean up the matched `OrderBlockNode`
  - Merge `OrderBlockNode` with bigger ID into its neighbor with smaller ID
- For each `OrderBlockLinkNode` or `OrderBlockTreeNode` with no remaining child pointers:
  - Clean up
- Find new head node pointer after compaction

**Update cache** (grouped by price):
- `num_order_blocks_consumed_delta`
- `num_order_blocks_compacted_delta`
- `next_insert_offset`
- head node pointer

**Update order-event-queue:**
- entry compaction event

---

### Operation 7 — Batch Update PriceNodes

**Inputs:** cache, source, destination

**Process:**
- Update `PriceNode` from source → destination:
  - `delta_quantity_total`
  - `delta_quote_total`
- Update `PriceNode` from cache → destination:
  - `num_order_blocks_consumed`
  - `num_order_blocks_compacted`
  - `next_insert_offset`
  - `order_block_pointer`

---

### Operation 8 — Batch Insert PriceNodes

**Inputs:** source, destination, cache

**Process:**
- Check flag → update destination stack if needed
- Update source `PriceNode` from cache (match by pointer):
  - `order_block_pointer`
  - `next_insert_offset`
  - `num_order_blocks`
- Insert `PriceNode` source → destination
- Update cache: pointer → `PriceNode`, indexed by price

---

### Operation 9 — Batch Insert TrieNodes

**Inputs:** source, destination, cache

**Process:**
- Check flag → update destination stack if needed
- Update source `TrieNode` (leaf) from cache:
  - `trie_link` → `PriceNode` pointer
- Insert `TrieNode` and `TrieLink` source → destination
- Create new `TrieLinkNode` records as needed

---

### Operation 10 — Batch Matched Orders (Matching Process)

> **Note:** This operation is incomplete and will be broken down into more detailed sub-operations in a future iteration.

**Inputs:** source, destination, event-queue

**Process:**
- Check flag
- For each market order (or compatible batch) in the queue:
  - Traverse order book using traversal methods (see 3.3)
  - Consume matching entries
- Update event-queue with match results

---

## 3.5 Cache and Order-Event-Queue

### Cache

The cache is a transient account — not a slab-based structure. It uses a simpler queue data structure since it is ephemeral, produced and consumed within the pipeline cycle.

The cache is the coordination mechanism between operations. Rather than re-reading slab accounts at each step, operations write their computed state into the cache and subsequent operations read from it. This reduces CU cost — compute once, pass forward.

The cache tracks per-price state:
- `num_order_blocks_produced_delta` — blocks added this cycle
- `num_order_blocks_consumed_delta` — blocks fully matched this cycle
- `num_order_blocks_compacted_delta` — blocks merged this cycle
- `next_insert_offset` — current insertion position
- head node pointer — current head of OrderBlock collection

### Order-Event-Queue

The order-event-queue records what happened during the matching cycle for downstream consumers. It is also a transient account using a simple queue structure.

Event types recorded:
- **Entry insert** — new limit order entered the book
- **Entry delete** — limit order cancelled
- **Entry compaction** — OrderBlocks merged
- **Entry match** — market order matched against limit order entries

---

## 3.6 Market Order Processing

Market orders are processed from the MarketOrderQueue. Orders are popped from the queue and can be batched together under specific conditions:

**Batching rules — all of the following must be true:**
- Same type — all buys or all sells
- Same dimension — all quantity-based or all quote-based
- No price target — any order with a price target cannot be included in a batch

**Process:**
```
batch = []

loop over queue:
  pop next order
  if order is compatible with current batch:
    add to batch
    continue
  else:
    run matching process on current batch
    start new batch with current order

run matching process on final batch
```

When a batch is processed, the traversal methods (see 3.3) navigate the trie to find the best available price levels and consume entries from the OrderBlock collection in FIFO order.

---

## 3.7 Fill Records

A fill record is produced for each entry consumed during the matching process. It is written to the order-event-queue and contains everything needed for the settlement stage to update the ledger and balances without re-reading the order book.

**Minimum required fields (exact layout WIP — Q-004):**
- Entry coordinate — the 8-byte ledger pointer from the OrderEntry
- Quantity filled — how much base token was exchanged
- Quote filled — how much quote token was exchanged
- Price — the price at which the fill occurred
- Side — ask or bid

> **WIP:** The settlement construct that holds fill records is not yet formally defined or named. "SettlementQueue" is a placeholder name showing intent. The exact structure, including the FillRecord byte layout and how settlement identifies which slab accounts to update, is still being designed.

---

## 3.8 Delta Propagation

Every fill must propagate delta changes upward through the TrieNode tree. If any propagation is missed, the delta invariant is violated and subsequent navigation decisions will be wrong.

**After filling `fill_qty` from an entry at price level P:**

```
// OrderBlock level (if OrderBlockTreeNode approach)
  block.delta_quantity_total  -= fill_qty
  block.delta_quote_total     -= fill_quote

// PriceNode level
  price_node.delta_quantity_total -= fill_qty
  price_node.delta_quote_total    -= fill_quote

// TrieNode path (leaf to root)
  for each addr in path_stack (bottom to top):
    node = &mut data[addr]
    node.delta_quantity_total -= fill_qty
    node.delta_quote_total    -= fill_quote
```

The path stack is built during downward trie traversal — each TrieNode raw byte address is pushed as we descend. The upward propagation iterates the stack after each fill.

**The delta invariant checker** — a function that walks the entire trie and verifies all subtree sums — must be implemented and run in all test contexts. It is the primary safety net against propagation bugs.

---

## 3.9 Cleanup Flagging

During the matching process, when a node is fully exhausted — all entries consumed, `delta_quantity_total == 0` — the matching engine sets the `is_active` flag (bit 14 in TrieLink.flags) to act as a gate. This gate prevents the traversal algorithm from visiting that node again during the current matching cycle.

The cleanup stage later reads these flags to know which nodes to add to the cleanup stack and free. The gate set during matching becomes the signal for cleanup.

**Only the top-level parent node needs to be flagged.** All of its children are implicitly gated. This is a CU optimization — flagging one parent is O(1), flagging every ancestor is O(depth).

---

## 3.10 FIFO Guarantee

Within a price level, orders are matched in the order they were inserted — first in, first out. This guarantee is structural, not behavioral — it comes from the way the PriceNode's match cursor works:

- The match cursor always reads from the oldest unconsumed block
- Within a block, it reads in entry offset order (0, 1, 2, ... 15)
- The entry offset order matches insertion order — entries are inserted sequentially
- The match cursor never moves backward

As long as these properties hold, FIFO is guaranteed. The matching engine does not need to sort or compare timestamps.

---

## 3.11 Price-Time Priority

Price-time priority is the combination of:
1. **Price priority** — better prices are matched first. For asks: lowest price first. For bids: highest price first. The TrieNode traversal enforces this structurally through the side-dependent child order (see 3.3).
2. **Time priority** — within a price level, earlier orders are matched first. The PriceNode match cursor enforces this via FIFO (see 3.10).

The matching engine provides both by construction.

---

## 3.12 Partial Fills

A market order is partially filled when:
- The order book runs out of liquidity (`BookExhausted`)
- A price target is set and the next price level violates it (`PriceTargetHit`)
- The compute budget approaches its limit (`BudgetApproached`)

In all cases, the `MatchResult` returns the remaining unfilled amount and the stop reason. The pipeline handles continuation — the matching engine only reports the result. How a partially filled order continues through the pipeline is still being designed. See Q-014.

---

## 3.13 CU Budget Management

The matching instruction has a fixed compute budget. The matching engine must stop before the budget is exhausted — not because it ran out, but because it proactively checked and decided to stop early, leaving enough CU for the instruction to cleanly return.

```
CU_SAFETY_MARGIN = [TBD — establish during benchmarking]

if remaining_budget < CU_SAFETY_MARGIN:
  return MatchResult { remaining, stop: BudgetApproached }
```

All CU cost estimates are placeholders until empirically measured. The 5k CU target applies to the limit order queue and market order queue instructions, not to the matching instruction. The matching instruction CU target will be established during Layer 5 benchmarking.

---

## 3.14 Hot and Cold Account Handling

The matching engine receives a pre-computed list of slab accounts from the Pre-Compute stage (Stage 5). This list determines which accounts are available during matching.

**Hot accounts** — slab accounts containing orders at or near the current best price — are included in this list. The matching engine works through them first.

**Cold accounts** — slab accounts further from the best price — are not included in the same transaction. They are loaded separately in a different pipeline configuration with more available CU.

When a hot account is fully depleted, the next account in price proximity order becomes hot. The mechanism for detecting this and transitioning to the next account is still being designed. See Q-017.

---

## 3.15 The TestSlab Pattern

> **Note:** This is a high-level idea being explored in depth. Not finalized.

The matching engine operates on raw byte slices — `&[u8]` and `&mut [u8]`. In production these come from Solana account data. During development and testing you don't want to spin up a validator just to test whether delta propagation works correctly or whether the trie finds the right price level.

The `TestSlab<N>` pattern solves this. It is a stack-allocated byte array of size N that you pass wherever the matching engine expects a raw byte slice. The matching engine doesn't know or care whether the bytes come from a Solana account or a stack array — it just sees `&[u8]`.

```rust
pub struct TestSlab<const N: usize> {
    data: [u8; N],
}
```

In a test you initialize slab headers on the TestSlab arrays, then run actual matching engine operations against them:

```rust
let mut trie_slab  = TestSlab::<10240>::new();
let mut price_slab = TestSlab::<10240>::new();
let mut block_slab = TestSlab::<10240>::new();

slab_init::<TrieNode>(&mut trie_slab.as_mut_slice());
slab_init::<PriceNode>(&mut price_slab.as_mut_slice());
slab_init::<OrderBlock>(&mut block_slab.as_mut_slice());
```

Tests run with `cargo test` in milliseconds. No validator. No CU budget constraint — so the delta invariant checker can be run after every single operation.

Every test that touches the order book must run the delta invariant checker after every operation.

---

## 3.16 Interface

> **Note:** This is a high-level idea being explored in depth. Not finalized. These are early candidate designs only.

The interface is the Rust API boundary between the matching engine library and the Solana program that calls it. This is one of the most important architectural decisions in the system — it determines how the two layers couple to each other and enforces the separation of concerns.

**The core problem:** The matching engine needs to read and write multiple slab accounts simultaneously. In Solana, account data comes as `&mut [u8]` slices from `AccountInfo`. You need to extract the raw byte slices and pass them into the matching engine. Rust's borrow checker requires careful ownership structuring when passing multiple mutable slices to the same function.

**Candidate approach — `SlabSet`:**

```rust
pub struct SlabSet<'a> {
    pub trie_data:  &'a mut [u8],
    pub link_data:  &'a mut [u8],
    pub price_data: &'a mut [u8],
    pub block_data: &'a mut [u8],
}

pub fn match_market_order(
    slabs:     &mut SlabSet,
    order:     MarketOrderNode,
    fills_out: &mut [FillRecord; MAX_FILLS],
) -> MatchResult
```

The lifetime `'a` ties all slices to the same duration — they all come from the same instruction's account infos. No heap allocation. No return-value Box. The caller provides all memory via the `fills_out` buffer.

**Why the interface matters architecturally:**

Once agreed, two people can work on each side independently. The matching engine developer doesn't need to know anything about Pinocchio or account validation. The Solana program developer doesn't need to know anything about trie traversal. The `SlabSet` struct is the handshake between the two layers.

**What's still unresolved:**
- How the hot/cold account distinction affects the interface — if hot and cold accounts are passed separately, does `SlabSet` need variants?
- How the cache and order-event-queue plug into the interface — raw byte slices passed in, or handled by the caller?
- Whether `SlabSet` needs to include the cache and order-event-queue slices or whether those are passed separately

> **WIP:** Exact signatures will be finalized once the library structure is agreed. This is a blocking prerequisite for Layer 4 integration (Q-007).

---

## 3.17 Source Accounts, Cache, and Order-Event-Queue — Not Yet Defined

> **WIP:** This section documents what is not yet designed.

Part 3 describes the operations the matching engine performs and the destination account structures (the order book slabs — TrieNode, PriceNode, OrderBlock, etc.). What is not yet defined:

**Source accounts:**
The producer/consumer transient accounts that feed data into the operations. These are the "source" in each operation's inputs — the accounts that carry new limit orders, cancel requests, and market orders into the pipeline. Their structure, byte layout, and how they are produced and consumed across pipeline stages is not yet designed.

**Cache structure:**
Section 3.5 describes what the cache tracks conceptually (per-price deltas, block counts, offsets, head node pointers). The exact byte layout of the cache account, how entries are indexed, and how it is read and written by each operation is not yet designed.

**Order-event-queue structure:**
Section 3.5 describes what the order-event-queue records (insert, delete, compaction, match events). The exact byte layout of each event type, how the queue is structured, and how downstream consumers (settlement, indexer) read from it is not yet designed.

These will be designed during implementation as the pipeline account model is finalized. See Q-003, Q-004, and Q-012 in `11-open-questions.md`.

---

---

# Part 4 — Ledger

---

## 4.1 What the Ledger Is

The Ledger is a **live tracking system for open orders.** It is not a history. It is not an audit trail. It is not a log.

At any given moment, the Ledger contains exactly the information needed to answer: "What orders does this trader have open, and what is the current status of each?" When an order is closed — fully filled, cancelled, or expired — its entries are removed. When a trader has no open orders for a sufficient duration, their entry in the Ledger is removed.

**The Ledger's data does not persist beyond its usefulness.** Once an order is settled and the relevant balance changes are reflected in AccountStatus, the Ledger entries for that order serve no further purpose and are cleaned up.

---

## 4.2 The Ledger as a Collection

Like the order book, the Ledger is a collection of slab accounts:

```
Ledger
  ├── slab[AddressIndexNode][market][0]       ← trader anchors
  ├── slab[OrderEntryStatusNode][market][0]   ← live order status
  ├── slab[OrderEntryChangeLogNode][market][0] ← working records during settlement
  ├── slab[OrderEntryStatusLinkNode][market][0]   ← chains
  └── slab[OrderEntryChangeLogLinkNode][market][0] ← chains
```

The Ledger accounts are disjoint from the order book accounts. This is what allows Settlement and Cleanup to run concurrently — Settlement writes to Ledger accounts, Cleanup writes to order book slab accounts. They do not conflict.

---

## 4.3 The Coordinate System

The connection between the order book and the Ledger is the **coordinate system**. Each `OrderEntry` in the order book carries an 8-byte coordinate:

```
address_index_pointer: u32   → raw byte address into AddressIndexNode slab
order_entry_status_pointer: u32 → raw byte address into OrderEntryStatusNode slab
```

These two u32 raw byte addresses together uniquely identify the trader (via AddressIndexNode) and the specific order's status record (via OrderEntryStatusNode). They are 4-byte numbers, not 32-byte Solana addresses.

**Why coordinates instead of addresses:**
A 32-byte owner pubkey on every OrderEntry would add 24 bytes of overhead per entry compared to the 4+4 coordinate. At 16 entries per OrderBlock, that's 384 bytes per block — a full extra PriceNode worth of space, wasted on addresses.

The coordinate resolves to the full trader identity only during settlement, when the actual address is needed for balance updates. During matching, the coordinate is irrelevant — it's just carried along in the fill record.

---

## 4.4 AddressIndexNode — Trader Anchor

### Concept

The `AddressIndexNode` is the trader's anchor in the Ledger. It maps a slot index (the `address_index_pointer` from the coordinate) to the trader's full 32-byte owner address and to the heads of their open order lists for each side.

Each trader who has ever placed an order in this market has at most one `AddressIndexNode`. When a trader opens their first limit order, an `AddressIndexNode` is created for them. When all their orders are closed and the node has had zero open entries for a set duration, the node is removed.

The `num_open_entries` field tracks how many open orders the trader currently has. When this reaches zero, a cleanup timer starts (duration not yet decided — Q-016). If no new orders arrive before the timer expires, the node is removed and its slab slot returned to the free stack.

### Structure — 48 bytes

```
Field                       Size   Byte Offset   Description
owner_address: Pubkey       32     0             Trader's 32-byte wallet public key
ask_order_entry_pointer: u32  4    32            Head of ask order chain
bid_order_entry_pointer: u32  4    36            Head of bid order chain
num_open_entries: u16         2    40            Count of currently open orders
_padding: [u8; 6]             6    42            —
```

**Lifecycle:**
1. Created when trader places their first order (by settlement, not by matching)
2. `num_open_entries` incremented on each new order
3. `num_open_entries` decremented on each filled or cancelled order
4. When `num_open_entries == 0` for a set duration → node removed, slot freed

---

## 4.5 OrderEntryStatusNode — Live Order Status

### Concept

The `OrderEntryStatusNode` tracks the live state of a single open order. It is created when a new limit order enters the order book and removed when the order is closed (fully filled or cancelled).

This node is the Ledger's view of a specific order — it answers "what is the current state of order X?" without needing to traverse the order book. It carries enough information to locate the order in the order book (via `order_block_pointer` and `order_entry_offset`) and to describe its current state.

### Structure — 32 bytes

```
Field                    Size   Byte Offset   Description
price: u64               8      0             Order price
slot_open: u64           8      8             Slot when order was placed
slot_update: u64         8      16            Slot of last status update
order_block_pointer: u32 4      24            Current location in order book (OrderBlock)
order_entry_offset: u16  2      28            Offset within that OrderBlock
flags: u8                1      30            bit 0: side (0=bid, 1=ask)
                                              bit 1: status (0=open, 1=closed)
_padding: u8             1      31            —
```

**Lifecycle:**
1. Created by settlement when a new limit order enters the order book
2. Updated by settlement when a partial fill occurs
3. Marked closed (flags.status = 1) when fully filled or cancelled
4. Removed and slot freed once the settlement process for this order is complete

---

## 4.6 OrderEntryChangeLogNode — Working Record

### Concept

The `OrderEntryChangeLogNode` is a working record of a state change on an order. It is created during the settlement process to record what happened to a specific order during a pipeline batch — a new open, a partial fill, a full fill, or a cancel.

It is **not** a permanent log. It is not an immutable audit trail. It is a working record that exists for the duration of the settlement process and is cleaned up when no longer needed. Its purpose is to provide the settlement stage with a structured record to process, not to serve as long-term history.

### Structure — 32 bytes

```
Field                  Size   Byte Offset   Description
type_flag: u8          1      0             open / cancel / partial_fill / full_fill
                                            [TBD: may move to pointer field]
_padding: [u8; 7]      7      1             —
slot: u64              8      8             Slot of the event
quantity_delta: u64    8      16            Change in quantity
quote_delta: u64       8      24            Change in quote
```

> **WIP:** The placement of `type_flag` (inline vs in the pointer) is still being resolved.

**Lifecycle:**
1. Created by the settlement stage when processing fill records
2. Used by settlement to update OrderEntryStatusNode and AccountStatus
3. Cleaned up when the settlement process for this batch is complete

---

## 4.7 Link Nodes — Chain Structure

### OrderEntryStatusLinkNode and OrderEntryChangeLogLinkNode — 16 bytes each

Link nodes provide the chain structure for building per-trader, per-side lists of order status and changelog records. They follow the same pattern as TrieLinkNode — each node holds up to 3 pointers plus a variant field that is either a pointer to the next link node or a 4th data pointer, determined by the encoding in the pointer that led to this node.

```
AddressIndexNode.ask_order_entry_pointer
  → OrderEntryStatusLinkNode[0]
       [ptr_a, ptr_b, ptr_c, variant]
                               ↓
    OrderEntryStatusLinkNode[1] (last node)
       [ptr_d, ptr_e, ptr_f, ptr_g]
       (variant = 4th pointer on last node)
```

### Structure — 16 bytes (both types)

```
Field               Size   Byte Offset   Description
pointers[0]: u32    4      0             First status/changelog raw byte address
pointers[1]: u32    4      4             Second status/changelog raw byte address
pointers[2]: u32    4      8             Third status/changelog raw byte address
variant: u32        4      12            Next link node | 4th pointer
                                         encoding determined by the pointer that led here
```

---

## 4.8 Ledger Lifecycle — When Things Are Written and Removed

The matching engine **never writes to the Ledger.** It does not know the Ledger exists. The coordinate in the fill record is opaque to the matching engine — it carries it along to give to the settlement process, which knows how to resolve it.

The Ledger is written by settlement and cleaned up by cleanup. The sequence is driven by order events, not pipeline stages:

**When a new limit order enters the order book:**
- Find or create `AddressIndexNode` for the trader
- Increment `num_open_entries`
- Create `OrderEntryStatusNode` with status=open
- Link into the trader's order chain

**When a limit order is filled:**
- Resolve coordinate → `AddressIndexNode` + `OrderEntryStatusNode`
- Update `OrderEntryStatusNode` (partial: reduce amounts, full: status=closed)
- Create `OrderEntryChangeLogNode` (working record)
- Update `AccountStatus` balances (credit received, debit sent)
- Decrement `num_open_entries` if fully filled

**When an order is cancelled:**
- Update `OrderEntryStatusNode` (status=closed)
- Create `OrderEntryChangeLogNode` (type=cancel)
- Decrement `num_open_entries`
- Return `OrderEntry` slot to slab (done by cleanup, not settlement)

**When `num_open_entries` reaches zero for set duration:**
- Remove `AddressIndexNode` and free slot
- Free associated link node chains

> **Note:** How exactly fill records are handed off to the settlement process — through what construct, in what format — is still being designed. See Appendix A.1.

---

## 4.9 Ledger Capacity

| Node | Size | Max Nodes per 10MB Account |
|---|---|---|
| `AddressIndexNode` | 48 bytes | 218,450 |
| `OrderEntryStatusNode` | 32 bytes | 327,670 |
| `OrderEntryChangeLogNode` | 32 bytes | 327,670 |
| `OrderEntryStatusLinkNode` | 16 bytes | 655,350 |
| `OrderEntryChangeLogLinkNode` | 16 bytes | 655,350 |

---

---

# Appendix — Open Integration Questions

This appendix documents areas that directly connect the four systems described in this document but are not yet fully designed. These are deferred rather than forgotten.

---

## A.1 Matching Engine ↔ Settlement Handoff

**Status: Deferred — revisit when settlement design begins**

**The question:** How does the matching engine pass fill records to the settlement process?

The matching engine produces fill records during the matching operation. The settlement process needs those records to update the Ledger and balances. Between them is some construct — currently called "SettlementQueue" as a placeholder, but not yet formally defined.

**What's known:**
- The settlement construct holds fill records produced by matching and consumed by settlement
- Settlement and Cleanup run concurrently — Settlement writes to Ledger accounts and Cleanup writes to order book accounts (disjoint)
- The FillRecord must contain enough information for Settlement to act without re-reading the order book — the pre-computed exchange amounts and the ledger coordinate cover this
- The construct is transient — it is produced per-pipeline-batch and consumed entirely by settlement and cleanup

**What's not known:**
- The formal name and structure of the construct
- The exact FillRecord byte layout (Q-004)
- How the matching engine writes to it (does it write to an account directly, or does the Solana program collect results and write them after the matching function returns?)
- Whether settlement and cleanup both read from the same construct simultaneously or whether it's split

**Why this is deferred:**
This depends on the settlement design, which depends on the pipeline account design, which depends on Q-003 (slot offsets) and Q-013 (concurrency confirmation). These are upstream. Once those are resolved, this can be designed.

**Where it's tracked:** Q-004, Q-013 in `11-open-questions.md`.

---

## A.2 Ledger ↔ Order Book Coordinate Resolution

**Status: Deferred — revisit when settlement design begins**

**The question:** Given an 8-byte coordinate from an OrderEntry, how does the settlement process determine which specific ledger slab accounts to load?

The coordinate is two u32 raw byte addresses. A raw byte address alone does not identify which slab account it lives in. In a market with multiple ledger slab accounts, the settlement process needs to know: which `AddressIndexNode` slab does `address_index_pointer` live in?

**What's known:**
- For MVP, there is one slab per node type per market — so the account is unambiguous
- Post-MVP, when multiple slabs exist, the encoding must somehow identify the account
- The slab account index is in the PDA seeds, so accounts are enumerable
- A simple approach: the settlement process loads all ledger slab accounts for the market and checks capacity ranges to determine which account a given address lives in

**Why this is deferred:**
For MVP this is a non-issue. The design for multi-account ledger resolution can wait until multi-account slabs are designed (post-MVP). For now, assume single-account ledger slabs.

**Where it's tracked:** Q-021 in `11-open-questions.md`.

---

## A.3 Slab Allocator ↔ Layer Above Integration

**Status: Partially designed — details emerge during implementation**

**The question:** How does the layer above the slab detect that a slab is nearly full and trigger account growth?

**What's known:**
- The slab header contains a flag that signals resize is needed
- The layer above detects this flag and a separate resize instruction handles the account growth
- The slab responds during the resize instruction by adjusting the stack structure for the new space
- The slab does not trigger or manage allocation — it only responds when the resize instruction runs

**What's not fully designed:**
- The exact mechanism for detecting the flag (crank observes it? instruction validates pre-flight?)
- The timing — does the resize happen before or after the instruction that triggered the full condition?
- The in-flight state during resize — can orders be placed while a slab is being resized?

**Where it's tracked:** Q-008 (multi-account expansion) in `11-open-questions.md` — marked post-MVP.
