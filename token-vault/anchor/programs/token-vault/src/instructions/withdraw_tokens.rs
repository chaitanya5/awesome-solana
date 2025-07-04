use super::*;
use crate::{error::VaultError, state::VaultState, UserState};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
#[instruction(withdraw_amount: u64)]
pub struct WithdrawTokens<'info> {
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
        mut @VaultError::AccountNotMutable,
        close = depositor,
        // has_one = depositor  @VaultError::InvalidOwner, // Ensure the signer is the owner of this deposit record,
        has_one = token_mint @ VaultError::InvalidMint, // Ensure the mint matches the record,
        seeds = [b"user_state", depositor.key().as_ref(), token_mint.key().as_ref()],
        bump,
    )]
    pub user_state: Account<'info, UserState>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(context: Context<WithdrawTokens>, withdraw_amount: u64) -> Result<()> {
    let user_state = &mut context.accounts.user_state;

    require_neq!(withdraw_amount, 0, VaultError::ZeroAmount);
    require_gte!(
        user_state.available_balance,
        withdraw_amount,
        VaultError::InsufficientFunds
    );
    if context.accounts.token_mint.key() != user_state.token_mint {
        return err!(VaultError::InvalidMint);
    }

    let vault_state_signer_seeds = &[
        b"vault_state",
        &[&context.accounts.vault_state.bump],
    ];
    let signers_seeds = Some(&vault_state_signer_seeds[..]);

    // Transfer the tokens from the maker's ATA to the vault ATA
    transfer_tokens(
        &context.accounts.depositor_ata,
        &context.accounts.vault_ata,
        &withdraw_amount,
        &context.accounts.token_mint,
        &context.accounts.depositor,
        &context.accounts.token_program,
        signers_seeds,
    )
    .map_err(|_| VaultError::TokenTransferFailed)?;

    user_state.user = context.accounts.depositor.key();
    user_state.token_mint = context.accounts.token_mint.key();
    user_state.available_balance = user_state
        .available_balance
        .checked_sub(withdraw_amount)
        .ok_or(VaultError::Overflow)?;
    user_state.total_deposited = user_state
        .total_deposited
        .checked_add(0)
        .ok_or(VaultError::Overflow)?;
    user_state.total_withdrawn = user_state
        .total_withdrawn
        .checked_add(withdraw_amount)
        .ok_or(VaultError::Overflow)?;
    user_state.available_balance = user_state
        .available_balance
        .checked_sub(withdraw_amount)
        .ok_or(VaultError::Overflow)?;
    user_state.bump = context.bumps.user_state;

    // TODO: Close the user state account and return the rent to the depositor
    
    Ok(())
}
