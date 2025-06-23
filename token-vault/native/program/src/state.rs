use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserState {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub is_initialized: bool,
}
impl UserState {
    // Size of this struct
    pub const LEN: usize = 32 + 32 + 8 + 1;
}
