# rUv Payout Stream Spec (Vault-Based Distribution)

## Overview

This spec defines the **rUv payout stream system**, enabling automatic, vault-secured, usage-based revenue distribution to contributors in the QuDAG Exchange. Payouts occur as agents, modules, or infrastructure nodes are used — generating rUv credits that are routed and split based on contribution roles.

## Roles & Payout Types

### 1. Agent Providers

* Earn rUv per unit of compute/storage/bandwidth consumed
* Measured via resource metering and DAG-validated transactions

### 2. Plugin/Module Creators

* Earn micro-payouts when their module is imported or executed by other agents
* Fee share is registered at module registration time

### 3. Node Operators

* Earn rUv via routing/consensus participation
* Optional: DAG validator rotation or uptime-weighted rewards

### 4. Bounty Agents

* Claim one-time or recurring rewards for completing tasks

---

## Vault-Based Routing Logic

### 1. Contributor Vault Setup

* Each contributor has a unique `VaultID`
* Vaults are created via `qudag-vault-core`
* Vaults store contributor identity, payment address (public), and access policies

### 2. Earnings Capture

Every time rUv is spent:

* The transaction is DAG-anchored and metered
* The `FeeRouter` module calculates:

  * Total rUv collected
  * Contributor shares

### 3. Distribution Algorithm

```rust
fn distribute_rUv(tx: Transaction) {
    let total = tx.total_fee;
    for (recipient, pct) in tx.payout_map {
        let amount = total * pct;
        vault.deposit(recipient, amount);
    }
    vault.deposit("rUv", total * 0.0001); // Genesis allocation
}
```

* Payout map includes agent owner, plugin developer, node signer, etc.
* All payouts are deposited to recipients' **vaults** securely

### 4. Vault Access

* Contributors withdraw to external wallet with signed proof
* Vault enforces rate limits and optional time-locks

### 5. Auditability

* All payouts are DAG-recorded
* Vaults maintain `payout history[]` log
* Optional: ZK-proof of fair distribution

---

## Configuration & Governance

* Default split templates for:

  * Single-agent jobs (95% to agent, 5% to infra)
  * Plugin-enhanced jobs (85/10/5%)
* Contributor can override % during module registration (within caps)
* Vault configuration is programmable via policy scripts (e.g., auto-withdraw every 30 days)

---

## Summary

The rUv Payout Stream system enables fair, transparent, and decentralized income for contributors. Using QuDAG Vaults ensures security, zero-custody design, and compliance with the system's utility-token model. It forms the backbone of economic sustainability within the agentic compute economy.
