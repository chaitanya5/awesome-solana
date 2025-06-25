use super::*;
use crate::{errors::EscrowErrorCode, state::Offer};

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct MakeOffer<'info> {
    // The user making the offer
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut @EscrowErrorCode::AccountNotMutable,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata: InterfaceAccount<'info, TokenAccount>,

    // The Offer state account
    #[account(
        init,
        payer = maker,
        space = Offer::DISCRIMINATOR.len() + Offer::INIT_SPACE,
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    // The Vault ATA which holds the tokens for the offer. It's owner is the offer state account.
    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub vault_ata_a: InterfaceAccount<'info, TokenAccount>,

    // Work with either classic SPL tokens or newer token interfaces
    pub token_program: Interface<'info, TokenInterface>,

    // Used to manage associated token accounts
    pub associated_token_program: Program<'info, AssociatedToken>,

    // Used almost everywhere for creating or deleting accounts
    pub system_program: Program<'info, System>,
}

// Move the offered tokens from the maker's ATA to the vault ATA
pub fn send_offered_tokens_to_vault(
    context: &Context<MakeOffer>,
    token_a_offered_amount: u64,
) -> Result<()> {
    msg!("Send the offered tokens to vault");
    require_gt!(token_a_offered_amount, 0, EscrowErrorCode::InvalidAmount);
    require!(
        context.accounts.maker_ata.amount >= token_a_offered_amount,
        EscrowErrorCode::InsufficientMakerBalance
    );
    require!(
        context.accounts.token_mint_a.key() != context.accounts.token_mint_b.key(),
        EscrowErrorCode::InvalidTokenMint
    );

    // Transfer the tokens from the maker's ATA to the vault ATA
    transfer_tokens(
        &context.accounts.maker_ata,
        &context.accounts.vault_ata_a,
        &token_a_offered_amount,
        &context.accounts.token_mint_a,
        &context.accounts.maker,
        &context.accounts.token_program,
        None,
    )
    .map_err(|_| EscrowErrorCode::TokenTransferFailed)?;

    Ok(())
}

// Save the details of the offer in the Offer state account
pub fn save_offer_details(
    context: Context<MakeOffer>,
    id: u64,
    token_a_offered_amount: u64,
    token_b_wanted_amount: u64,
) -> Result<()> {
    msg!("Save the offer details in the Offer state account");
    require_gt!(token_b_wanted_amount, 0, EscrowErrorCode::InvalidAmount);

    // Save the details of the offer to the offer account
    context.accounts.offer.set_inner(Offer {
        id,
        maker: context.accounts.maker.key(),
        token_mint_a: context.accounts.token_mint_a.key(),
        token_mint_b: context.accounts.token_mint_b.key(),
        token_a_offered_amount,
        token_b_wanted_amount,
        bump: context.bumps.offer,
    });
    Ok(())
}
