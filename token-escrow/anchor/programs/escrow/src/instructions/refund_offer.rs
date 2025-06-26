use super::*;
use crate::{errors::EscrowErrorCode, state::Offer};

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct RefundOffer<'info> {
    // The maker
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
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // The Offer state account
    #[account(
        mut @EscrowErrorCode::AccountNotMutable,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        has_one = token_mint_b,
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    // The Vault ATA which holds the tokens for the offer. It's owner is the offer state account.
    #[account(
        mut @EscrowErrorCode::AccountNotMutable,
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

pub fn refund_tokens_to_maker(context: Context<RefundOffer>) -> Result<()> {
    // This function is the handler for the RefundOffer instruction.
    // It currently does nothing and returns Ok(()), indicating success.
    msg!("RefundOffer handler called");
    let offer_account_seeds = &[
        b"offer",
        context.accounts.maker.to_account_info().key.as_ref(),
        &context.accounts.offer.id.to_le_bytes()[..],
        &[context.accounts.offer.bump],
    ];
    let signers_seeds = Some(&offer_account_seeds[..]);

    transfer_tokens(
        &context.accounts.vault_ata_a,
        &context.accounts.maker_ata_a,
        &context.accounts.vault_ata_a.amount,
        &context.accounts.token_mint_a,
        &context.accounts.offer.to_account_info(),
        &context.accounts.token_program,
        signers_seeds,
    )
    .map_err(|_| EscrowErrorCode::FailedRefundTransfer)?;

    // Close the vault and return the rent to the maker
    close_ata(
        &context.accounts.vault_ata_a,
        &context.accounts.maker.to_account_info(),
        &context.accounts.offer.to_account_info(),
        &context.accounts.token_program,
        signers_seeds,
    )
    .map_err(|_| EscrowErrorCode::FailedRefundClosure)?;
    Ok(())
}
