use anchor_lang::prelude::*;

// ── LpPool ────────────────────────────────────────────────────────
// One PDA per market. Tracks total liquidity and LP token supply.
// Seeds: [b"lp_pool", market.key()]

#[account]
pub struct LpPool {
    /// The market this pool belongs to
    pub market: Pubkey,

    /// Total USDC currently in the pool
    pub total_liquidity: u64,

    /// Total LP tokens in circulation
    pub total_lp_supply: u64,

    /// SPL mint for LP receipt tokens
    pub lp_mint: Pubkey,

    /// Fees accrued — distributed proportionally on withdrawal
    pub cumulative_fees: u64,

    /// Slot of last settle_batch — used to enforce withdrawal timing
    pub last_settled_slot: u64,

    /// Canonical bump
    pub bump: u8,
}

impl LpPool {
    pub const LEN: usize = 8   // discriminator
        + 32   // market
        + 8    // total_liquidity
        + 8    // total_lp_supply
        + 32   // lp_mint
        + 8    // cumulative_fees
        + 8    // last_settled_slot
        + 1    // bump
        + 64; // padding
}
