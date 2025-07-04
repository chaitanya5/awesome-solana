use anchor_lang::prelude::*;

// VaultState account represents the state of the vault. Acts as authority for the vault token accounts.
// Acts as a central registry for the project owner to manage registered mints and their associated vault token accounts.
#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub project_owner: Pubkey, // Public key of the project owner
    pub bump: u8, // Bump seed for PDA (Program Derived Address) to ensure uniqueness
}

#[account]
#[derive(InitSpace)]
pub struct UserState {
    pub user: Pubkey, // Public key of the user
    pub token_mint: Pubkey, // Public key of the token mint
    pub total_deposited: u64, // Total number of tokens deposited
    pub total_withdrawn: u64, // Total number of tokens withdrawn
    pub available_balance: u64, // Available user balance
    pub bump: u8, // Bump seed for PDA (Program Derived Address) to ensure uniqueness
}