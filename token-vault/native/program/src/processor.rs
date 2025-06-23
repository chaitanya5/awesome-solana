use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::error;
use crate::instructions::{
    initialize_vault::initialize_vault,
    initialize_user::initialize_user,
    deposit_tokens::deposit_tokens,
    withdraw_tokens::withdraw_tokens,

};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum VaultInstruction {
    InitializeVault,
    InitializeUser,
    Deposit { amount: u64 },
    Withdraw { amount: u64 },
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instr = VaultInstruction::try_from_slice(instruction_data)
        .map_err(|_| error::VaultError::InvalidInstruction)?;

    match instr {
        VaultInstruction::InitializeVault => initialize_vault(program_id, accounts, instruction_data),
        VaultInstruction::InitializeUser => initialize_user(program_id, accounts, instruction_data),
        VaultInstruction::Deposit { amount } => deposit_tokens(program_id, accounts, amount),
        VaultInstruction::Withdraw { amount } => withdraw_tokens(program_id, accounts, amount),
    }
}
