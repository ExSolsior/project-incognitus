# project-incognitus — Boundless On-Chain CLOB

---

## What this is

An on-chain central limit order book built from first principles on Solana. Not a DEX. A **matching engine primitive** — the infrastructure layer that spot exchanges, perpetual protocols, options platforms, and structured product builders use as their foundation.

The architecture solves three problems that every existing on-chain CLOB on Solana has accepted as unsolvable: a bounded orderbook depth, a throughput ceiling imposed by crankless design, and transaction failure rates under write contention. None of these are fundamental Solana limitations. They are architectural choices. This project makes different ones.

---

## The problems with existing CLOBs

**Existing approaches**
- Orderbook bounded by single account size limit
- Eviction policies remove orders under load
- Crankless design caps orders processed per transaction
- Write contention causes high failure rates
- Settlement blocks the matching engine
- Throughput ceiling well below CEX-grade

**This approach**
- Boundless orderbook — no depth ceiling
- Millions of orders held simultaneously
- Decentralized cranking — throughput without centralization
- Write-locks eliminate contention and failure
- Settlement decoupled — matching never blocked
- 200k orders/sec, 400k matches/transaction

---

## How it works

**Boundless orderbook via slab allocation**
Orderbook data is fragmented across multiple accounts and managed by a slab allocator — the same low-level memory management system used by database storage engines. No single account size limit applies. The orderbook scales to whatever depth the market requires. Only the relevant accounts are hot-loaded per operation, with no impact on matching performance.

**Three-dimensional matching algorithm**
Two custom data structures power the matching engine:

  - **TrieNode** — organizes orders as a tree traversable by price, quantity delta, or quote delta. Locates any target order entry in O(log n). Stores pre-computed settlement amounts — enabling matching against orders without recomputing at settlement time.
  
  - **OrderBlock** — holds 16 order entries per block, arranged in its own traversable tree. Pre-computed amounts mean the matching engine only needs to find and flag — settlement happens separately.
  
The result: **400,000 orders matched per transaction.** Settlement is fully decoupled and runs in a delayed non-blocking stage. The matching engine is never stalled waiting for settlement to complete.

**Concurrent transaction pipeline**
Operations are split into five staged pipeline phases, each running concurrently without blocking the others:

  1. **Limit order placement** — heavy computation off-chain, cheap validation on-chain. Target: 5k CU per transaction → 200k orders/sec under Alpenglow.

  2. **Orderbook aggregation** — orders written into the orderbook structure. Write-locked to prevent contention.

  3. **Market order queuing** — market orders placed into a processing queue.

  4. **Matching execution** — queue processed, each market order matched against the orderbook. Write-locked.

  5. **Settlement and cleanup** — matched orders settled, ledger updated, fees computed. Delayed, non-blocking, never in the critical path.

**Decentralized cranking**
Crankless systems maintain decentralization by removing the need for an operator — but impose a hard throughput ceiling in doing so. This protocol implements cranking while preserving full censorship resistance through a **client-side decision tree** — cranking logic distributed to clients, no central operator required. A **p2p incentive network** with economic rewards for crank operators is the longer-term expansion path.

---

## Performance targets
> Theoretical maximums under Solana's Alpenglow upgrade at 5,000 CU per limit order transaction. Benchmarks published as development progresses.

- 200,000 limit orders per second
- 400,000 matched orders per transaction
- ~1,500 CU per transaction — theoretical floor with full Pinocchio optimization

---

## Stack

- **On-chain:** Pinocchio, zero-copy, bytemuck — lowest level of the Solana stack, no abstraction overhead
- **Client:** React, Vite, TypeScript, WASM/Rust, TradingView charting library
- **Backend / Indexer:** Rust or Go, PostgreSQL, WebSocket real-time pipelines

---

## Vision

Solana is positioning itself as the on-chain Nasdaq. That vision requires a matching engine primitive operating at CEX-grade speed — fully on-chain, fully decentralized, composable enough that the entire ecosystem of financial products can be built on top of it.
This is that primitive. Spot trading is the proof of concept. Perpetuals, options, and structured products are what gets built on top of it.

---

## Building in public
The entire build is documented publicly — architecture decisions, failures, breakthroughs, and benchmarks. Follow along, challenge the design, contribute.

Twitter: [0xNomadic4Life](https://x.com/0xNomadic4Life)

---

## Status
Active development — Colosseum Spring 2026, April 6 to May 11. Updated continuously as the protocol develops.

---

This project is licensed under the MIT License.

