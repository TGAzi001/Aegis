use anchor_lang::prelude::*;

#[account]
pub struct ResolutionProposal {
    pub market:           Pubkey,
    pub proposer:         Pubkey,
    pub proposed_outcome: bool,    // true = YES won, false = NO won
    pub bond_amount:      u64,     // proposer's stake — lost if wrong
    pub proposed_at_slot: u64,
    pub challenge_window: u64,     // slots before auto-finalization
    pub is_disputed:      bool,
    pub is_finalized:     bool,
    pub bump:             u8,
}

impl ResolutionProposal {
    pub const LEN: usize = 8
        + 32   // market
        + 32   // proposer
        + 1    // proposed_outcome
        + 8    // bond_amount
        + 8    // proposed_at_slot
        + 8    // challenge_window
        + 1    // is_disputed
        + 1    // is_finalized
        + 1    // bump
        + 32;  // padding
}