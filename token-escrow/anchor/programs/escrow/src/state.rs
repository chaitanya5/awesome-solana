use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub id: u64, // Unique identifier for the offer
    pub maker: Pubkey, // Public key of the maker (creator) of the offer
    pub token_mint_a: Pubkey, // Public key of the token mint A involved in the offer
    pub token_mint_b: Pubkey, // Public key of the token mint B involved in the offer
    pub token_a_offered_amount: u64, // Amount of token A offered
    pub token_b_wanted_amount: u64, // Amount of token B required
    pub bump: u8, // Bump seed for PDA (Program Derived Address) to ensure uniqueness
}