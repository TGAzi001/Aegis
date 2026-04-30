use crate::state::market::Outcome;
use anchor_lang::prelude::*;

// ── BatchOrder ────────────────────────────────────────────────────
// One PDA per pending order, per batch window.
// Seeds: [b"order", market.key(), user.key()]
//
// Created by submit_order.
// Consumed (and closed) by settle_batch.
// Closing returns rent to the user — no dust accumulation.

#[account]
pub struct BatchOrder {
    /// The market this order belongs to
    pub market: Pubkey,

    /// User who submitted the order
    pub user: Pubkey,

    /// Bet direction
    pub outcome: Outcome,

    /// USDC amount (already transferred to vault at submit time)
    pub amount_in: u64,

    /// Which batch window this order was submitted in
    /// Must match market.batch_slot_start at settle time
    /// Prevents stale order replay attacks
    pub batch_slot_start: u64,

    /// Commit-reveal: hash(outcome + amount + nonce)
    /// Zero if this is a standard (non-commit-reveal) order
    pub commitment_hash: [u8; 32],

    /// Whether this order used commit-reveal
    pub is_commit_reveal: bool,

    /// Whether the order has been revealed (for commit-reveal orders)
    pub is_revealed: bool,

    /// Whether the order has been filled by settle_batch
    pub is_filled: bool,

    /// Canonical bump
    pub bump: u8,
}

impl BatchOrder {
    pub const LEN: usize = 8   // discriminator
        + 32   // market
        + 32   // user
        + 1    // outcome (enum)
        + 8    // amount_in
        + 8    // batch_slot_start
        + 32   // commitment_hash
        + 1    // is_commit_reveal
        + 1    // is_revealed
        + 1    // is_filled
        + 1    // bump
        + 32; // padding
}
