use anchor_lang::prelude::*;
use anchor_spl::{token::{self, Mint, Token, TokenAccount, Transfer}};

declare_id!("9wxWDkCXccoaiGXWBphS2fsB6yLjtQ622sLeWGS9V8u1"); // Replace with your program ID after building

#[program]
pub mod spl_token_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.bump = *ctx.bumps.get("vault").unwrap();
        // No BTreeMap here anymore!
        Ok(())
    }

    pub fn register_token_mint(ctx: Context<RegisterTokenMint>) -> Result<()> {
        // This instruction is mainly for initializing the vault_token_account
        // and ensuring the mint is supported (by creating its dedicated vault_token_account).
        // The Vault account itself doesn't need to store a map.
        // The existence of vault_token_account PDA for a given mint implies it's "registered".
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        if amount == 0 {
            return err!(ErrorCode::ZeroAmount);
        }

        let user_deposit = &mut ctx.accounts.user_deposit;
        let deposit_mint = &ctx.accounts.deposit_mint;

        // The vault_token_account is a PDA derived from vault and deposit_mint.
        // Anchor's constraints already verify this in the context.
        // We ensure it's initialized.

        // Transfer tokens from user to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Initialize or update user's deposit record for this specific mint
        // Note: user_deposit is init_if_needed, so it will be initialized if this is the first deposit
        // for this specific mint by this user.
        user_deposit.owner = ctx.accounts.signer.key();
        user_deposit.mint = deposit_mint.key();
        user_deposit.vault_address = ctx.accounts.vault.key();
        user_deposit.amount = user_deposit.amount.checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        user_deposit.bump = *ctx.bumps.get("user_deposit").unwrap();


        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        if amount == 0 {
            return err!(ErrorCode::ZeroAmount);
        }

        let user_deposit = &mut ctx.accounts.user_deposit;
        let vault = &ctx.accounts.vault;
        let withdraw_mint = &ctx.accounts.withdraw_mint;

        // Ensure the withdraw_mint matches the one stored in user_deposit
        if user_deposit.mint != withdraw_mint.key() {
            return err!(ErrorCode::InvalidMintForUserDeposit);
        }

        // Check if user has sufficient deposited funds
        if user_deposit.amount < amount {
            return err!(ErrorCode::InsufficientFunds);
        }

        // Generate PDA seeds for the vault account to sign the transfer
        let vault_signer_seeds = &[
            b"vault",
            &vault.bump.to_le_bytes(),
        ];
        let signer_seeds = &[&vault_signer_seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: vault.to_account_info(), // The vault PDA is the authority
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, amount)?;

        // Update user's deposit record
        user_deposit.amount = user_deposit.amount.checked_sub(amount)
            .ok_or(ErrorCode::MathOverflow)?;

        // Optionally close the user_deposit account if amount is zero to save rent
        if user_deposit.amount == 0 {
            // Reassign the account to system program to mark it as closed for rent reclamation
            let dest_account_info = ctx.accounts.signer.to_account_info();
            let user_deposit_account_info = user_deposit.to_account_info();
            **user_deposit_account_info.exit(&ctx.program_id)?;** // Reallocate space to 0 and transfer lamports

            // Another way to close:
            // **user_deposit_account_info.realloc(0, false)?;** // This might not immediately reclaim rent unless destination is specified
            // **let current_rent_lamports = user_deposit_account_info.lamports();**
            // ****user_deposit_account_info.sub_lamports(current_rent_lamports)?;**
            // **dest_account_info.add_lamports(current_rent_lamports)?;**
        }

        Ok(())
    }
}


// --- Accounts Contexts ---

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault"],
        bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub authority: Signer<'info>, // The initial authority for the vault
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterTokenMint<'info> {
    #[account(
        // The vault account doesn't change state here, so it doesn't need to be `mut`
        seeds = [b"vault"],
        bump = vault.bump,
        has_one = authority, // Only the vault authority can "register" new mints (by initializing their vault_token_account)
    )]
    pub vault: Account<'info, Vault>,
    /// This account acts as the canonical vault for a specific token mint.
    /// Its existence implies the mint is "registered".
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = vault, // The vault PDA will be the authority of this token account
        seeds = [b"vault_token_account", vault.key().as_ref(), token_mint.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mint::token_program = token_program)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>, // Must be the vault authority
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
#[instruction(amount: u64)] // Add amount to instruction to help with space calculation if needed (not strictly for INIT_SPACE)
pub struct Deposit<'info> {
    #[account(
        seeds = [b"vault"],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>, // No `mut` needed, as `vault.registered_mints` is gone
    /// CHECK: This is the specific vault token account for the `deposit_mint`.
    /// Its address is derived from `vault` and `deposit_mint`.
    /// The `associated_token::authority = vault` constraint handles ownership verification.
    #[account(
        mut,
        associated_token::mint = deposit_mint,
        associated_token::authority = vault,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed, // Create if this is the first deposit for this mint by this user
        payer = signer,
        space = 8 + UserDeposit::INIT_SPACE, // Initial space for UserDeposit
        seeds = [b"user_deposit", signer.key().as_ref(), deposit_mint.key().as_ref()],
        bump
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    #[account(
        // Verify user's token account for the deposit_mint
        associated_token::mint = deposit_mint,
        associated_token::authority = signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>, // User's token account for the deposit_mint
    pub deposit_mint: Account<'info, Mint>, // The mint of the token being deposited
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>, // Needed for init_if_needed on ATA
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [b"vault"],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK: This is the specific vault token account for the `withdraw_mint`.
    /// Its address is derived from `vault` and `withdraw_mint`.
    /// The `associated_token::authority = vault` constraint handles ownership verification.
    #[account(
        mut,
        associated_token::mint = withdraw_mint,
        associated_token::authority = vault,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        close = signer, // Close the account and send remaining lamports to signer if amount becomes 0
        seeds = [b"user_deposit", signer.key().as_ref(), withdraw_mint.key().as_ref()],
        bump = user_deposit.bump,
        has_one = owner @ ErrorCode::Unauthorized, // Ensure the signer is the owner of this deposit record
        has_one = mint @ ErrorCode::InvalidMintForUserDeposit, // Ensure the mint matches the record
        has_one = vault_address @ ErrorCode::InvalidVaultForUserDeposit, // Ensure it belongs to this vault
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    #[account(
        // Verify user's token account for the withdraw_mint
        associated_token::mint = withdraw_mint,
        associated_token::authority = signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>, // User's token account for the withdraw_mint
    pub withdraw_mint: Account<'info, Mint>, // The mint of the token being withdrawn
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>, // Added for completeness, though not strictly needed here
}


// --- Account Structures ---

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub authority: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserDeposit {
    pub owner: Pubkey,
    pub vault_address: Pubkey, // Reference to the vault this deposit belongs to
    pub mint: Pubkey, // The specific token mint this deposit is for
    pub amount: u64, // The amount deposited for this specific mint
    pub bump: u8,
}

// --- Custom Errors ---

#[error_code]
pub enum ErrorCode {
    #[msg("Amount must be greater than zero.")]
    ZeroAmount,
    #[msg("Insufficient funds in your deposit for this token.")]
    InsufficientFunds,
    #[msg("The token mint is not registered with the vault. Please register it first.")]
    TokenMintNotRegistered, // This error code won't be used directly now, as registration is implicit
    #[msg("Invalid vault token account for the specified mint.")]
    InvalidVaultTokenAccount, // This error code won't be used directly now due to ATA constraints
    #[msg("Unauthorized access. Only the vault authority can perform this action.")]
    Unauthorized,
    #[msg("Arithmetic overflow or underflow occurred.")]
    MathOverflow,
    #[msg("The provided user deposit account is not for the specified mint.")]
    InvalidMintForUserDeposit,
    #[msg("The provided user deposit account does not belong to this vault.")]
    InvalidVaultForUserDeposit,
}