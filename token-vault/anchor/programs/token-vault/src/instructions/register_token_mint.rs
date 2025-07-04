use crate::error::VaultError;
use crate::state::VaultState;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct RegisterTokenMint<'info> {
    // Protocol/Project Owner - Can be a DAO or a single entity
    #[account(mut @VaultError::AccountNotMutable)]
    pub project_owner: Signer<'info>,

    // The SPL token mint account. 
    #[account(mint::token_program = token_program)]
    pub token_mint: InterfaceAccount<'info, Mint>,

    // The Vault state account
    #[account(
        has_one = project_owner,
        seeds = [b"vault_state", project_owner.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    // The Vault ATA which holds the tokens for the project. It's owner is the vault state account.
    #[account(
        init_if_needed,
        payer = project_owner,
        associated_token::mint = token_mint,
        associated_token::authority = vault_state,
        associated_token::token_program = token_program,
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    // Work with either classic SPL tokens or newer token interfaces
    pub token_program: Interface<'info, TokenInterface>,

    // Used to manage associated token accounts
    pub associated_token_program: Program<'info, AssociatedToken>,

    // Used almost everywhere for creating or deleting accounts
    pub system_program: Program<'info, System>,
}

pub fn handler(_context: Context<RegisterTokenMint>) -> Result<()> {
    Ok(())
}
