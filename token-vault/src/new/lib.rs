use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, entrypoint, 
    entrypoint::ProgramResult, program::{invoke, invoke_signed},
    pubkey::Pubkey, sysvar::{rent::Rent, Sysvar}, 
    program_error::ProgramError, msg,
    system_instruction, system_program, 
};
use spl_token::instruction as token_instruction;
use spl_associated_token_account::instruction::create_associated_token_account;

mod error;
use crate::error::VaultError;

entrypoint!(process_instruction);

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VaultState {
    pub vault_mint: Pubkey,
    pub total_deposits: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserAccount {
    pub amount: u64,
}

#[derive(BorshDeserialize)]
pub enum VaultInstruction {
    InitializeVault,
    InitializeUser,
    Deposit { amount: u64 },
    Withdraw { amount: u64 },
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instr = VaultInstruction::try_from_slice(input)
        .map_err(|_| VaultError::InvalidInstruction)?;
    let account_info_iter = &mut accounts.iter();

    match instr {
        VaultInstruction::InitializeVault => {
            // Accounts: [signer payer], [writable vault_state PDA], [writable vault_ATA],
            //           [read-only vault_authority], [read-only token_mint],
            //           [sys prog], [token prog], [assoc token prog], [rent]
            let payer = next_account_info(account_info_iter)?;
            let vault_state_info = next_account_info(account_info_iter)?;
            let vault_token_info = next_account_info(account_info_iter)?;
            let vault_authority_info = next_account_info(account_info_iter)?;
            let mint_info = next_account_info(account_info_iter)?;
            let system_prog = next_account_info(account_info_iter)?;
            let token_prog = next_account_info(account_info_iter)?;
            let assoc_token_prog = next_account_info(account_info_iter)?;
            let rent_sys = next_account_info(account_info_iter)?;

            // Basic checks
            if !payer.is_signer { return Err(VaultError::NotSigner.into()); }
            let rent = &Rent::from_account_info(rent_sys)?;

            // Derive the vault state PDA
            let (vault_state_pda, state_bump) = Pubkey::find_program_address(
                &[b"vault", mint_info.key.as_ref()], program_id);
            if vault_state_pda != *vault_state_info.key {
                return Err(VaultError::InvalidVaultState.into());
            }
            // Ensure PDA account is not already in use
            if vault_state_info.lamports() > 0 {
                return Err(VaultError::AlreadyInitialized.into());
            }

            // Allocate vault_state PDA
            let state_size = VaultState::try_from_slice(&[]).unwrap().try_to_vec()?.len();
            invoke_signed(
                &system_instruction::create_account(
                    payer.key,
                    vault_state_info.key,
                    rent.minimum_balance(state_size),
                    state_size as u64,
                    program_id,
                ),
                &[payer.clone(), vault_state_info.clone(), system_prog.clone()],
                &[&[b"vault", mint_info.key.as_ref(), &[state_bump]]],
            )?;

            // Initialize vault state data
            let mut vault_state_data = VaultState { 
                vault_mint: *mint_info.key, 
                total_deposits: 0 
            };
            vault_state_data.serialize(&mut &mut vault_state_info.data.borrow_mut()[..])?;

            // Derive the vault authority PDA
            let (vault_auth_pda, auth_bump) = Pubkey::find_program_address(
                &[b"vault_auth", vault_state_info.key.as_ref()], program_id);
            if vault_auth_pda != *vault_authority_info.key {
                return Err(VaultError::InvalidVaultAuthority.into());
            }

            // Create the vault's associated token account via CPI
            invoke(
                &create_associated_token_account(
                    payer.key,
                    vault_authority_info.key,
                    mint_info.key,
                ),
                &[
                    payer.clone(), 
                    vault_token_info.clone(), 
                    vault_authority_info.clone(),
                    mint_info.clone(), 
                    system_prog.clone(), 
                    token_prog.clone(), 
                    rent_sys.clone(),
                ],
            )?;

            Ok(())
        }

        VaultInstruction::InitializeUser => {
            // Accounts: [signer user], [writable user_account PDA], [read-only vault_state],
            //           [sys prog], [rent]
            let user = next_account_info(account_info_iter)?;
            let user_account_info = next_account_info(account_info_iter)?;
            let vault_state_info = next_account_info(account_info_iter)?;
            let system_prog = next_account_info(account_info_iter)?;
            let rent_sys = next_account_info(account_info_iter)?;

            if !user.is_signer { return Err(VaultError::NotSigner.into()); }

            // Verify vault_state PDA
            let vault_state = VaultState::try_from_slice(&vault_state_info.data.borrow())
                .map_err(|_| VaultError::InvalidVaultState)?;
            let (vault_state_pda, _bump) = Pubkey::find_program_address(
                &[b"vault", vault_state.vault_mint.as_ref()], program_id);
            if vault_state_pda != *vault_state_info.key {
                return Err(VaultError::InvalidVaultState.into());
            }

            // Derive user account PDA
            let (user_acc_pda, user_bump) = Pubkey::find_program_address(
                &[b"user", user.key.as_ref(), vault_state_info.key.as_ref()], program_id);
            if user_acc_pda != *user_account_info.key {
                return Err(VaultError::InvalidUserAccount.into());
            }
            if user_account_info.lamports() > 0 {
                return Err(VaultError::AlreadyInitialized.into());
            }

            // Allocate user account PDA
            let user_size = UserAccount { amount: 0 }.try_to_vec()?.len();
            invoke_signed(
                &system_instruction::create_account(
                    user.key,
                    user_account_info.key,
                    Rent::get()?.minimum_balance(user_size),
                    user_size as u64,
                    program_id,
                ),
                &[user.clone(), user_account_info.clone(), system_prog.clone()],
                &[&[b"user", user.key.as_ref(), vault_state_info.key.as_ref(), &[user_bump]]],
            )?;

            // Initialize user amount to 0
            let mut user_data = UserAccount { amount: 0 };
            user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
            Ok(())
        }

        VaultInstruction::Deposit { amount } => {
            // Accounts: [signer user], [writable user_token_acc], [writable vault_token_acc],
            //           [writable user_account], [writable vault_state], [token_program]
            let user = next_account_info(account_info_iter)?;
            let user_token_info = next_account_info(account_info_iter)?;
            let vault_token_info = next_account_info(account_info_iter)?;
            let user_account_info = next_account_info(account_info_iter)?;
            let vault_state_info = next_account_info(account_info_iter)?;
            let token_prog = next_account_info(account_info_iter)?;

            if !user.is_signer { return Err(VaultError::NotSigner.into()); }

            // Load and verify vault state
            let mut vault_state = VaultState::try_from_slice(&vault_state_info.data.borrow())
                .map_err(|_| VaultError::InvalidVaultState)?;
            let (vault_state_pda, _bump_state) = Pubkey::find_program_address(
                &[b"vault", vault_state.vault_mint.as_ref()], program_id);
            if vault_state_pda != *vault_state_info.key {
                return Err(VaultError::InvalidVaultState.into());
            }

            // Verify token accounts
            if *user_token_info.owner != spl_token::id() ||
               *vault_token_info.owner != spl_token::id() {
                return Err(VaultError::InvalidOwner.into());
            }
            let user_token_data = spl_token::state::Account::unpack(&user_token_info.data.borrow())
                .map_err(|_| VaultError::InvalidOwner)?;
            if user_token_data.mint != vault_state.vault_mint {
                return Err(VaultError::MintMismatch.into());
            }
            let vault_token_data = spl_token::state::Account::unpack(&vault_token_info.data.borrow())
                .map_err(|_| VaultError::InvalidOwner)?;
            if vault_token_data.mint != vault_state.vault_mint {
                return Err(VaultError::MintMismatch.into());
            }

            // Verify user account PDA
            let (user_acc_pda, _user_bump) = Pubkey::find_program_address(
                &[b"user", user.key.as_ref(), vault_state_info.key.as_ref()], program_id);
            if user_acc_pda != *user_account_info.key {
                return Err(VaultError::InvalidUserAccount.into());
            }
            let mut user_state = UserAccount::try_from_slice(&user_account_info.data.borrow())
                .map_err(|_| VaultError::InvalidUserAccount)?;

            // Transfer tokens from user to vault (user is authority)
            let ix = token_instruction::transfer(
                token_prog.key,
                user_token_info.key,
                vault_token_info.key,
                user.key,
                &[],
                amount,
            )?;
            invoke(
                &ix,
                &[
                    user_token_info.clone(),
                    vault_token_info.clone(),
                    user.clone(),
                    token_prog.clone(),
                ],
            )?;

            // Update balances
            user_state.amount = user_state.amount.checked_add(amount)
                .ok_or(VaultError::Overflow)?;
            vault_state.total_deposits = vault_state.total_deposits.checked_add(amount)
                .ok_or(VaultError::Overflow)?;

            // Save state
            user_state.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
            vault_state.serialize(&mut &mut vault_state_info.data.borrow_mut()[..])?;
            Ok(())
        }

        VaultInstruction::Withdraw { amount } => {
            // Accounts: [signer user], [writable user_token_acc], [writable vault_token_acc],
            //           [writable user_account], [writable vault_state], [read-only vault_authority],
            //           [token_program]
            let user = next_account_info(account_info_iter)?;
            let user_token_info = next_account_info(account_info_iter)?;
            let vault_token_info = next_account_info(account_info_iter)?;
            let user_account_info = next_account_info(account_info_iter)?;
            let vault_state_info = next_account_info(account_info_iter)?;
            let vault_authority_info = next_account_info(account_info_iter)?;
            let token_prog = next_account_info(account_info_iter)?;

            if !user.is_signer { return Err(VaultError::NotSigner.into()); }

            // Load states
            let mut vault_state = VaultState::try_from_slice(&vault_state_info.data.borrow())
                .map_err(|_| VaultError::InvalidVaultState)?;
            let mut user_state = UserAccount::try_from_slice(&user_account_info.data.borrow())
                .map_err(|_| VaultError::InvalidUserAccount)?;

            // Verify PDAs
            let (vault_state_pda, _bump_state) = Pubkey::find_program_address(
                &[b"vault", vault_state.vault_mint.as_ref()], program_id);
            if vault_state_pda != *vault_state_info.key {
                return Err(VaultError::InvalidVaultState.into());
            }
            let (vault_auth_pda, vault_auth_bump) = Pubkey::find_program_address(
                &[b"vault_auth", vault_state_info.key.as_ref()], program_id);
            if vault_auth_pda != *vault_authority_info.key {
                return Err(VaultError::InvalidVaultAuthority.into());
            }
            let (user_acc_pda, _user_bump) = Pubkey::find_program_address(
                &[b"user", user.key.as_ref(), vault_state_info.key.as_ref()], program_id);
            if user_acc_pda != *user_account_info.key {
                return Err(VaultError::InvalidUserAccount.into());
            }

            // Check sufficient user funds
            if user_state.amount < amount {
                return Err(VaultError::InsufficientFunds.into());
            }

            // Verify token accounts mint/owner
            if *user_token_info.owner != spl_token::id() ||
               *vault_token_info.owner != spl_token::id() {
                return Err(VaultError::InvalidOwner.into());
            }
            let user_token_data = spl_token::state::Account::unpack(&user_token_info.data.borrow())
                .map_err(|_| VaultError::InvalidOwner)?;
            if user_token_data.mint != vault_state.vault_mint {
                return Err(VaultError::MintMismatch.into());
            }
            let vault_token_data = spl_token::state::Account::unpack(&vault_token_info.data.borrow())
                .map_err(|_| VaultError::InvalidOwner)?;
            if vault_token_data.mint != vault_state.vault_mint {
                return Err(VaultError::MintMismatch.into());
            }

            // Transfer tokens from vault back to user (vault authority signs)
            let ix = token_instruction::transfer(
                token_prog.key,
                vault_token_info.key,
                user_token_info.key,
                vault_authority_info.key,
                &[],
                amount,
            )?;
            invoke_signed(
                &ix,
                &[
                    vault_token_info.clone(),
                    user_token_info.clone(),
                    vault_authority_info.clone(),
                    token_prog.clone(),
                ],
                &[&[b"vault_auth", vault_state_info.key.as_ref(), &[vault_auth_bump]]],
            )?;

            // Update balances
            user_state.amount = user_state.amount.checked_sub(amount)
                .ok_or(VaultError::Overflow)?;
            vault_state.total_deposits = vault_state.total_deposits.checked_sub(amount)
                .ok_or(VaultError::Overflow)?;

            // Save state
            user_state.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
            vault_state.serialize(&mut &mut vault_state_info.data.borrow_mut()[..])?;
            Ok(())
        }
    }
}

pub fn process_initialize_vault() {
    
}