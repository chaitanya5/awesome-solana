use anchor_lang::prelude::*;
use crate::error::VaultError;
use crate::state::VaultState;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    // Protocol/Project Owner - Can be a DAO or a single entity 
    #[account(mut @VaultError::AccountNotMutable)]
    pub project_owner: Signer<'info>,

    #[account(
        init,
        payer = project_owner,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [b"vault_state", project_owner.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>
}

pub fn handler(context: Context<InitializeVault>) -> Result<()> {
    context.accounts.vault_state.set_inner(VaultState {
        project_owner: context.accounts.project_owner.key(),
        bump: context.bumps.vault_state
    });

    Ok(())
}
