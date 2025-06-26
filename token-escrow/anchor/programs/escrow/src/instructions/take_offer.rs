use super::shared::{close_ata, transfer_tokens};
use crate::{errors::EscrowErrorCode, state::Offer};

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct TakeOffer<'info> {
    /// The account that is taking the offer.
    #[account(mut)]
    pub taker: Signer<'info>,

    /// The maker account receives tokens so it's mutable
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

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

pub fn send_wanted_tokens_to_maker(context: &Context<TakeOffer>) -> Result<()> {
    // Send from taker ATA to maker ATA
    require!(
        context.accounts.taker_ata_b.amount >= context.accounts.offer.token_b_wanted_amount,
        EscrowErrorCode::InsufficientTakerBalance
    );
    require!(
        context.accounts.token_mint_a.key() != context.accounts.token_mint_b.key(),
        EscrowErrorCode::InvalidTokenMint
    );
    transfer_tokens(
        &context.accounts.taker_ata_b,
        &context.accounts.maker_ata_b,
        &context.accounts.offer.token_b_wanted_amount,
        &context.accounts.token_mint_b,
        &context.accounts.taker,
        &context.accounts.token_program,
        None,
    )
    .map_err(|_| EscrowErrorCode::TokenTransferFailed)?;
    Ok(())
}

pub fn withdraw_tokens_from_vault_to_taker(context: Context<TakeOffer>) -> Result<()> {
    // Since the Offer account owns the Vault, we will say
    // there is one signer (the offer), with the seeds of the specific offer account
    // We can use these signer seeds to withdraw the token from the vault
    let offer_account_seeds = &[
        b"offer",
        context.accounts.maker.to_account_info().key.as_ref(),
        &context.accounts.offer.id.to_le_bytes()[..],
        &[context.accounts.offer.bump],
    ];
    let signers_seeds = Some(&offer_account_seeds[..]);

    transfer_tokens(
        &context.accounts.vault_ata_a,
        &context.accounts.taker_ata_a,
        &context.accounts.offer.token_a_offered_amount,
        &context.accounts.token_mint_a,
        &context.accounts.offer.to_account_info(),
        &context.accounts.token_program,
        signers_seeds,
    )
    .map_err(|_| EscrowErrorCode::TokenTransferFailed)?;

    // Close the vault and return the rent to the maker
    close_ata(
        &context.accounts.vault_ata_a,
        &context.accounts.taker.to_account_info(),
        &context.accounts.offer.to_account_info(),
        &context.accounts.token_program,
        signers_seeds,
    )
    .map_err(|_| EscrowErrorCode::FailedVaultClosure)?;
    Ok(())
}
