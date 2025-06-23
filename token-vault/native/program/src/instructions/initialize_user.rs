use crate::error::VaultError;
use crate::state::UserState;

use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed},
    pubkey::Pubkey,
    sysvar::{Sysvar, rent::Rent},
};
use solana_system_interface::{instruction, program};

/// Accounts:
/// [signer payer]
/// [writable user_state]
/// [readonly token_mint]
/// [readonly token program]
/// [readonly system program]
pub fn initialize_user(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer = next_account_info(account_info_iter)?;
    let user_state = next_account_info(account_info_iter)?;
    let token_mint = next_account_info(account_info_iter)?;
    let token_prog = next_account_info(account_info_iter)?;
    let system_prog = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;

    // Basic checks
    if !payer.is_signer {
        return Err(VaultError::NotSigner.into());
    }
    if !user_state.is_writable {
        return Err(VaultError::NotSigner.into());
    }

    // Derive a PDA for user's state
    let (state_pda, state_bump) = Pubkey::find_program_address(
        &[b"user", payer.key.as_ref(), token_mint.key.as_ref()],
        program_id,
    );

    if !user_state.data_is_empty() {
        return Err(VaultError::AlreadyInitialized.into());
    }

    // Create user state
    msg!("Creating state account for the user and mint");
    // Calculate rent-exempt lamports for the state account
    let space = UserState::LEN;
    let rent = &Rent::from_account_info(rent_sysvar)?;
    let lamports = rent.minimum_balance(space);

    // Create the instruction
    let create_state_ix = &instruction::create_account(
        payer.key,      // Payer
        user_state.key, // New account address
        lamports,       // Lamports
        space as u64,   // Space
        &program::ID,     // Owner program
    );
    invoke_signed(
        &create_state_ix,
        &[payer.clone(), user_state.clone()],
        &[&[b"user", payer.key.as_ref(), token_mint.key.as_ref(), &[state_bump]]],
    )?;

    // Initialize default state for this user
    let initial_state = UserState {
        user: *payer.key,
        mint: *token_mint.key,
        amount: 0,
        is_initialized: true,
    };

    // Now serialize and save it
    initial_state.serialize(&mut &mut user_state.data.borrow_mut()[..])?;

    Ok(())
}
