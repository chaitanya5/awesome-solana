use crate::error::VaultError;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    pubkey::Pubkey,
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
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer = next_account_info(account_info_iter)?;
    let vault_ata = next_account_info(account_info_iter)?;
    let token_mint = next_account_info(account_info_iter)?;
    let token_prog = next_account_info(account_info_iter)?;

    // Basic checks
    if !payer.is_signer {
        return Err(VaultError::NotSigner.into());
    }
    if !vault_ata.is_writable {
        return Err(VaultError::NotSigner.into());
    }

    // Derive a PDA which acts as owner for the vault(And vault is basically ATA for this program)
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", token_mint.key.as_ref()], program_id);

    if !vault_ata.data_is_empty() {
        return Err(VaultError::AlreadyInitialized.into());
    }

    // Create Vault or in simpler terms it's the ATA for this program
    msg!("Creating program token vault account");

    // Create the instruction - ATA for this program owned by pda
    let create_ata_ix =
        create_associated_token_account(payer.key, &vault_pda, token_mint.key, token_prog.key);

    invoke(&create_ata_ix, &[vault_ata.clone(), token_mint.clone()])?;

    Ok(())
}
