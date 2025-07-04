use super::*;
use crate::{error::VaultError, state::VaultState, UserState};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
#[instruction(deposit_amount: u64)]
pub struct DepositTokens<'info> {
    // The user depositing tokens
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut @VaultError::AccountNotMutable,
        associated_token::mint = token_mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_ata: InterfaceAccount<'info, TokenAccount>,

    // The Vault state account
    #[account(
        seeds = [b"vault_state"],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut @VaultError::AccountNotMutable,
        associated_token::mint = token_mint,
        associated_token::authority = vault_state,
        associated_token::token_program = token_program,
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = depositor,
        space = 8 +  UserState::INIT_SPACE,
        seeds = [b"user_state", depositor.key().as_ref(), token_mint.key().as_ref()],
        bump,
    )]
    pub user_state: Account<'info, UserState>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(context: Context<DepositTokens>, deposit_amount: u64) -> Result<()> {
    require_neq!(deposit_amount, 0, VaultError::ZeroAmount);

    // Transfer the tokens from the maker's ATA to the vault ATA
    transfer_tokens(
        &context.accounts.depositor_ata,
        &context.accounts.vault_ata,
        &deposit_amount,
        &context.accounts.token_mint,
        &context.accounts.depositor,
        &context.accounts.token_program,
        None,
    )
    .map_err(|_| VaultError::TokenTransferFailed)?;

    let user_state = &mut context.accounts.user_state;
    user_state.user = context.accounts.depositor.key();
    user_state.token_mint = context.accounts.token_mint.key();
    user_state.available_balance = user_state
        .available_balance
        .checked_add(deposit_amount)
        .ok_or(VaultError::Overflow)?;
    user_state.total_deposited = user_state
        .total_deposited
        .checked_add(deposit_amount)
        .ok_or(VaultError::Overflow)?;
    user_state.total_withdrawn = user_state
        .total_withdrawn
        .checked_add(0)
        .ok_or(VaultError::Overflow)?;
    user_state.available_balance = user_state
        .available_balance
        .checked_add(deposit_amount)
        .ok_or(VaultError::Overflow)?;
    user_state.bump = context.bumps.user_state;

    Ok(())
}
