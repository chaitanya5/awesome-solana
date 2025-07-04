pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("9wxWDkCXccoaiGXWBphS2fsB6yLjtQ622sLeWGS9V8u1");

#[program]
pub mod anchor_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        initialize_vault::handler(ctx)
    }

    pub fn register_token_mint(ctx: Context<RegisterTokenMint>) -> Result<()> {
        register_token_mint::handler(ctx)
    }

    pub fn deposit_tokens(ctx: Context<DepositTokens>, deposit_amount: u64) -> Result<()> {
        deposit_tokens::handler(ctx, deposit_amount)
    }

    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, withdraw_amount: u64) -> Result<()> {
        withdraw_tokens::handler(ctx, withdraw_amount)
    }
}
