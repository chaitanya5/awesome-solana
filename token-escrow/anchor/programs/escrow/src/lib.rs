pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
pub use instructions::*;
pub use state::*;

declare_id!("6qDkztsEJ4r73Sqk1pDdL1degbzjyccqAX2uPDCsJJWf");

#[program]
pub mod escrow {
    use super::*;

    pub fn make_offer(
        ctx: Context<MakeOffer>,
        id: u64,
        token_a_offered_amount: u64,
        token_b_offered_amount: u64,
    ) -> Result<()> {
        instructions::make_offer::send_offered_tokens_to_vault(&ctx, token_a_offered_amount)?;  // Passing reference to context
        instructions::make_offer::save_offer_details(ctx, id, token_a_offered_amount, token_b_offered_amount)
    }

    pub fn take_offer(ctx: Context<TakeOffer>) -> Result<()> {
        instructions::take_offer::send_wanted_tokens_to_maker(&ctx)?;
        instructions::take_offer::withdraw_tokens_from_vault_to_taker(ctx)
    }

    pub fn refund_offer(ctx: Context<RefundOffer>) -> Result<()> {
        instructions::refund_offer::refund_tokens_to_maker(ctx)
    }
}
