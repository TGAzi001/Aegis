use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use error::*;
pub use instructions::{
    add_liquidity, create_market, finalize_resolution, propose_resolution, redeem_winnings,
    remove_liquidity, settle_batch, submit_order, AddLiquidity, BatchSettled, CreateMarket,
    FinalizeResolution, LiquidityAdded, LiquidityRemoved, ProposeResolution, RedeemWinnings,
    RemoveLiquidity, SettleBatch, SubmitOrder, *,
};
pub use state::*;

declare_id!("CpTzTQ38Q4BTzC9tSC7m1Vuiqt84vvDSASK7pAgYNAYc");

#[program]
pub mod aegis_project {
    use super::*;

    pub fn create_market(
        ctx: Context<CreateMarket>,
        question_hash: [u8; 32],
        b_param: u64,
        batch_window_slots: u64,
        resolution_slot: u64,
        fee_bps: u16,
    ) -> Result<()> {
        instructions::create_market::create_market(
            ctx,
            question_hash,
            b_param,
            batch_window_slots,
            resolution_slot,
            fee_bps,
        )
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, usdc_amount: u64) -> Result<()> {
        instructions::add_liquidity::add_liquidity(ctx, usdc_amount)
    }

    pub fn submit_order(
        ctx: Context<SubmitOrder>,
        outcome: crate::state::Outcome,
        amount: u64,
    ) -> Result<()> {
        crate::instructions::submit_order::submit_order(ctx, outcome, amount)
    }

    pub fn settle_batch<'info>(
        ctx: Context<'_, '_, 'info, 'info, SettleBatch<'info>>,
    ) -> Result<()> {
        // instructions::settle_batch::settle_batch(ctx)
        crate::instructions::settle_batch(ctx)
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lp_tokens_to_burn: u64) -> Result<()> {
        crate::instructions::remove_liquidity::remove_liquidity(ctx, lp_tokens_to_burn)
    }

    pub fn propose_resolution(
        ctx: Context<ProposeResolution>,
        outcome: bool,
        bond_amount: u64,
    ) -> Result<()> {
        crate::instructions::propose_resolution::propose_resolution(ctx, outcome, bond_amount)
    }

    pub fn finalize_resolution(ctx: Context<FinalizeResolution>) -> Result<()> {
        crate::instructions::finalize_resolution::finalize_resolution(ctx)
    }

    pub fn redeem_winnings(ctx: Context<RedeemWinnings>) -> Result<()> {
        crate::instructions::redeem_winnings::redeem_winnings(ctx)
    }
}
