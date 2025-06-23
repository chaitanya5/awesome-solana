use crate::error::VaultError;
use crate::state::UserState;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::{invoke},
    pubkey::Pubkey};
use borsh::{BorshDeserialize, BorshSerialize};
use spl_token::{instruction as token_instruction};


/// Accounts:
/// [signer payer]
/// [writable user_ata]
/// [writable vault_ata]
/// [writable user_state]
/// [readonly token_mint]
/// [readonly token program]
pub fn deposit_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer = next_account_info(account_info_iter)?;
    let user_ata = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let user_state = next_account_info(account_info_iter)?;
    let token_mint = next_account_info(account_info_iter)?;
    let token_prog = next_account_info(account_info_iter)?;

    // Basic checks
    if !payer.is_signer {
        return Err(VaultError::NotSigner.into());
    }
    if !user_state.is_writable || !user_ata.is_writable || !vault_ata.is_writable {
        return Err(VaultError::NotWritable.into());
    }

    // Derive a PDA for user's state
    let (state_pda, state_bump) = Pubkey::find_program_address(
        &[b"user", payer.key.as_ref(), token_mint.key.as_ref()],
        program_id,
    );

    // Transfer tokens from user ATA to vault ATA
    msg!("Transferring tokens from user ATA to vault ATA");
    // Create the transfer instruction
    let transfer_ix = token_instruction::transfer(
        token_prog.key,
        user_ata.key,
        vault_ata.key,
        payer.key,
        &[payer.key],
        deposit_amount, // Amount to transfer, this should be parsed from instruction_data
    )?;

    invoke(
        &transfer_ix,
        &[
            user_ata.clone(),
            vault_ata.clone(),
            payer.clone(),
            token_mint.clone(),
            token_prog.clone(),
        ],
    )?;

    // Update user state
    msg!("Updating user state");
    // Deserialize the user state
    let mut user_state_data = UserState::try_from_slice(&user_state.data.borrow())
        .map_err(|_| VaultError::InvalidUserState)?;
    // Update the user state with the new deposit amount
    user_state_data.amount += deposit_amount;
    // Serialize the updated user state back to the account
    user_state_data.serialize(&mut &mut user_state.data.borrow_mut()[..])
        .map_err(|_| VaultError::SerializationError)?;
    msg!("Deposit successful!");
    Ok(())
}