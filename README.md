# ⚔️ Aegis Protocol

**Bot-proof prediction markets on Solana.**
Aegis eliminates front-running and MEV advantages by combining batch auctions, uniform pricing, and MEV-resistant design into a single on-chain system.

> Trade what you know — not what bots can see.

---

## 🚀 Overview

Aegis is a **permissionless prediction market protocol** where users trade on real-world binary outcomes using USDC.

Unlike traditional on-chain markets that process orders sequentially, Aegis introduces:

* **Batch auctions** → no “first-in-line” advantage
* **Uniform clearing price** → everyone trades at the same price per window
* **CoW-style internal matching** → better fills, reduced slippage
* **LMSR-based pricing** → prices reflect true probabilities

The result is a system where **speed is irrelevant** and **information is the edge**.

---

## ⚙️ Core Mechanics

### 🧩 Batch Auction Engine

Orders are collected over a short window (~3 seconds) and executed together.

* All trades clear at a **single price**
* No sequential ordering → no sandwich attacks
* No mempool advantage → no front-running

---

### 📊 LMSR Pricing Model

Aegis uses a **Logarithmic Market Scoring Rule (LMSR)** AMM.

* Ensures: `P(YES) + P(NO) = 1`
* Guarantees liquidity at all prices
* Bounded LP risk

---

### 🔄 CoW-Style Internal Netting

Opposing trades are matched before hitting the AMM.

Example:

* 300 USDC YES vs 200 USDC NO
  → 200 matched internally (zero slippage)
  → only 100 goes to AMM

Result:

* Better user execution
* Lower LP exposure

---

## 🏗️ Architecture

### On-Chain (Solana Program)

Built with **Anchor (v0.32.1)**

Core responsibilities:

* Market lifecycle management
* Order batching and settlement
* Liquidity provisioning
* Resolution + payouts

---

### Key Program Features

* Deterministic **PDA-based architecture**
* Fixed-point arithmetic (no floating point errors)
* Checked math (`u128`) for all financial operations
* Compile-time account validation via Anchor constraints

---

## 📜 Instructions

Aegis implements **8 core on-chain instructions**:

| Instruction           | Description                              |
| --------------------- | ---------------------------------------- |
| `create_market`       | Initializes market, tokens, and vault    |
| `add_liquidity`       | LP deposits USDC and receives LP tokens  |
| `submit_order`        | Locks funds and places batched order     |
| `settle_batch`        | Executes batch auction + pricing         |
| `remove_liquidity`    | Withdraws LP share + fees                |
| `propose_resolution`  | Submits outcome with bonded stake        |
| `finalize_resolution` | Finalizes outcome after challenge window |
| `redeem_winnings`     | Redeems winning tokens for USDC          |

---

## 🔐 Security Design

* No sequential execution → eliminates sandwich attacks
* Uniform clearing price → removes timing advantage
* Tick size (1%) → kills micro-arbitrage
* Pre-resolution lockout → prevents last-minute manipulation

Additional protections:

* Checked arithmetic (no overflow)
* Account validation on all instructions
* Reentrancy-safe token handling
* Deterministic PDAs (no hidden state)

---

## 🧪 Testing

* ✅ **23 / 23 tests passing**
* Covers full lifecycle:

  * Market creation
  * Trading
  * Batch settlement
  * Resolution
  * Redemption

Includes:

* Invariant testing
* End-to-end flows
* Real SPL token interactions (no mocks)

---

## 📊 Current Status

* ✅ Localnet complete
* 🔜 Devnet deployment
* ⏳ Frontend + SDK in progress
* 🔒 Audit required before mainnet

---

## 🛣️ Roadmap

### Phase 1

* Devnet deployment
* Public testing

### Phase 2

* Commit–reveal for large orders
* Dispute resolution mechanism

### Phase 3

* TypeScript SDK
* Frontend interface
* Real-time market data

### Phase 4

* Security audit
* Mainnet launch

---

## 💡 Why Aegis?

Most on-chain markets reward:

* Speed
* Bots
* MEV extraction

Aegis rewards:

* Information
* Conviction
* Fair participation

---

## 🧱 Tech Stack

* Solana
* Anchor Framework
* Rust
* SPL Token Program

---

## 🤝 Contributing

Contributions, feedback, and discussions are welcome.

If you're interested in:

* Smart contract development
* Frontend (React / Next.js)
* SDK tooling
* Market design

Feel free to open an issue or reach out.

---

## 📜 License

MIT License

---

## ⚔️ Closing

Aegis is building the foundation for **fair on-chain prediction markets**.

No bots.
No hidden advantages.
Just transparent, uniform execution.
