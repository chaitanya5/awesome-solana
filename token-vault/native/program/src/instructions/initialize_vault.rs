use crate::error::VaultError;
use crate::state;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    pubkey::Pubkey,
    system_instruction, system_program,
};
use spl_associated_token_account::instruction::create_associated_token_account;

/// Accounts:
/// [signer payer]
/// [writable vault_ATA]
/// [readonly token_mint]
/// [readonly token program]
/// [readonly system program]
pub fn initialize_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let token_mint = next_account_info(account_info_iter)?;
    let token_prog = next_account_info(account_info_iter)?;
    let system_prog = next_account_info(account_info_iter)?;

    // Basic checks
    if !payer.is_signer {
        return Err(VaultError::NotSigner.into());
    }
    if !vault_ata.is_writable {
        return Err(VaultError::NotSigner.into());
    }

    // Derive the vault state PDA
    let (vault_state_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", token_mint.key.as_ref()], program_id);

    if vault_state_pda != vault_ata.key {
        return Err(VaultError::InvalidVaultState.into());
    }

    if !vault_ata.data_is_empty() {
        return Err(VaultError::AlreadyInitialized.into());
    }

    // Create Vault
    msg!("Creating program token vault account");
    invoke(
        &create_associated_token_account(payer.key, payer.key, token_mint.key, token_prog.key),
        &[vault_ata.clone(), token_mint.clone(), rent_.clone()],
    )?;

    Ok(())
}
