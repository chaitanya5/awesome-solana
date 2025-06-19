use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VaultState {
    pub vault_mint: Pubkey,
    pub total_deposits: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserAccount {
    pub amount: u64,
}
