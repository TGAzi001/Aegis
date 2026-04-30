use crate::{
    error::AegisError,
    state::{Market, MarketStatus, ResolutionProposal},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

pub const CHALLENGE_WINDOW_SLOTS: u64 = 432_000; // ~48 hours
pub const MIN_BOND_USDC: u64 = 100_000_000; // 100 USDC minimum bond

#[derive(Accounts)]
pub struct ProposeResolution<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

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

    /// Resolution proposal PDA — one per market
    #[account(
        init,
        payer = proposer,
        space = ResolutionProposal::LEN,
        seeds = [b"resolution", market.key().as_ref()],
        bump,
    )]
    pub proposal: Account<'info, ResolutionProposal>,

    /// Proposer's USDC — bond taken from here
    #[account(
        mut,
        associated_token::mint = collateral_mint,
        associated_token::authority = proposer,
        associated_token::token_program = token_program,
    )]
    pub proposer_collateral: InterfaceAccount<'info, TokenAccount>,

    /// Market vault — bond held here until finalization
    #[account(
        mut,
        constraint = collateral_vault.key() == market.collateral_vault @ AegisError::Unauthorized,
    )]
    pub collateral_vault: InterfaceAccount<'info, TokenAccount>,

    pub collateral_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn propose_resolution(
    ctx: Context<ProposeResolution>,
    outcome: bool, // true = YES won, false = NO won
    bond_amount: u64,
) -> Result<()> {
    let market = &ctx.accounts.market;
    let clock = Clock::get()?;

    // ── Guards ────────────────────────────────────────────────────
    // Market must be locked or past resolution slot
    require!(
        market.status == MarketStatus::Locked || clock.slot >= market.resolution_slot,
        AegisError::ResolutionSlotNotReached
    );
    require!(
        market.status != MarketStatus::Resolved,
        AegisError::AlreadyResolved
    );

    // Bond scales with market liquidity — griefing gets expensive
    // 1% of vault balance, min 100 USDC, max 10,000 USDC
    let vault_balance = ctx.accounts.collateral_vault.amount;
    let required_bond = (vault_balance / 100).max(MIN_BOND_USDC).min(10_000_000_000); // 10,000 USDC cap

    require!(
        bond_amount >= required_bond,
        AegisError::InsufficientLpTokens
    );

    // ── Transfer bond from proposer → vault ───────────────────────
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.proposer_collateral.to_account_info(),
            mint: ctx.accounts.collateral_mint.to_account_info(),
            to: ctx.accounts.collateral_vault.to_account_info(),
            authority: ctx.accounts.proposer.to_account_info(),
        },
    );
    token_interface::transfer_checked(
        transfer_ctx,
        bond_amount,
        ctx.accounts.collateral_mint.decimals,
    )?;

    // ── Write proposal ────────────────────────────────────────────
    let proposal = &mut ctx.accounts.proposal;
    proposal.market = market.key();
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.proposed_outcome = outcome;
    proposal.bond_amount = bond_amount;
    proposal.proposed_at_slot = clock.slot;
    proposal.challenge_window = CHALLENGE_WINDOW_SLOTS;
    proposal.is_disputed = false;
    proposal.is_finalized = false;
    proposal.bump = ctx.bumps.proposal;

    emit!(ResolutionProposed {
        market: market.key(),
        proposer: ctx.accounts.proposer.key(),
        proposed_outcome: outcome,
        bond_amount,
        challenge_ends_at: clock.slot + CHALLENGE_WINDOW_SLOTS,
    });

    msg!(
        "Resolution proposed: outcome={} bond={} challenge_window_ends={}",
        outcome,
        bond_amount,
        clock.slot + CHALLENGE_WINDOW_SLOTS
    );

    Ok(())
}

#[event]
pub struct ResolutionProposed {
    pub market: Pubkey,
    pub proposer: Pubkey,
    pub proposed_outcome: bool,
    pub bond_amount: u64,
    pub challenge_ends_at: u64,
}
