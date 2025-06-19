use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    system_instruction, system_program,
};

use crate::error;

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
        VaultInstruction::InitializeVault => Ok(()),
        VaultInstruction::InitializeUser => Ok(()),
        VaultInstruction::Deposit { amount } => Ok(()),
        VaultInstruction::Withdraw { amount } => Ok(()),
    }
    Ok(())
}
