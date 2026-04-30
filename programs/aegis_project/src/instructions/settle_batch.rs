use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::{
    error::AegisError,
    instructions::submit_order::{lmsr_yes_price_bps, round_to_tick},
    state::{BatchOrder, LpPool, Market, MarketStatus, Outcome},
};

// ── Constants ─────────────────────────────────────────────────────
pub const MAX_ORDERS_PER_BATCH: usize = 50; // CU guard — prevents DoS
pub const TICK_SIZE_BPS: u64 = 100; // 1% price increments

// ── Accounts ──────────────────────────────────────────────────────
// settle_batch is a CRANK instruction — permissionless.
// Anyone can call it once the batch window closes.
// The cranker pays no fees and gets no reward (you can add a tip later).

#[derive(Accounts)]
pub struct SettleBatch<'info> {
    /// Cranker — anyone can call this, no authority check needed
    #[account(mut)]
    pub cranker: Signer<'info>,

    /// The market being settled
    #[account(
        mut,
        seeds = [
            b"market",
            market.authority.as_ref(),
            market.question_hash.as_ref(),
        ],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    /// LP pool — fees accrued here
    #[account(
        mut,
        seeds = [b"lp_pool", market.key().as_ref()],
        bump = lp_pool.bump,
        constraint = lp_pool.market == market.key(),
    )]
    pub lp_pool: Account<'info, LpPool>,

    /// YES token mint — program mints to filled YES orders
    #[account(
        mut,
        seeds = [b"yes_mint", market.key().as_ref()],
        bump,
        constraint = yes_mint.key() == market.yes_mint,
    )]
    pub yes_mint: InterfaceAccount<'info, Mint>,

    /// NO token mint — program mints to filled NO orders
    #[account(
        mut,
        seeds = [b"no_mint", market.key().as_ref()],
        bump,
        constraint = no_mint.key() == market.no_mint,
    )]
    pub no_mint: InterfaceAccount<'info, Mint>,

    /// Market's USDC vault
    #[account(
        mut,
        associated_token::mint = collateral_mint,
        associated_token::authority = market,
        associated_token::token_program = token_program,
        constraint = collateral_vault.key() == market.collateral_vault @ AegisError::InvalidCollateralVault,
    )]
    pub collateral_vault: InterfaceAccount<'info, TokenAccount>,

    pub collateral_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// ── Handler ───────────────────────────────────────────────────────

pub fn settle_batch<'info>(
    ctx: anchor_lang::context::Context<'_, '_, 'info, 'info, SettleBatch<'info>>,
) -> Result<()> {
    let clock = Clock::get()?;

    // ── Guard 1: batch window must be closed ──────────────────────
    {
        let market = &ctx.accounts.market;
        require!(
            clock.slot
                >= market
                    .batch_slot_start
                    .checked_add(market.batch_window_slots)
                    .ok_or(AegisError::Overflow)?,
            AegisError::BatchWindowNotClosed
        );
        require!(
            market.status == MarketStatus::Active,
            AegisError::MarketNotActive
        );
    }

    // ── Guard 2: remaining accounts layout + order cap ────────────
    // Layout: first N accounts = BatchOrder PDAs, next N = destination token accounts.
    require!(
        ctx.remaining_accounts.len() % 2 == 0,
        AegisError::InvalidRemainingAccounts
    );
    let order_count = ctx.remaining_accounts.len() / 2;
    require!(
        order_count <= MAX_ORDERS_PER_BATCH,
        AegisError::TooManyOrders
    );
    let (order_infos, token_infos) = ctx.remaining_accounts.split_at(order_count);

    // ── Step 1: Aggregate net flow from pending orders ────────────
    let mut net_yes: u64 = 0;
    let mut net_no: u64 = 0;
    let mut order_data: Vec<(usize, Pubkey, Outcome, u64)> = Vec::with_capacity(order_count);
    let mut seen_orders: Vec<Pubkey> = Vec::with_capacity(order_count);

    let market_key = ctx.accounts.market.key();
    let batch_slot_start = ctx.accounts.market.batch_slot_start;

    for (index, order_info) in order_infos.iter().enumerate() {
        require!(order_info.is_writable, AegisError::InvalidRemainingAccounts);

        let order_key = order_info.key();
        require!(
            !seen_orders.contains(&order_key),
            AegisError::DuplicateOrderAccount
        );
        seen_orders.push(order_key);

        let order: Account<BatchOrder> = Account::try_from(order_info)?;
        require!(order.market == market_key, AegisError::Unauthorized);
        require!(
            order.batch_slot_start == batch_slot_start,
            AegisError::StaleOrder
        );
        require!(!order.is_filled, AegisError::OrderAlreadyFilled);
        require!(order.is_revealed, AegisError::OrderNotRevealed);

        match order.outcome {
            Outcome::Yes => {
                net_yes = net_yes
                    .checked_add(order.amount_in)
                    .ok_or(AegisError::Overflow)?;
            }
            Outcome::No => {
                net_no = net_no
                    .checked_add(order.amount_in)
                    .ok_or(AegisError::Overflow)?;
            }
        }

        order_data.push((index, order.user, order.outcome.clone(), order.amount_in));
    }

    // ── Step 2: Internal netting + uniform clearing price ─────────
    let matched = net_yes.min(net_no);
    let remaining_yes = net_yes.saturating_sub(matched);
    let remaining_no = net_no.saturating_sub(matched);

    let new_yes_qty = ctx
        .accounts
        .market
        .yes_qty
        .checked_add(remaining_yes)
        .ok_or(AegisError::Overflow)?;
    let new_no_qty = ctx
        .accounts
        .market
        .no_qty
        .checked_add(remaining_no)
        .ok_or(AegisError::Overflow)?;

    let raw_clearing_price =
        lmsr_yes_price_bps(ctx.accounts.market.b_param, new_yes_qty, new_no_qty)?;
    let clearing_price_bps = round_to_tick(raw_clearing_price, TICK_SIZE_BPS)?;
    let clearing_no_price_bps = 10_000u64
        .checked_sub(clearing_price_bps)
        .ok_or(AegisError::Overflow)?;

    // ── Step 3: Mint fills + mark orders as filled ────────────────
    let fee_bps = ctx.accounts.market.fee_bps as u64;
    let authority_key = ctx.accounts.market.authority;
    let question_hash = ctx.accounts.market.question_hash;
    let bump = ctx.accounts.market.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"market",
        authority_key.as_ref(),
        question_hash.as_ref(),
        &[bump],
    ]];

    let mut total_fees: u64 = 0;

    for (index, user, outcome, amount_in) in order_data.iter() {
        let user_token_info = &token_infos[*index];
        require!(
            user_token_info.is_writable,
            AegisError::InvalidRemainingAccounts
        );
        let user_token_account: InterfaceAccount<'info, TokenAccount> =
            InterfaceAccount::try_from(user_token_info)?;
        require!(
            user_token_account.owner == *user,
            AegisError::InvalidUserTokenAccount
        );

        let (mint_account, fill_price_bps, expected_mint) = match outcome {
            Outcome::Yes => (
                ctx.accounts.yes_mint.to_account_info(),
                clearing_price_bps,
                ctx.accounts.yes_mint.key(),
            ),
            Outcome::No => (
                ctx.accounts.no_mint.to_account_info(),
                clearing_no_price_bps,
                ctx.accounts.no_mint.key(),
            ),
        };
        require!(
            user_token_account.mint == expected_mint,
            AegisError::InvalidOutcomeMint
        );

        let fee = (*amount_in as u128)
            .checked_mul(fee_bps as u128)
            .ok_or(AegisError::Overflow)?
            .checked_div(10_000)
            .ok_or(AegisError::DivisionByZero)? as u64;
        total_fees = total_fees.checked_add(fee).ok_or(AegisError::Overflow)?;

        let amount_after_fee = amount_in.checked_sub(fee).ok_or(AegisError::Overflow)?;
        let tokens_to_mint = (amount_after_fee as u128)
            .checked_mul(10_000)
            .ok_or(AegisError::Overflow)?
            .checked_div(fill_price_bps as u128)
            .ok_or(AegisError::DivisionByZero)? as u64;
        require!(tokens_to_mint > 0, AegisError::InvalidRedeemAmount);

        let mint_ctx: CpiContext<'_, '_, '_, 'info, MintTo<'info>> = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: mint_account,
                to: user_token_info.clone(),
                authority: ctx.accounts.market.to_account_info(),
            },
            signer_seeds,
        );
        token_interface::mint_to(mint_ctx, tokens_to_mint)?;

        let mut order: Account<BatchOrder> = Account::try_from(&order_infos[*index])?;
        require!(!order.is_filled, AegisError::OrderAlreadyFilled);
        order.is_filled = true;

        msg!(
            "Filling order: user={} outcome={:?} tokens={}",
            user,
            outcome,
            tokens_to_mint
        );
    }

    // ── Step 4: Persist market + LP pool state ────────────────────
    let market = &mut ctx.accounts.market;
    market.yes_qty = new_yes_qty;
    market.no_qty = new_no_qty;
    market.batch_slot_start = clock.slot;
    market.batch_active = false;
    market.total_fees_collected = market
        .total_fees_collected
        .checked_add(total_fees)
        .ok_or(AegisError::Overflow)?;

    // ── Auto-lock market when approaching resolution slot
    if clock.slot
        >= market
            .resolution_slot
            .saturating_sub(market.batch_window_slots)
    {
        market.status = MarketStatus::Locked;
        msg!("Market locked — approaching resolution slot");
    }

    // ── Update LP pool ────────────────────────────────────────────
    let lp_pool = &mut ctx.accounts.lp_pool;
    lp_pool.cumulative_fees = lp_pool
        .cumulative_fees
        .checked_add(total_fees)
        .ok_or(AegisError::Overflow)?;
    lp_pool.last_settled_slot = clock.slot;

    // ── Emit ──────────────────────────────────────────────────────
    emit!(BatchSettled {
        market: ctx.accounts.market.key(),
        clearing_price_bps,
        net_yes,
        net_no,
        matched,
        total_fees,
        orders_filled: order_data.len() as u8,
        new_batch_slot_start: clock.slot,
    });

    msg!(
        "Batch settled: price={}bps yes={} no={} matched={} fees={}",
        clearing_price_bps,
        net_yes,
        net_no,
        matched,
        total_fees
    );

    Ok(())
}

// ── Events ────────────────────────────────────────────────────────

#[event]
pub struct BatchSettled {
    pub market: Pubkey,
    pub clearing_price_bps: u64,
    pub net_yes: u64,
    pub net_no: u64,
    pub matched: u64,
    pub total_fees: u64,
    pub orders_filled: u8,
    pub new_batch_slot_start: u64,
}
