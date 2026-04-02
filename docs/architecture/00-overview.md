# Incognitus — High-Level Architecture Overview

> **Project:** Incognitus — On-Chain Central Limit Order Book (CLOB)
> **Chain:** Solana
> **Hackathon:** Colosseum Frontier (April 6 – May 11, 2026)
> **Status:** Pre-build / Architecture Phase
> **Version:** 2.2
> **Changelog:**
> - v0.1 — Initial draft
> - v0.2 — Corrected project name, updated slab allocator scope, OrderBlock/OrderTreeNode,
>           matching algorithm, write-lock timeout, indexer out-of-scope, Appendix A
> - v0.3 — Section 6 review pass: slab allocator, write-lock, pipeline slot batching,
>           hot/cold accounts all corrected
> - v0.4 — Section 7 rewritten as WIP, slab account identification added, Section 17 added
> - v0.5 — Section 8 updated: TrieNode, TrieLink, TrieLinkNode, PriceNode corrected
> - v0.6 — Section 11 rewritten: ledger as live tracking system, cleanup behavior added
> - v0.7 — Section 14 updated: raw format, u128 clarified, validation rules non-exhaustive,
>           post-MVP trading down to unit with drift prevention
> - v0.8 — Section 15 updated: indexer post-MVP, WASM client rewritten as WIP
> - v0.9 — Section 16 updated: separated fees from governance, compaction clarified,
>           added missing out of scope items
> - v1.0 — Section 12 updated: marked WIP, settlement goal clarified, Internal label removed
> - v1.1 — Section 13 marked WIP, noted to be finalized after matching engine is complete
> - v1.2 — Section 10 marked WIP throughout, flow noted as intent, next_match_offset
>           corrected, SettlementQueue noted as intent/unnamed, parent node flagging clarified
> - v1.3 — Section 9 marked WIP with detailed notes on all unfinalized aspects
> - v1.4 — Appendix B updated: corrected entries, added new glossary terms
> - v1.5 — Section 18 added: Priority and Order of Focus with build layers, concurrent
>           client/frontend tracks, key priorities per layer. Table of Contents updated.
> - v1.6 — Sections 2 and 3 updated: problem framing note added, slab language corrected,
>           matching engine expanded to all data structures, pre-computed exchange amounts
>           explained, solution framing note added
> - v1.7 — Section 1 updated: note added on v1 scope and performance claims as theoretical
> - v1.8 — Appendix A updated: Q-011 resolved and moved to 12-resolved-questions.md (R-021)
> - v1.9 — Q-016 through Q-021 added to 11-open-questions.md, R-022 and R-023 added to
>           12-resolved-questions.md
> - v2.0 — Section 19 added (Matching Engine as standalone concept), Appendix C added
> - v2.1 — Section 5 fully rewritten: system layers, boundaries, connections, pipeline flow
>           (WIP noted), component summary grouped by layer, Crank v2 moved to P2P layer
>           (Constraints, Error Handling Philosophy, Security Considerations, Order Lifecycle),
>           companion docs table fixed, Table of Contents updated
>           12-resolved-questions.md

---

## Table of Contents

1. [Project Summary](#1-project-summary)
2. [The Problem](#2-the-problem)
3. [The Solution](#3-the-solution)
4. [Performance Targets](#4-performance-targets)
5. [System Architecture Overview](#5-system-architecture-overview)
6. [Core Concepts](#6-core-concepts)
7. [Account Model](#7-account-model)
8. [Data Structures](#8-data-structures)
9. [The Concurrent Pipeline](#9-the-concurrent-pipeline)
10. [Matching Algorithm](#10-matching-algorithm)
11. [Ledger System](#11-ledger-system)
12. [Settlement](#12-settlement)
13. [Instruction Set](#13-instruction-set)
14. [Key Formulas & Math](#14-key-formulas--math)
15. [Technology Stack](#15-technology-stack)
16. [Out of Scope (v1)](#16-out-of-scope-v1)
17. [Slab Allocator](#17-slab-allocator)
18. [Priority & Order of Focus](#18-priority--order-of-focus)
19. [Matching Engine](#19-matching-engine)

---

## 1. Project Summary

Incognitus is a fully on-chain Central Limit Order Book (CLOB) built on Solana. It is designed as a foundational primitive for spot trading, perpetuals, options, and derivatives — the matching engine layer that other financial products are built on top of.

Existing on-chain CLOBs on Solana are constrained by two hard limits: account size caps the depth of the order book, and shared account write contention causes transaction failures under load. Incognitus rejects both constraints through a novel combination of a slab allocator-managed boundless order book, a three-dimensional TrieNode matching engine, a slot-based concurrent transaction pipeline, and a decentralized crank system.

The result is an order book that can hold millions of resting orders, match against hundreds of thousands per transaction, and operate at Solana's theoretical throughput ceiling.

> **Note:** v1 focuses on spot trading as the foundational layer. Perpetuals, options, and derivatives are future phases built on top of this primitive. Performance claims are theoretical maximums — benchmarks will be published as the protocol matures.

---

## 2. The Problem

### 2.1 Bounded Order Books

Every major on-chain CLOB on Solana — Serum, OpenBook, Phoenix — stores the order book inside a single account. Solana accounts have a hard size limit. When the order book fills, orders are evicted. This is not a Solana limitation — it is an architectural choice. But it means no existing on-chain CLOB can support deep, liquid markets at scale.

### 2.2 Write Contention and Transaction Failures

Placing a limit order, matching, and settling all write to the same accounts. Under load, multiple transactions contend on the same account in the same slot. Solana's runtime serializes conflicting writes — the transactions that lose the race fail and must be retried. Failure rates spike exactly when throughput is highest, which is the opposite of what a trading venue needs.

### 2.3 The Crankless Throughput Tradeoff

Crankless architectures (where users self-settle) are the standard approach to censorship resistance. But crankless designs impose a structural ceiling on how many orders can be processed per transaction, because the full order lifecycle — place, match, settle — must fit within a single transaction's compute budget. Nobody has solved this tradeoff. Incognitus takes a different path.

> **Note:** The framing of the problem will be improved in a future iteration.

---

## 3. The Solution

### 3.1 Boundless Order Book

Order book data is managed by a **slab allocator** — a memory management system built on a raw byte array. The slab allocator operates within a single Solana account, managing one node type per account. Multiple slab accounts — one per node type per side per market — compose the full order book structure.

The design intention is for the order book to scale across multiple accounts of the same node type when a single account's capacity is exceeded, removing the depth ceiling entirely. For MVP, each slab operates within a single account. Multi-account expansion is a higher-level concern deferred post-MVP.

### 3.2 Novel Matching Engine

The matching algorithm is built on a collection of custom data structures that together compose the order book:

- **TrieNode** — organizes orders by price using a decimal digit trie. Each node stores `quantity_delta` and `quote_delta` enabling three-dimensional traversal: by price, by quantity delta, or by quote delta.
- **TrieLink** — embedded in each TrieNode, encodes what the node points to and carries flags for traversal and cleanup.
- **TrieLinkNode** — a support node that stores child TrieNode pointers, allowing a TrieNode to have up to 10 children without bloating the TrieNode itself.
- **PriceNode** — maps a specific price level to its collection of order entries. Tracks insertion and match positions across OrderBlocks.
- **OrderBlock** — batches order entries into groups. Each entry stores pre-computed exchange amounts — the quantity to send and the quantity to receive at the given price. These values are computed off-chain and validated on-chain, reducing CU cost during matching. The matching engine finds the last entry to match against rather than recomputing exchange amounts at match time.
- **OrderBlockTreeNode** — a candidate structure for organizing OrderBlocks within a price level with delta aggregation. Under evaluation alongside OrderBlockLinkNode.

Settlement is fully decoupled from matching. The engine identifies and flags matched orders without processing transfers — settlement happens in a separate delayed, non-blocking pipeline stage.

### 3.3 Concurrent Transaction Pipeline

Rather than processing the full order lifecycle in a single transaction, operations are divided into distinct pipeline stages, each mapped to a slot boundary. Because each stage touches a disjoint set of accounts, Solana's Sealevel runtime executes all active stages in parallel across slots. The result is a pipeline where multiple batches are always in flight simultaneously — the effective throughput is the pipeline throughput, not single-transaction throughput.

### 3.4 Decentralized Cranking

Incognitus is a cranked system, but cranking does not require centralization. v1 implements a client-side decision tree that distributes cranking responsibility to clients. v2 will expand this into a peer-to-peer incentive network where crankers compete for fees.

> **Note:** The framing of the solution will be improved in a future iteration.

---

## 4. Performance Targets

| Metric | Target | Condition |
|---|---|---|
| Limit orders per second | 200,000 | Solana Alpenglow upgrade |
| Matched orders per transaction | 400,000 | Single matching transaction |
| Compute units per limit order | 5,000 CU | On-chain verification only |
| Order book depth | Unbounded | Slab allocator across N accounts |

> These are theoretical maximums. Benchmarks will be published as the protocol matures.

---

## 5. System Architecture Overview

A high-level map of the system — the major components, where they live, and how they connect. Detail on each component is covered in dedicated sections.

---

### 5.1 System Layers

The system is organized into four layers, each building on the one below it:

```
┌─────────────────────────────────────────────────────────┐
│  Interface Layer                                        │
│  Solana Program — validations, instructions, pipeline   │
│  coordination, CU management                           │
├─────────────────────────────────────────────────────────┤
│  Core Layer                                             │
│  Matching Engine — traversal, matching, delta           │
│  propagation, fill record production                    │
├─────────────────────────────────────────────────────────┤
│  Data Layer                                             │
│  Order Book — TrieNode, PriceNode, OrderBlock           │
│  Ledger — AddressIndexNode, OrderEntryStatusNode        │
├─────────────────────────────────────────────────────────┤
│  Memory Layer                                           │
│  Slab Allocator — raw byte buffer, node storage,        │
│  insert, delete, allocation event response             │
└─────────────────────────────────────────────────────────┘
```

---

### 5.2 System Boundaries

```
ON-CHAIN
┌──────────────────────────────────────────────────────────┐
│  Solana Program                                          │
│  ┌─────────────────┐      ┌───────────────────────────┐ │
│  │  Order Book     │      │  Ledger                   │ │
│  │  (slab accounts)│      │  (slab accounts)          │ │
│  └─────────────────┘      └───────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐ │
│  │  Transient Accounts                                 │ │
│  │  (queues, cache, settlement)                        │ │
│  └─────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘

OFF-CHAIN CLIENT
┌──────────────────────────────────────────────────────────┐
│  ┌─────────────────┐      ┌───────────────────────────┐ │
│  │  Frontend UI    │      │  WASM / Client Logic      │ │
│  │  (React/Vite)   │◄────►│  Crank, Pipeline Mgmt,   │ │
│  │                 │      │  Instruction Builders     │ │
│  └─────────────────┘      └───────────────────────────┘ │
└──────────────────────────────────────────────────────────┘

P2P LAYER (future)
┌──────────────────────────────────────────────────────────┐
│  Crank (v2) — decentralized cranking, economic          │
│  incentives, p2p coordination                           │
└──────────────────────────────────────────────────────────┘

OFF-CHAIN SERVICES (post-MVP)
┌──────────────────────────────────────────────────────────┐
│  Indexer — log parsing, order book aggregation,          │
│  REST API, WebSocket feed                               │
└──────────────────────────────────────────────────────────┘
```

---

### 5.3 How They Connect

```
User
  │ signs transaction
  ▼
Frontend / WASM Client
  │ builds and submits instructions via RPC
  ▼
Solana Program (on-chain)
  │ validates, routes, manages pipeline
  │ reads/writes
  ▼
Slab Accounts (Order Book + Ledger)
  │ raw byte data
  ▼
Matching Engine (pure logic layer)
  operates on raw byte buffers passed in from the program

Crank (v1 — client-side)
  │ monitors on-chain state
  │ advances pipeline stages via RPC
  ▼
Solana Program

Crank (v2 — p2p, future)
  │ p2p coordination layer
  │ economic incentives for cranking
  ▼
Solana Program

Indexer (post-MVP)
  │ subscribes to program logs
  │ serves data via REST + WebSocket
  ▼
Frontend
```

---

### 5.4 Pipeline Flow

> **WIP** — The diagram below shows the intent and direction of the pipeline flow. It is partially wrong and will be iterated on as the pipeline design is finalized. See Section 9 for the full pipeline detail.

```
                        ┌─────────────────────────────────────────────────────────┐
                        │                    SOLANA PROGRAM                       │
                        │                                                         │
  User / Trader ───────►│  Limit Order Queue     Market Order Queue               │
                        │         │                      │                        │
                        │         ▼                      ▼                        │
                        │  Limit Order         Market Order                       │
                        │  Aggregation         Pre-Compute / Cache                │
                        │         │                      │                        │
                        │         ▼                      ▼                        │
                        │     Order Book Update     Match Orders                  │
                        │    (Write-Lock ON)              │                       │
                        │         │              Cleanup + Write-Lock OFF         │
                        │         └──────────────────────┘                       │
                        │                      │                                  │
                        │                      ▼                                  │
                        │                 Settlement                              │
                        │           (Ledger Update)                               │
                        └─────────────────────────────────────────────────────────┘
                                              │
                               ┌──────────────┴────────────────┐
                               ▼                               ▼
                        ┌─────────────┐                ┌──────────────┐
                        │   Indexer   │                │   Frontend   │
                        │  (Rust/Go)  │                │  (React/WASM)│
                        │  Postgres   │                │  TradingView │
                        │  WebSocket  │                │  Wallet      │
                        └─────────────┘                └──────────────┘
                               │
                        ┌──────▼──────┐
                        │    Crank    │
                        │  Decision   │
                        │    Tree     │
                        │  (Client)   │
                        └─────────────┘
```

---

### 5.5 Component Summary

**Memory Layer**

| Component | Role | Location |
|---|---|---|
| Slab Allocator | Raw byte buffer management. Handles node storage, insert, delete, and responds to allocation events. Foundation for all data structures. | On-chain accounts |

**Data Layer**

| Component | Role | Location |
|---|---|---|
| Order Book | A construct and collection of slab accounts. Organizes and tracks all resting limit orders via TrieNode, PriceNode, and OrderBlock structures. | On-chain slab accounts |
| Ledger | A construct and collection of slab accounts. Tracks open orders and their relationship to traders. Entries are removed when orders are closed. | On-chain slab accounts |

**Core Layer**

| Component | Role | Location |
|---|---|---|
| Matching Engine | The algorithmic core. Traverses the order book, matches market orders against limit orders, propagates deltas, and produces fill records. Pure logic — no knowledge of accounts or pipeline. | On-chain program (pure Rust library) |

**Interface Layer**

| Component | Role | Location |
|---|---|---|
| Solana Program | The interface to the system. Handles instruction validation, account management, pipeline coordination, and CU management. Wraps the matching engine. Built on Pinocchio. | On-chain (Pinocchio) |
| Concurrent Pipeline | Slot-based staged transaction processing. Coordinates the order lifecycle across pipeline stages using the sliding window technique. | On-chain coordination |
| Transient Accounts | Slot-tagged pipeline state — queues, cache, settlement construct. Produced and consumed within each pipeline cycle. | On-chain accounts |

**Client**

| Component | Role | Location |
|---|---|---|
| WASM / Client Logic | Instruction builders, account decoders, crank decision tree, pipeline account management, sliding window coordination. | Browser / client |
| Frontend UI | Trading interface — order placement, order book display, trade history, TradingView chart. | Browser (React/Vite) |
| Crank (v1) | Client-side decision tree to advance pipeline stages. Permissionless — any client can crank. | Off-chain client |

**P2P Layer (future)**

| Component | Role | Location |
|---|---|---|
| Crank (v2) | P2P incentive network for decentralized cranking with economic incentives. Replaces v1 client-side crank. | Off-chain p2p network |

**Off-Chain Services (post-MVP)**

| Component | Role | Location |
|---|---|---|
| Indexer / Backend | Aggregates events, serves order book data and trade history via REST and WebSocket. | Off-chain server |

---

## 6. Core Concepts

### 6.1 Slab Allocator

A slab allocator is a memory management system built on a raw byte array. It manages fixed-size nodes within a single Solana account using raw byte addresses — u32 values that are literal byte offsets into the account data buffer.

When a node is inserted, the slab writes it at a raw byte address and that address is what gets stored as a pointer. When a node is deleted, its raw byte address is pushed onto a free stack so it can be reused. No slot indices. No multiplications. Just a byte address and the account data buffer.

**Regions of a SlabAllocator account:**

```
┌──────────────────────────────────────────────┐  ← start of account
│  Header  (24 bytes)                          │
│  discriminator, root_node_pointer,           │
│  stack_pointer, allocator_size,              │
│  num_node_elements, flags                    │
├──────────────────────────────────────────────┤  ← aligned to node_size
│  Node Section                                │
│  nodes written sequentially                  │
│  each accessed by its raw byte address       │
│  grows ↓                                     │
│                                              │
│  (free space)                                │
│                                              │
│  Stack Section                               │
│  u32 raw byte addresses of deleted nodes     │
│  grows ↑ toward node section                 │
├──────────────────────────────────────────────┤
│  Stack-Node  (16 bytes)                      │
│  HeadNode — always at end of account         │
└──────────────────────────────────────────────┘  ← end of account
```

Accounts start at 10kb and grow in 10kb increments up to 10mb. Growth is handled by a separate resize instruction. For the full design see Section 17.

### 6.2 Write-Lock

A write-lock is both a lock and a signal on the order book's current write state. When active, it indicates the order book is currently being written to.

The write-lock serves two purposes. First, it is a coordination signal — the crank decision tree reads this state to avoid submitting conflicting write transactions when the order book is already being written to. Second, it keeps the pipeline in sync by providing a shared state that all participants can observe.

The write-lock does not enforce anything at the network level — anyone can submit a transaction regardless of its state, but doing so will result in failed transactions due to write contention on shared accounts. Its value is in coordination: in conjunction with the crank and decision tree, it minimizes unnecessary transaction failures and keeps the system operating cleanly.

The exact mechanism and where this state is stored is still being decided. See `11-open-questions.md`.

### 6.3 Ledger Coordinate System

Rather than storing a trader's owner address (32 bytes) on each `OrderEntry`, Incognitus uses an 8-byte dual-pointer coordinate system: an `address_index_pointer` and an `order_entry_status_pointer`. These coordinates point into the Ledger — a separate account collection that stores owner addresses and order status. This reduces per-entry size from 32+ bytes to 8 bytes, significantly increasing order book density.

### 6.4 Pipeline Slot Batching

Each pipeline stage maintains a pool of accounts keyed to slots using a sliding window technique — borrowed from UDP networking protocols — to ensure data is never lost due to slot timing uncertainty.

Each account in the pool maps to a slot via `slot % N` (where N is the pool size, e.g. 100). Because a transaction cannot know in advance which slot it will be included in, each transaction includes a window of 4 slot-aligned accounts covering adjacent slots. Wherever the transaction lands, the correct account is in sync with that slot.

Multiple pipeline batches are always in-flight simultaneously across different slots. Each stage touches a different set of accounts, so Solana's Sealevel runtime executes all active stages in parallel.

### 6.5 Hot and Cold Accounts

Order book accounts are classified as hot or cold based on their proximity to the current best price.

**Hot accounts** contain orders at or near the best price and are actively loaded during the matching process. For example, if the best ask is $10, the accounts holding orders from $10 down through some depth are hot — these are the accounts the matching engine works through first.

**Cold accounts** contain orders further from the best price. They are not loaded during active matching. They are processed through a separate, delayed pipeline that runs slower but has more CU resources available since it is not competing with the hot path.

As hot accounts are depleted, the next cold account in line becomes hot and enters the active matching pipeline.

---

## 7. Account Model

> **Status: WIP** — The account model is not yet fully defined. The current priority is the matching engine. Accounts will be defined and iterated on as the matching engine is built out. The SlabAllocator is the one exception — it is well understood and documented in Section 17. All other accounts are stubs and will be filled in as the design matures.

---

### 7.1 Account Overview

| Account | Status | Notes |
|---|---|---|
| `MarketConfig` | WIP | Root market account — fields and structure to be defined |
| `AccountStatus` | WIP | Per-user market account — fields and structure to be defined |
| `SlabAllocator` | Defined | See Section 17 |
| `LimitOrderQueue` | WIP | Transient — to be defined |
| `MarketOrderQueue` | WIP | Transient — to be defined |
| `MarketOrderCache` | WIP | Transient — to be defined |
| `SettlementQueue` | WIP | Transient — to be defined |
| `AccountLedgerAddresses` | WIP | Ledger — to be defined |
| `AccountLedgerOrders` | WIP | Ledger — to be defined |

---

### 7.2 SlabAllocator Account

The SlabAllocator account stores a single node type for one side of one market. Multiple slab accounts of the same node type can exist — this is what makes the order book boundless. A higher-level layer tracks and coordinates all slab accounts of the same node type. The slab itself has no knowledge of other slab accounts.

Each slab account is identified by node type, side, market, and an account index that increments as new accounts are allocated. The exact PDA seed composition is still being iterated on — the intent is:

```
slab[TrieNode][bid][market][0]    ← first TrieNode bid slab
slab[TrieNode][bid][market][1]    ← second (when first fills)
slab[TrieNode][ask][market][0]
slab[PriceNode][bid][market][0]
slab[OrderBlock][bid][market][0]
... etc
```

For the full SlabAllocator design including layout, operations, and stack structure — see Section 17.

---

### 7.3 Other Accounts

All remaining accounts are WIP and will be defined as the matching engine is built. They will be documented here as they are finalized.

---

## 8. Data Structures

> **Note:** These data structures will be iterated on as details emerge during implementation. Field layouts, sizes, and flags reflect the current design intent and will be updated as the matching engine is built out.

### 8.1 Order Book Node Types

All order book nodes live inside `SlabAllocator` accounts. They are accessed by raw byte address — a u32 literal byte offset into the account data buffer.

#### `TrieNode` — 32 bytes

The primary tree structure for the matching engine. Each node represents one decimal digit at one depth in the price trie. Traversing the trie from root to leaf finds a `PriceNode`.

```
offset   size   field
0        8      delta_quantity_total: u64
64       8      delta_quote_total: u64
128      4      value: (u16, u16) — (decimal digit, depth)
160      12     trie_link: TrieLink
```

**Three traversal dimensions:**
1. By price digit → find a specific price level
2. By `delta_quantity_total` → find best price that can fill a target quantity
3. By `delta_quote_total` → find best price that can fill a target quote amount

#### `TrieLink` — 12 bytes

Embedded inside each `TrieNode`. Encodes what the node points to and carries flags for traversal and cleanup.

```
offset   size   field
0        2      flags (u16 bitmask)
                  bits 0–9:  decimal digit child bitmask (which digits exist as children)
                  bit  10:   is_root (under consideration — may be redundant)
                  bit  11:   pointer type — is TrieLinkNodePointer (only one of 11–13 active)
                  bit  12:   pointer type — is TrieNodePointer
                  bit  13:   pointer type — is PriceNodePointer
                  bit  14:   is_active — cleanup flag; when unset on a parent, all
                             children are implicitly inactive without traversal
                  bit  15:   reserved
16       1      padding
24       1      size: u8 (max 10 — number of children)
32       4      parent_trie_node: raw byte address of parent TrieNode
64       4      pointer: raw byte address — TrieLinkNode | TrieNode | PriceNode
```

Only one of bits 11–13 can be active at a time. If none or more than one are set, the state is invalid.

#### `TrieLinkNode` — 16 bytes

A support node that stores up to 3 child TrieNode raw byte addresses. The 4th field is a variant — either the next TrieLinkNode or a 4th TrieNodePointer — determined by the encoding in the pointer that led to this node. A chain of three TrieLinkNodes supports up to 10 children total.

```
offset   size   field
0        4      TrieNodePointer [0]
32       4      TrieNodePointer [1]
64       4      TrieNodePointer [2]
96       4      variant: TrieNodePointer | TrieLinkNodePointer
```

#### `PriceNode` — 32 bytes

Maps a specific price level to its collection of OrderBlocks.

| Field | Description |
|---|---|
| `last_slot_update` | Slot when this node was last modified |
| `price` | The price value this node represents |
| `parent_pointer` | Raw byte address of parent TrieNode (not needed if path stack approach chosen) |
| `order_block_pointer` | Raw byte address of OrderBlock collection — single OrderBlock, OrderBlockLinkNode, or OrderBlockTreeNode |
| `next_entry_offset` | 1 byte (0–15). Offset within last block for next insert. Resets to 0 and creates new block after 16. |
| `next_match_offset` | 1 byte (0–15). Offset within first block for next match. Resets to 0 and advances block after 16. |
| `num_order_blocks_produced` | NOBP — total blocks ever created at this price |
| `num_order_blocks_consumed` | NOBC1 — blocks fully matched and removed |
| `num_order_blocks_compacted` | NOBC2 — blocks merged via compaction |

`num_order_blocks_live` is derived: `NOBP - (NOBC1 + NOBC2)` — never stored.

#### `OrderBlock` — 408 bytes

Batches 16 `OrderEntry` records. Tracks cumulative deltas. The block_id encodes compaction history.

| Field | Description |
|---|---|
| `block_id_start: u16` | This block's ID (or start of compacted range) |
| `block_id_end: u16` | 0x0000 = no compaction. 0x_xxxx = end of compacted range. If end != 0x0000 then start < end. |
| `_padding` | 4 bytes |
| `delta_quantity_total` | Sum of all entry quantities in this block |
| `delta_quote_total` | Sum of all entry quotes in this block |
| `entries: [OrderEntry; 16]` | 16 × 24 bytes = 384 bytes |

> **Note:** Approaches 2 (OrderBlockLinkNode) and 3 (OrderBlockTreeNode) are under evaluation for navigating multiple blocks. Approach 1 (doubly linked list) is eliminated. The OrderBlock itself does not get additional pointer fields — traversal structures live in separate slab nodes. See `11-open-questions.md` Q-002.

#### `OrderEntry` — 24 bytes

The atomic unit of a resting limit order. Owner identity is stored in the Ledger via an 8-byte coordinate — two 4-byte raw byte addresses — not inline.

| Field | Description |
|---|---|
| `quantity_amount` | Resting quantity (pre-computed) |
| `quote_amount` | Resting quote (pre-computed, = price × quantity) |
| `address_index_pointer` | 4-byte raw byte address → owner in Ledger |
| `order_entry_status_pointer` | 4-byte raw byte address → order status in Ledger |

#### `OrderBlockTreeNode` — 24 bytes

A candidate structure (Approach 3) for organizing OrderBlocks within a PriceNode using a binary tree with delta aggregation. Only implemented if Approach 3 is chosen during benchmarking.

| Field | Description |
|---|---|
| `delta_quantity_total` | Subtree quantity sum |
| `delta_quote_total` | Subtree quote sum |
| `left` | Raw byte address — left child OrderBlockTreeNode or OrderBlock at depth 0. 0x0000_0000 = nothing. |
| `right` | Raw byte address — right child OrderBlockTreeNode or OrderBlock at depth 0. 0x0000_0000 = nothing. |

#### `OrderBlockLinkNode` — 32 bytes

A candidate structure (Approach 2) for organizing OrderBlocks within a PriceNode using a link chain with delta aggregation. Only implemented if Approach 2 is chosen during benchmarking.

| Field | Description |
|---|---|
| `delta_quantity_total` | Sum of quantity in blocks this node contains |
| `delta_quote_total` | Sum of quote in blocks this node contains |
| `OrderBlockPointer[0]` | First OrderBlock raw byte address |
| `OrderBlockPointer[1]` | Second OrderBlock raw byte address |
| `OrderBlockPointer[2]` | Third OrderBlock raw byte address |
| `variant` | Next OrderBlockLinkNode \| 4th OrderBlock pointer |

### 8.2 Node Sizes and Slab Capacity

| Node Type | Size | Max Nodes per 10MB Account |
|---|---|---|
| `TrieNode` | 32 bytes | 327,670 |
| `TrieLinkNode` | 16 bytes | 655,350 |
| `PriceNode` | 32 bytes | 327,670 |
| `OrderBlock` | 408 bytes | ~25,700 |
| `OrderEntry` | 24 bytes (inside OrderBlock) | — |
| `OrderBlockTreeNode` | 24 bytes | 436,900 |
| `OrderBlockLinkNode` | 32 bytes | 327,670 |

### 8.3 Pointer Types

All slab pointers are 4-byte raw byte addresses — literal byte offsets into a `SlabAllocator` account's data buffer.

| Pointer | Size | Points To |
|---|---|---|
| `TrieNodePointer` | 4 bytes | `TrieNode` raw byte address |
| `TrieLinkNodePointer` | 4 bytes | `TrieLinkNode` raw byte address |
| `PriceNodePointer` | 4 bytes | `PriceNode` raw byte address |
| `OrderBlockPointer` | 4 bytes | `OrderBlock` raw byte address |
| `OrderBlockLinkNodePointer` | 4 bytes | `OrderBlockLinkNode` raw byte address |
| `OrderBlockTreeNodePointer` | 4 bytes | `OrderBlockTreeNode` raw byte address |

---

## 9. The Concurrent Pipeline

> **Status: WIP** — The pipeline is actively being designed. The stage breakdown below demonstrates the idea, intention, and direction we are working toward. The details are currently disjointed and not yet fully accurate or finalized. Specifically:
> - Slot offsets (n through n+6) are not yet confirmed
> - Account lists per stage are not finalized
> - Process steps are directional, not authoritative
> - `SettlementQueue` is a placeholder name showing intent — not yet defined or named
> - `OrderBookChangeLog` is a placeholder name — not yet defined
> - Write-lock location (currently shown on `MarketConfig`) is not yet decided
> - Stage 7 (Cleanup) and Stage 8 (Settlement) slot ordering may appear inverted — Settlement at n+5 runs before Cleanup at n+6 intentionally, as they operate on disjoint account sets concurrently
>
> Everything here will be iterated on as the matching engine is built and the pipeline design matures.

### 9.1 Design Principle

Each pipeline stage maps to a slot offset. Each stage reads/writes a disjoint set of accounts. Solana's Sealevel runtime detects no account conflicts between stages and executes them all in parallel within the same slot.

```
              Slot N    N+1      N+2      N+3      N+4      N+5      N+6
Batch A:    [Place]  [Aggr]  [OBUpdate] [Cache]  [Match] [Settle] [Cleanup]
Batch B:             [Place]  [Aggr]  [OBUpdate] [Cache]  [Match] [Settle] [Cleanup]
Batch C:                      [Place]  [Aggr]  [OBUpdate] [Cache]  [Match] [Settle]

At any slot, 4–6 stages execute concurrently for different batches.
All touch different accounts → Sealevel runs them in parallel.
```

### 9.2 Pipeline Stage Reference

---

#### Stage 1 — Limit Order Queue
`slot: n` | Producer | User-Signed

**Purpose:** Accept a new limit order or cancel request from a trader. Validate inputs. Write the order into the transient queue accounts batched by slot and side.

**Inputs:**
- `side` — ask or bid
- `price` — limit price
- `quantity` — base token amount
- `quote` — price × quantity (pre-computed by client)
- OR: `cancel` flag + `order_entry_id`

**Accounts:**

| Account | Access | Notes |
|---|---|---|
| `MarketConfig` | Read | Validate market reference |
| `AccountStatus` | Read | Validate user balance |
| `LimitOrderQueue` (side) | Write | Destination queue |
| — `PriceQueue` | Write | Price node entries |
| — `PriceTrie` | Write | Trie structure for price |
| — `EntriesQueue` | Write | Order entry data |
| — `AddressIndexQueue` | Write | Ledger address coords |
| — `CoordinatesQueue` | Write | Trie path coordinates |
| Owner/Trader | Signer | — |

**Validation Logic:**
- `quote % price == 0` — quote must be divisible by price
- `quote % quantity == 0` — quote must be divisible by quantity
- If ask: `amount <= AccountStatus.quantity_amount`
- If bid: `amount <= AccountStatus.quote_amount`
- `AccountStatus.market_config_account == MarketConfig`
- `LimitOrderQueue.market_config_account == MarketConfig`
- `AccountStatus.owner == Owner`
- `Owner.is_signer == true`

**Process:**
1. Validate accounts and inputs
2. Store coordinates pointer into local variable
3. Traverse `PriceTrie` path — create new `TrieNode` if none exists
4. Push each `TrieNodePointer` in path onto stack
5. Find `PriceNode` in `PriceQueue` — validate price matches
6. Push `PriceNodePointer` onto stack
7. Insert `OrderEntry` into `EntriesQueue`
8. Add `address_index_pointer` to `OrderEntry`
9. Store pointer to `OrderEntry` into variable
10. Pop `PriceNodePointer` — add `p_pointer` to `OrderEntry`
11. Add `quantity` and `quote` onto `PriceNode`
12. Pop each `TrieNodePointer` — add `quantity` and `quote` to each node on path

**State Changes:** `PriceQueue`, `PriceTrie`, `EntriesQueue`, `AddressIndexQueue`, `CoordinatesQueue` all written.

---

#### Stage 2 — Market Order Queue
`slot: n` | Producer | User-Signed (concurrent with Stage 1)

**Purpose:** Accept a market order from a trader. Validate balance. Write to the market order queue.

**Inputs:**
- `side` — buy or sell
- `target_price` (optional) — do not exceed this price; `0x0000_0000` = no limit
- `target_amount` (optional) — do not exceed this amount; `0x0000_0000` = no limit

> If both `target_price` and `target_amount` are set, the constraint reached first applies.

**Event structure — 17 bytes:**

| Field | Size | Notes |
|---|---|---|
| `flag` | 1 byte | side + [quantity, quote] |
| `price` | 8 bytes | 0 = no target |
| `amount` | 8 bytes | 0 = no target |

**Accounts:**

| Account | Access | Notes |
|---|---|---|
| `MarketConfig` | Read | Validate market reference |
| `AccountStatus` | Read | Validate user balance |
| `MarketOrderQueue` | Write | 24 bytes/node, batched by slot |
| `MarketOrderPriceQueue` | Write | Price target entries |
| `AddressIndexCache` | Write | Address coord cache |
| User/Trader | Signer | — |

**Validation Logic:**
- If sell: `amount <= AccountStatus.quantity_amount`
- If buy: `amount <= AccountStatus.quote_amount`
- If price target set: validate price is non-zero rational

---

#### Stage 3 — Limit Orders Aggregation
`slot: n + 1` | Crank

**Purpose:** Read the staged limit order queue from slot `n`. Aggregate order data and prepare it for the order book update. Group entries by price for efficient insertion.

**Accounts:**

| Account | Access | Notes |
|---|---|---|
| `LimitOrderQueue` (side: ask) | Read/Consume | Prior slot's queue |
| `LimitOrderQueue` (side: bid) | Read/Consume | Prior slot's queue |
| `LimitOrderAggregation` | Write | Aggregated output |

> **Status:** Stage process, validation, and state change fields are pending finalization.

---

#### Stage 4 — Order Book Update
`slot: n + 2` | Crank | **Sets Write-Lock ON**

**Purpose:** Apply aggregated limit orders to the live order book. Insert new `TrieNode`, `PriceNode`, and `OrderBlock` entries into the appropriate slab allocators. Set the write-lock on `MarketConfig` for the duration of this stage.

**Inputs:** `action` — `[cancel, create]`

**Accounts:**

| Account | Access | Notes |
|---|---|---|
| `MarketConfig` | Write | Set write-lock flag |
| `OrderBookChangeLog` | Write | Record changes for pre-compute |
| `LimitOrderQueue` (side) | Read/Consume | Aggregated limit orders |
| TrieNode Slab (bid/ask) | Write | Insert/update TrieNodes |
| PriceNode Slab (bid/ask) | Write | Insert/update PriceNodes |
| OrderBlock Slab (bid/ask) | Write | Insert OrderEntries |

**Process:** Write-lock is set at start of this instruction. It is released in Stage 7 (Cleanup).

---

#### Stage 5 — Pre-Compute / Cache Market Orders
`slot: n + 3` | Crank

**Purpose:** Load the current order book state and pre-compute which accounts will be needed to execute each pending market order. Cache these account lists to minimize on-chain work during matching.

**Accounts:**

| Account | Access | Notes |
|---|---|---|
| `MarketOrderQueue` | Read/Consume | Market orders to process |
| `MarketOrderCache` | Write | Pre-computed output |
| `OrderBookChangeLog` | Read | Latest order book state |
| PriceNode Slab (bid/ask) | Read | Fast lookup |
| OrderBlock Slab (bid/ask) | Read | Order entries |

**Validation:**
- If buy → use ask side
- If sell → use bid side

---

#### Stage 6 — Match Orders
`slot: n + 4` | Crank

**Purpose:** Execute matching. For each queued market order, traverse the order book and match against resting limit orders. Write matched results to the settlement queue.

**Accounts:** TBD — determined by pre-computed cache from Stage 5.

**Process:**
1. Load pre-computed account list from `MarketOrderCache`
2. For each market order in queue:
   a. Traverse `TrieNode` tree to find best price
   b. Walk `PriceNode` → `OrderBlock` → `OrderEntry`
   c. Match entry: reduce `quantity_delta`/`quote_delta` at each level
   d. Flag entry for settlement
   e. Write fill record to `SettlementQueue`
3. Continue until order is fully filled or target constraints are hit

> **Status:** Full account list and detailed process pending finalization.

---

#### Stage 7 — Cleanup
`slot: n + 6` | Crank | **Releases Write-Lock**

**Purpose:** Remove matched and cancelled orders from the order book slabs. Compact empty nodes. Release the write-lock on `MarketConfig`.

**Accounts:**

| Account | Access | Notes |
|---|---|---|
| `SettlementQueue` | Write | Mark entries processed |
| Slab Allocators (write) | Write | Return freed slots to stack |
| `MarketConfig` | Write | Release write-lock flag |

**Process:**
1. For each matched/cancelled entry in `SettlementQueue`:
   - Return `OrderEntry` slot to slab free list
   - If `OrderBlock` is empty → return block slot
   - Update `PriceNode` deltas
   - If `PriceNode` is empty → remove from trie, return slot
2. Release write-lock on `MarketConfig`

---

#### Stage 8 — Settlement
`slot: n + 5` | Crank (concurrent with Cleanup)

**Purpose:** Update the ledger with the result of matched and cancelled orders. Credit/debit trader balances. Runs concurrently with Cleanup — touches ledger accounts, not order book accounts.

**Accounts:**

| Account | Access | Notes |
|---|---|---|
| `SettlementQueue` | Write | Consume fill records |
| Slab Allocators | Read | Verify entry data |
| `AccountLedger` (addresses + orders) | Write | Update trader balances |

**Validation:** `SettlementQueue` is not empty

**Process:**
1. Read fill records from `SettlementQueue`
2. Group fills by `address_index_pointer` (batch by trader)
3. For each trader's fills:
   - Update `OrderEntryStatusNode` (open → closed / partial)
   - Append `OrderEntryChangeLogNode` (fill type, slot, delta)
   - Update `AccountStatus` balances (credit received, debit sent)

---

### 9.3 Pipeline Account Disjointness (Why This Works)

| Stage | Writes To | Reads From |
|---|---|---|
| Limit Order Queue | `LimitOrderQueue` (per-user) | `AccountStatus`, `MarketConfig` |
| Market Order Queue | `MarketOrderQueue` (per-user) | `AccountStatus`, `MarketConfig` |
| Aggregation | `LimitOrderAggregation` | `LimitOrderQueue` (prior slot) |
| Order Book Update | Order Book Slabs, `MarketConfig` | `LimitOrderAggregation` |
| Pre-Compute | `MarketOrderCache` | Order Book Slabs (read-only), `MarketOrderQueue` |
| Match | `SettlementQueue` | `MarketOrderCache`, Order Book Slabs |
| Cleanup | Order Book Slabs, `MarketConfig` | `SettlementQueue` |
| Settlement | `AccountLedger`, `AccountStatus` | `SettlementQueue` |

No two concurrent stages write to the same account → Sealevel parallelizes all of them.

---

## 10. Matching Algorithm

> **Status: WIP** — The matching algorithm is actively being designed and will be iterated on during implementation. The sections below show intent and current thinking but are not yet fully accurate. They will be updated as the matching engine is built out.

### 10.1 High-Level Flow

> **Note:** The flow below shows intent but is not yet accurate. Details will be worked out during implementation.

```
Market Order arrives (buy N quantity of token A)
    │
    ▼
Traverse TrieNode tree from root
    │   → at each node: check quantity_delta >= remaining
    │   → descend toward leaf with best price
    ▼
Arrive at PriceNode (best ask price)
    │
    ▼
Navigate OrderBlock collection at PriceNode
    │   → traversal approach under evaluation (see 07-algorithm-design.md Section 3)
    │   → candidate approaches: linked list, TrieLinkNode batching, tree with delta
    ▼
Consume OrderEntry records in FIFO order
    │   → full fill: mark entry consumed, proceed to next
    │   → partial fill: reduce entry amounts, stop
    ▼
Update deltas bottom-up:
    OrderEntry → OrderBlock → [OrderTreeNode if tree approach] → PriceNode → TrieNode path
    │
    ▼
Flag top-level parent node for cleanup if it no longer has children
    │
    ▼
Write fill record to cleanup/settlement construct
    │
    ▼
Repeat for next OrderEntry / PriceNode until order is filled or constraints hit
```

### 10.2 Price-Time Priority

Orders at the same price are matched in the order they were inserted — FIFO within a price level. The `next_match_offset` on `PriceNode` tracks the current position within the first OrderBlock. Once all 16 entries in that block are matched, `next_match_offset` resets to 0, advances to the next OrderBlock, and the consumed OrderBlock is marked for cleanup.

### 10.3 Partial Fills

If a market order cannot be fully filled at one price level, matching continues to the next best price in the trie. When a price level is exhausted, the top-level parent `TrieNode` that no longer has children is flagged and added to a cleanup construct. Only the necessary parent node is flagged — not every node in the path.

> **Note:** The cleanup construct referred to here as `SettlementQueue` is a name showing intent only. It has not yet been formally defined or named. This will be worked out during implementation.

### 10.4 Delta Propagation

> **Note:** The example below shows intent. The exact propagation path at the OrderBlock level depends on which traversal approach is chosen — see `07-algorithm-design.md` Section 3.

When an `OrderEntry` is consumed or partially filled, deltas propagate upward through the tree:

```
OrderEntry.quantity -= filled_amount
OrderBlock.header.quantity_delta -= filled_amount
[OrderTreeNode subtree deltas -= filled_amount]  ← if tree traversal approach chosen
PriceNode.quantity_delta -= filled_amount
TrieNode[leaf].quantity_delta -= filled_amount
TrieNode[parent].quantity_delta -= filled_amount
... up to root
```

---

## 11. Ledger System

The Ledger is a live tracking system for open orders. It maintains the state of every active order and its relationship to the trader who placed it. When an order is closed and balances are settled, its entries are removed. The Ledger does not maintain permanent history.

---

### 11.1 Ledger Node Types

> **Note:** Node structures will be iterated on as details emerge during implementation.

#### `AddressIndexNode` — 48 bytes

Maps a ledger coordinate to a trader's public key and their open order pointers. Tracks the number of open entries for that trader. When `num_open_entries` has been zero for a set duration, the node is removed. The exact duration is still being determined.

| Field | Description |
|---|---|
| `owner_address` | Trader's wallet public key (32 bytes) |
| `num_open_entries` | Count of open orders |
| `ask_order_entry_pointer` | Head pointer into ask order entry list |
| `bid_order_entry_pointer` | Head pointer into bid order entry list |

#### `OrderEntryStatusNode` — 32 bytes

Tracks the live status of a single open order. Removed when the order is closed.

| Field | Description |
|---|---|
| `price` | Order price |
| `slot_open` | Slot when order was placed |
| `slot_update` | Slot of last status update |
| `order_block_pointer` | Current location in order book |
| `order_entry_offset` | Offset within `OrderBlock` |
| `flags` | side (ask/bid), status (open/closed) |

#### `OrderEntryChangeLogNode` — 24 bytes

A working record of state changes on an order. Cleaned up when no longer needed.

| Field | Description |
|---|---|
| `type_flag` | open / cancel / partial_fill / full_fill |
| `slot` | Slot of the event |
| `quantity_delta` | Change in quantity |
| `quote_delta` | Change in quote |

#### `OrderEntryStatusLinkNode` — 16 bytes
#### `OrderEntryChangeLogLinkNode` — 16 bytes

Support link nodes for building linked-list chains of status and changelog records per trader, per side. Each stores an array of pointers and a `next` link pointer.

### 11.2 Ledger Capacity

| Node | Size | Max per 10MB Account |
|---|---|---|
| `AddressIndexNode` | 48 bytes | 218,450 |
| `OrderEntryStatusNode` | 32 bytes | 327,670 |
| `OrderEntryChangeLogNode` | 24 bytes | 436,900 |
| `OrderEntryStatusLinkNode` | 16 bytes | 655,350 |
| `OrderEntryChangeLogLinkNode` | 16 bytes | 655,350 |

---

## 12. Settlement

> **Status: WIP** — The settlement and cleanup process is still being designed. The following captures the current intent and will be iterated on during implementation.

### 12.1 Goal

Update the Ledger with the results of all pipeline activity and mark nodes ready for cleanup. Settlement handles three cases:

- **New orders** — inserts new order entries into the Ledger when a limit order enters the order book
- **Cancelled orders** — updates Ledger entries and marks cancelled order nodes for cleanup
- **Matched orders** — updates Ledger entries for filled orders, credits and debits trader balances, and marks consumed order book nodes for cleanup

Settlement runs concurrently with Cleanup because they write to disjoint account sets — Settlement writes to Ledger accounts, Cleanup writes to order book slab accounts.

### 12.2 Settlement Stages

> **WIP** — Stage detail below reflects current thinking and is being iterated on.

**Stage S1 — Insert New Order Entries**
- Read: `OrderBlock` (new inserts)
- Consume: `InsertCacheOrderBlock`
- Write: `OrderEntryStatusNode`, `AddressIndexNode`

**Stage S2 — Cache Matched Entries**
- Consume: `OrderBlock` fill records
- Write: `CacheOrderBlock`, `NodeCleanupStack`

**Stage S3 — Update Ledger**
- Read: `AddressIndexNode`
- Consume: `CacheOrderBlock`
- Write: `OrderEntryStatusNode`, `OrderEntryChangeLogNode`

### 12.3 Cleanup Modes

**Fast Cleanup** (concurrent with settlement):
- Target: `PriceNode`, `TrieNode`, `TrieLinkNode`
- Removes empty price levels and trie nodes immediately

**Slow Cleanup** (deferred):
- Target: `OrderBlock`
- Deferred to minimize on-chain work per transaction

---

## 13. Instruction Set

> **Status: WIP** — The instruction set will be defined and finalized after the matching engine is complete. The following represents the current known set of instructions and will be iterated on as the design matures.

### 13.1 Pipeline Instructions (called by User or Crank)

| Instruction | Caller | Description |
|---|---|---|
| `limit_order_queue` | User | Place or cancel a limit order |
| `market_order_queue` | User | Place a market order |
| `limit_order_aggregation` | Crank | Aggregate slot's limit orders |
| `order_book_update` | Crank | Apply aggregated orders to book (sets write-lock) |
| `precompute_market_orders` | Crank | Pre-compute accounts for matching |
| `match_market_orders` | Crank | Execute matching, write fills |
| `cleanup` | Crank | Remove consumed entries, release write-lock |
| `settlement` | Crank | Update ledger with fills and cancels |

### 13.2 User Instructions

| Instruction | Caller | Description |
|---|---|---|
| `deposit` | User | Deposit tokens into `AccountStatus` |
| `withdraw` | User | Withdraw tokens from `AccountStatus` |
| `initialize_account_status` | User | Create a new `AccountStatus` for a market |

### 13.3 Administrative Instructions

| Instruction | Caller | Description |
|---|---|---|
| `initialize_market_config` | Admin | Create a new market |
| `initialize_slab_allocator` | Admin | Allocate a new slab account |
| `initialize_account_ledger_addresses` | Admin | Allocate ledger address account |
| `initialize_account_ledger_orders` | Admin | Allocate ledger orders account |
| `initialize_limit_order_queue` | Admin | Create queue account for a slot/side |
| `initialize_market_order_queue` | Admin | Create market order queue |
| `initialize_market_order_cache` | Admin | Create cache account |
| `initialize_address_lookup_table` | Admin | Create LUT for hot accounts |
| `allocate_slab_allocator` | Admin | Increase slab account capacity |
| `allocate_account_ledger` | Admin | Increase ledger account capacity |
| `update_address_lookup_table` | Admin | Update hot account LUT entries |
| `close_market_config` | Admin | Close a market and reclaim rent |
| `close_slab_allocator` | Admin | Close a slab account |
| `close_account_ledger` | Admin | Close ledger account |
| `close_account_status` | Admin | Close a user's market account |
| `close_limit_order_queue` | Admin | Close a queue account |
| `close_market_order_queue` | Admin | Close a market order queue |
| `close_market_order_cache` | Admin | Close a cache account |
| `close_address_lookup_table` | Admin | Close a LUT |

---

## 14. Key Formulas & Math

### 14.1 Trading Pair

```
base token / quote token
1 / price
```

### 14.2 Exchange Concepts

- `price` — value of 1 unit of base token, denominated in quote token
- `quantity` — amount of base token
- `quote` — amount of quote token exchanged; equal to `price × quantity`
- `cost` — synonym for quote
- All values are in raw token format equivalent to lamports — no decimal abstraction

### 14.3 Core Formulas

```
price × quantity = quote
quote / price    = quantity
quote / quantity = price
```

### 14.4 Simplified Ratio

```
quantity / quote → 1 / price → base / price
g = gcd(base, price)
ratio = (base / g) / (price / g)

quantity = base_multiple × n
quote    = price_multiple × n
```

### 14.5 Validation Rules

> **Note:** These are the foundational validation rules. Additional validation rules will be added as the implementation is built out.

```
quote % price    == 0   (quote divisible by price)
quote % quantity == 0   (quote divisible by quantity)
```

### 14.6 Math Implementation Notes

- All math uses raw integer format
- `u64 × u64` must use `u128` intermediate — `u64::MAX × u64::MAX` fits within `u128`
- Division: `u128 / u128 = u64`
- All inputs must be sanitized and converted to valid ratio before use

### 14.7 Post-MVP — Trading Down to the Unit

For MVP, the simplified ratio with even divisibility is enforced via the validation rules above. Post-MVP, trading down to the smallest unit of both quantity and quote will be supported.

This requires a systematic drift prevention process. Without it, partial fills and accumulated rounding residuals create tiny amounts that cannot be matched — fragmenting liquidity. The `unit_overflow` field on `AccountStatus` is the current placeholder for handling drift at the account level. The full design will be worked out during implementation.

---

## 15. Technology Stack

### 15.1 On-Chain Program

| Tool | Purpose |
|---|---|
| Pinocchio | Solana program framework (zero overhead, max CU efficiency) |
| zero-copy | Direct memory access to account data without deserialization |
| bytemuck | Safe casting of byte slices to typed structs |

### 15.2 Frontend / Client

| Tool | Purpose |
|---|---|
| React + Vite | UI framework |
| TypeScript | Type-safe client logic |
| WASM / Rust | High-performance client-side computation, state management |
| TradingView | Charting library |
| Solana wallet adapter | Wallet connection |

### 15.3 Backend / Indexer

> **Post-MVP** — will be revisited after core pipeline is complete if time permits.

| Tool | Purpose |
|---|---|
| Rust or Go | Indexer service language |
| PostgreSQL | Persistent trade and order history |
| WebSocket | Real-time order book and trade feed |

### 15.4 Client-Side State (WASM)

> **Status: WIP** — not fully defined yet. The following captures the current known responsibilities. Additional requirements will emerge during implementation.

The WASM client is responsible for:

- **Pipeline account management** — implements the sliding window concept, creating and dispatching pipeline transactions based on on-chain events
- **Leader election** — a mechanism to determine which client dispatches pipeline transactions in a decentralized way, coordinating without a central operator
- **Replica state** — maintains a local replica of order book and ledger account state
- **Trade history** — a lightweight version showing trades that occurred during the current WebSocket session. Persistent trade history is post-MVP.
- **Candles** — post-MVP, included if time permits

---

## 16. Out of Scope (v1)

The following are explicitly not being built for the hackathon submission.

| Feature | Status |
|---|---|
| Perpetuals / Options / Derivatives | Future — CLOB is the primitive |
| P2P Crank Incentive Network | Future — v1 uses client-side decision tree |
| Leader Election for Crank | Future — decentralized dispatch coordination |
| Fee Structure | Future — fee design deferred post-MVP |
| Governance | Future |
| Multi-market routing | Future |
| Advanced compaction strategies | Future — basic pipeline cleanup only in v1 |
| Mobile frontend | Future |
| Indexer / Backend | Post-MVP — revisit after core pipeline complete |
| Multi-account slab expansion | Post-MVP — single-account slab for v1 |
| Trading down to the unit | Post-MVP — requires drift prevention system |
| Cancel order (full lifecycle) | Post-MVP — design in progress, may not complete for v1 |
| Persistent trade history | Post-MVP — lightweight session-only history in v1 |
| Candles | Post-MVP — include if time permits |

---

## Appendix A — Open Questions

> For the full open questions register with status, blockers, and resolution paths, see `11-open-questions.md`.
> For resolved questions and explored topics, see `12-resolved-questions.md`.
> The items below are the highest-priority unresolved questions that affect this document directly.

**🔴 Blocks implementation — must resolve before building:**
- [ ] Cancel order lifecycle — how a cancel flows through the pipeline and how race conditions with fills are handled (Q-001)
- [ ] OrderBlock traversal approach — linked list vs TrieLinkNode batching vs tree with delta (Q-002)
- [ ] Pipeline slot offsets — confirm exact slot assignments n through n+6, resolve Settlement/Cleanup ordering (Q-003)
- [ ] SettlementQueue FillRecord layout — minimum fields for settlement without re-reading order book (Q-004)
- [ ] OrderBlockTreeNode leaf encoding — left/right field packing for leaf nodes (Q-005)

**🟠 Affects design — should resolve before building:**
- [ ] Write-lock timeout slot count — exact value for TIMEOUT_SLOTS (Q-006)
- [ ] Matching library function signatures — exact interface for passing slab byte slices (Q-007)
- [ ] Crank decision tree logic and states — full state machine definition (Q-009)
- [ ] Transient account lifecycle — when slot-tagged accounts are initialized and closed (Q-012)
- [ ] Settlement and Cleanup concurrency — confirm account access sets are fully disjoint (Q-013)
- [ ] Partial fill continuation — how the pipeline handles a market order not fully filled in one instruction (Q-014)

---

## Appendix B — Glossary

| Term | Definition |
|---|---|
| CLOB | Central Limit Order Book — a matching engine where all orders are listed and matched by price-time priority |
| Crank | An off-chain process that advances the pipeline by submitting transactions at each stage |
| Slab Allocator | A memory management system built on a raw byte array. Nodes are stored and accessed by raw byte address — a u32 literal byte offset into the account data buffer. Insert = pop raw byte address from stack, write node there. Delete = push raw byte address back onto stack. O(1) both ways. One slab per node type per side per market. |
| Write-Lock | A lock and signal on the order book's current write state. Indicates the order book is being written to. The crank decision tree reads this signal to avoid submitting conflicting transactions. Location not yet decided. |
| Pipeline Batch | A group of orders that flow through the pipeline together across a series of slots, from placement through to settlement and cleanup |
| Sliding Window | A technique borrowed from UDP networking used to align pipeline accounts to slots. A pool of N accounts maps to slots via `slot % N`. Transactions include a window of 4 slot-aligned accounts to handle slot inclusion uncertainty. |
| TrieNode | A node in the price trie. Each node represents one decimal digit at one depth. Stores cumulative `quantity_delta` and `quote_delta` for its subtree, enabling three-dimensional traversal. |
| OrderBlock | A node that batches order entries. Stores up to 16 `OrderEntry` records with cumulative delta tracking. |
| StackNode | A node in the slab allocator's stack region. Forms a linked structure (head-node, tail-node, linked-node) that tracks raw byte addresses of deleted nodes and adapts as the account grows. |
| Hot Account | An order book account containing orders at or near the current best price. Actively loaded during matching. |
| Cold Account | An order book account containing orders further from the best price. Processed through a separate slower pipeline with more available CU. |
| Delta | A cumulative sum stored at a tree node representing the total quantity or quote of all entries in its subtree |
| Ledger Coordinate | An 8-byte value composed of two 4-byte raw byte addresses into ledger slab accounts, used instead of a 32-byte public key on `OrderEntry` to reduce per-entry storage cost |
| PDA | Program Derived Address — a deterministic account address derived from seeds, owned by a program |
| LUT | Address Lookup Table — compresses transaction size by referencing accounts by 1-byte index instead of 32-byte key |
| Zero-copy | Accessing account data directly from the raw byte slice without deserialization or copying |
| Alpenglow | Solana's upcoming consensus upgrade expected to significantly increase theoretical TPS |

---

---

## 17. Slab Allocator

The SlabAllocator is the foundational memory management system that all order book and ledger nodes are built on. It manages the storage, insertion, and deletion of fixed-size nodes within a raw byte buffer using raw byte addresses — u32 values that are literal byte offsets into the account data. Every node type in the system lives inside a slab account — TrieNode, PriceNode, OrderBlock, AddressIndexNode, etc.

The slab knows what node type it stores via the `discriminator` constant on the `SlabNode` trait. The slab does not manage relationships between slab accounts — that is the traversal algorithm's responsibility.

All operations work on batches of data — a batch of inserts or a batch of deletes, not individual nodes.

---

### 17.1 Account Layout

```
┌──────────────────────────────────────────────┐  ← start of account
│  Header  (24 bytes)                          │
│  discriminator, root_node_pointer,           │
│  stack_pointer, allocator_size,              │
│  num_node_elements, flags                    │
├──────────────────────────────────────────────┤  ← aligned to node_size
│  Node Section                                │
│  raw byte array                              │
│  each node accessed by its raw byte address  │
│  grows ↓                                     │
│                                              │
│             (free space)                     │
│                                              │
│  Stack Section                               │  grows ↑
│  u32 raw byte addresses of deleted nodes     │
├──────────────────────────────────────────────┤
│  Stack-Node  (16 bytes)                      │
│  HeadNode — always at end of account         │
└──────────────────────────────────────────────┘  ← end of account
```

Accounts start at 10kb and grow in 10kb increments up to a maximum of 10mb. Growth is handled by a separate resize instruction — not inline with insert or delete.

---

### 17.2 Operations

**Insert** — if there are free addresses on the stack, pops the next raw byte address and writes node data there. No zeroing. The caller must write all fields.

**Delete** — pushes the raw byte address back onto the stack. No zeroing. Do not read a deleted node.

**Resize** — handled by a separate instruction. When `validate_memory` sets the resize interrupt flag, normal operations halt until the resize instruction runs. The resize instruction grows the account, writes a new HeadNode at the new end, and transitions the old HeadNode into a TailNode or LinkedNode.

---

### 17.3 Stack Structure

The stack is not a flat array. It is a linked structure of `StackNode`s (16 bytes each). The HeadNode always lives at the end of the account. As the account grows, the old HeadNode transitions into a TailNode or LinkedNode, and a new HeadNode is written at the new end.

`StackNode` types:
- **head-node** — default state, at the end of the account
- **tail-node** — transitional state during account growth
- **linked-node** — intermediate node in the chain (rare, rapid growth fallback)

Chain relationships:
```
[head-node]                                    ← normal state
[tail-node] → [head-node]                      ← typical after growth
[tail-node] → [linked-node; n] → [head-node]  ← rare, rapid growth
```

For the full StackNode byte layout and transition details see `20-core-systems-deep-dive.md` Part 1.

---

## 18. Priority & Order of Focus

This section defines the order in which work is approached. Each layer depends on the previous being stable before the next begins. This is not a timeline — it is a dependency order.

---

### 18.1 Build Layers

```
Layer 1 — Slab Allocator
    Foundation for everything. Pure Rust, no Solana.
    Must be complete and tested before anything else starts.
            ↓
Layer 2 — Data Structures  [order book and ledger concurrent]
    TrieNode, TrieLink, TrieLinkNode, PriceNode, OrderBlock, OrderEntry, OrderTreeNode
    Pure Rust, no Solana. Built on top of the slab allocator.
    Ledger data structures can be worked on concurrently — lower priority.
            ↓
Layer 3 — Matching Library
    Pure Rust, no Solana. The core algorithm.
    Insert, traverse, match, delta propagation.
    Extensive testing and benchmarking.
    OrderBlock traversal approach is evaluated and decided here.
            ↓
Layer 4 — Solana Integration
    Account structs, PDAs, errors, administrative instructions,
    deposit and withdraw.
            ↓
Layer 5 — Pipeline Instructions
    Limit order queue → CU benchmark
    Market order queue → CU benchmark
    Then the full pipeline regardless of benchmark results.
    Benchmarks inform the narrative, not the decision to build.
            ↓
Layer 6 — Client / Frontend  [concurrent tracks]
    ┌─────────────────────────────┐  ┌─────────────────────────────┐
    │  WASM / Client Logic        │  │  Frontend UI                │
    │  Starts at Layer 4          │  │  Starts at Layer 2          │
    │                             │  │  with mock data             │
    │  - Instruction builders     │  │                             │
    │  - Account decoders         │  │  - Order placement form     │
    │  - Crank decision tree      │  │  - Order book display       │
    │  - Pipeline account mgmt    │  │  - Trade history feed       │
    │                             │  │                             │
    │  Math + decision tree logic │  │  Does not block on WASM     │
    │  can start at Layer 2/3 —   │  │  or backend being complete  │
    │  pure logic, no program     │  │                             │
    │  needed                     │  │                             │
    └─────────────────────────────┘  └─────────────────────────────┘
```

---

### 18.2 Key Priorities Within Each Layer

**Layer 1 — Slab Allocator:**
- Single-account slab only for MVP
- Insert, delete, respond to allocation events
- Full test coverage before moving on — locked once done

**Layer 2 — Data Structures:**
- TrieNode and TrieLink first — most complex, most novel
- PriceNode second
- OrderBlock and OrderEntry third
- OrderTreeNode alongside OrderBlock — traversal approach evaluated here
- Ledger nodes concurrently — lower priority than order book nodes

**Layer 3 — Matching Library:**
- Build `TestSlab` wrapper first to enable isolated testing
- Implement simplest OrderBlock traversal first to establish baseline
- Implement tree-based traversal as primary candidate
- Benchmark both — choose based on data
- Delta invariant checker must be built and run on all tests

**Layer 4 — Solana Integration:**
- Account structs and PDAs locked first — no changes after this
- Init instructions before any pipeline instructions
- Deposit and withdraw to validate token flow

**Layer 5 — Pipeline:**
- Limit order queue first — CU benchmark immediately after
- Market order queue second — CU benchmark immediately after
- Full pipeline built regardless of benchmark results
- Cancel order lifecycle deferred until design is resolved

**Layer 6 — Client / Frontend:**
- Math and decision tree logic can start at Layer 2/3 — pure logic, no program needed
- Frontend UI built with mocks from Layer 2 — does not block on backend
- WASM instruction builders and account decoders start at Layer 4
- Crank decision tree is the most critical client component
- Leader election is post-MVP

---

### 18.3 What Is Explicitly Deferred

Anything not in Layers 1–6 above is deferred. See Section 16 for the full out of scope list.

---

## 19. Matching Engine

The matching engine is the core of Incognitus. Everything else in the system — the Solana program, the pipeline, the accounts, the ledger — exists to serve it or to interface with it.

### 19.1 What the Matching Engine Is

The matching engine is a pure algorithmic system. It takes order data and produces fill results. It operates on raw byte buffers. It has no knowledge of:

- Solana accounts, PDAs, or the account model
- The pipeline, slot timing, or the crank
- The ledger or trader identities
- Fees, balances, or token transfers

Its only concern is: given a market order and a collection of resting limit orders organized in the order book data structures, find the best matching entries, consume them, update the deltas, and return fill records.

### 19.2 Its Boundaries

```
┌─────────────────────────────────────────────────┐
│  Solana Program (interface layer)               │
│  - Account validation                           │
│  - Instruction routing                          │
│  - Pipeline coordination                        │
│  - CU budget management                         │
│       │                                         │
│       ▼                                         │
│  ┌─────────────────────────────────────────┐   │
│  │  Matching Engine (core)                 │   │
│  │  - TrieNode traversal                   │   │
│  │  - OrderBlock navigation                │   │
│  │  - Entry consumption                    │   │
│  │  - Delta propagation                    │   │
│  │  - Fill record production               │   │
│  └─────────────────────────────────────────┘   │
│       │                                         │
│       ▼                                         │
│  Raw byte buffers (slab account data)           │
└─────────────────────────────────────────────────┘
```

The Solana program is the interface. It loads the right accounts, validates inputs, manages the pipeline, and passes raw byte slices into the matching engine. The matching engine does its work and returns results. The program takes those results and writes them to the appropriate accounts.

### 19.3 Why This Separation Matters

Building the matching engine as a pure Rust library — before any Solana integration — means:

- It can be tested with `cargo test` in milliseconds, no validator needed
- It can be benchmarked with `criterion` to measure CU-equivalent cost before any on-chain work begins
- The interface between the engine and the Solana program is a clear, explicit boundary — two people can work on each side independently once the interface is agreed
- Bugs in the algorithm are found and fixed at the cheapest possible level

### 19.4 What It Produces

For each market order processed, the matching engine produces:
- A set of fill records — each containing the coordinates of the matched entries and the amounts exchanged
- Updated delta values propagated through the trie and order block structures
- Flags on exhausted price levels and nodes pending cleanup

It does not produce: token transfers, ledger updates, or balance changes. Those are the settlement layer's responsibility.

### 19.5 Current Status

The matching engine is the first thing to be built after the slab allocator and data structures are complete (Layer 3 in the build order). The exact function signatures and interface are still being worked out. See `11-open-questions.md` Q-007 and `07-algorithm-design.md` for the full design detail.

---

## Appendix C — Areas for Further Exploration

> This appendix captures broad topics that require dedicated design sessions, research, and deeper thought before they can be fully documented. These are not specific open questions — they are areas where the full picture is not yet understood. Each has a direct impact on the architecture and implementation and will be expanded as the project matures.

---

### C.1 Constraints

Incognitus is built within a specific set of hard constraints imposed by Solana and the BPF runtime. Many architectural decisions exist specifically because of these constraints. Understanding them is essential before building.

**What needs to be explored and documented:**

- **Account size** — max 10MB per account, fixed at creation, resize is expensive. How does this shape the slab capacity planning and multi-account expansion strategy?
- **Transaction size** — limited accounts per transaction (64 with LUTs), limited instruction data size. How does this constrain the pipeline stage design and the pre-compute approach?
- **Compute units** — fixed budget per transaction. What is the actual CU cost of each hot-path operation? How does this inform the matching engine design and the 5k CU target?
- **Sealevel parallelism** — only transactions with disjoint write sets run in parallel. How does this constrain the pipeline stage account design?
- **Slot timing** — ~400ms per slot, transaction inclusion is non-deterministic. How does this affect the sliding window sizing and pipeline latency?
- **BPF constraints** — no dynamic memory allocation, no std, limited stack size. How does this shape the data structure and algorithm implementation choices?
- **Rent** — accounts must maintain minimum balance or be closed. How does rent factor into the transient account lifecycle and slab account management?
- **Account limits per transaction** — how many accounts can realistically be loaded in a single matching instruction given the order book fragmentation?

**Why this matters:**
Every constraint above has already influenced at least one architectural decision. Documenting them explicitly gives the team the reasoning behind those decisions and helps them make good decisions when new ones arise.

---

### C.2 Error Handling Philosophy

The pipeline is a multi-stage, multi-batch system. When something goes wrong at one stage, the impact on the rest of the pipeline depends on what state was left behind. There is no single transaction that can be rolled back — each stage is its own transaction.

**What needs to be explored and documented:**

- **What happens when each pipeline stage fails?** — which failures leave state clean, which leave state dirty?
- **Write-lock failure** — if the OB Update instruction fails after setting the write-lock but before completing, the lock is stuck. What is the recovery path?
- **Partial pipeline advancement** — if a batch advances to Stage 4 but the crank stops, what happens to that batch? Does it block new batches? How is it recovered?
- **Invalid state detection** — how does the system detect that the pipeline is in an inconsistent state? What signals does it emit?
- **Recovery instructions** — are there administrative instructions specifically for recovering from stuck or corrupted pipeline state?
- **Acceptable vs unacceptable failure modes** — some failures are recoverable (crank restarts), some are not (slab data corruption). What is the line?

**Why this matters:**
A pipeline system that fails silently or leaves state in an unknown condition is dangerous for a financial system. The team needs a shared mental model of what failure looks like and how to recover before they write the first instruction.

---

### C.3 Security Considerations

Incognitus is a permissionless financial system. The attack surface is meaningful and specific to this architecture.

**What needs to be explored and documented:**

- **Write-lock griefing** — can a malicious actor spam OB Update transactions to keep the write-lock perpetually active, freezing the market? What rate limiting or authorization exists?
- **Crank griefing** — can someone submit malformed crank transactions that advance a pipeline stage incorrectly or consume queue data without processing it?
- **Slab pointer manipulation** — what prevents a malicious instruction from crafting a transaction with an out-of-bounds slot index to corrupt slab data?
- **Pre-computed value manipulation** — the divisibility validation rules (Section 14.5) protect against incorrect pre-computed amounts. Are there other manipulation vectors?
- **Pipeline replay** — can a completed pipeline batch be replayed? What prevents the same fill record from being settled twice?
- **Account substitution** — can a malicious actor pass a fake account where a real slab account is expected? How are account identities validated?
- **Slot window abuse** — can someone create slot-tagged accounts for future slots to interfere with future pipeline batches?

**Why this matters:**
These attack vectors are specific to the pipeline architecture and the slab-based order book. Standard Solana security practices (discriminators, signer checks, PDA validation) address some of them but not all.

---

### C.4 Order Lifecycle and State Diagram

An order has a lifecycle from the moment it is placed to the moment it is fully settled and cleaned up. Right now this lifecycle is implied across Sections 9, 10, 11, 12, and 17 but never stated in one place.

**What needs to be explored and documented:**

- **The complete state machine for a limit order** — what states can it be in, what transitions exist, what triggers each transition?
  - Possible states: `queued`, `in_order_book`, `partially_matched`, `fully_matched`, `cancelled`, `settled`, `cleaned_up`
- **The complete state machine for a market order** — similar but different lifecycle
- **What data exists at each state** — which accounts hold the order's data at each point in its lifecycle?
- **State transitions across pipeline stages** — which stage causes which transition?
- **Failure states** — what does a stuck or corrupted order look like? How is it detected and recovered?
- **The cancel interaction** — how does a cancel interact with each possible state? (This is also Q-001 in the open questions)

**Why this matters:**
The order lifecycle is the thread that connects every section of this document. Without a clear state diagram, developers will have different mental models of what an order is at any given point in time — leading to bugs at the boundaries between pipeline stages.

---

## Companion Documents

| Document | Purpose |
|---|---|
| `02-decision-log.md` | Formal architectural decisions with alternatives and tradeoffs |
| `06-technical-spec.md` | Byte-level account layouts, PDA seeds, instruction specs |
| `07-algorithm-design.md` | Deep-dive on slab, TrieNode, OrderBlock traversal approaches, matching engine |
| `08-testing-strategy.md` | Test matrix, unit and integration test cases |
| `09-risk-register.md` | Top risks with mitigation plans |
| `11-open-questions.md` | All unresolved questions with status, blockers, and resolution paths |
| `12-resolved-questions.md` | Resolved questions and explored topics — the intellectual record |

*Document maintained in `/docs/overview.md`. For implementation details, see individual module READMEs in `/src`.*
