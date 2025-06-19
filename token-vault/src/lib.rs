use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar::Sysvar,
};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::stable_layout::stable_vec;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::{ID as TOKEN_PROGRAM_ID, instruction as token_instruction};

mod error;
use error::TokenVaultError;

// Define the program ID. Replace with your actual program ID after building and deploying.
solana_program::declare_id!("CDbgE8B3ZRfoKKUhCjePByGHQG2LSBd2JcTNz3eeBdMt");

// Seed for the PDA vault
const VAULT_SEED: &[u8] = b"vault";
// Seed for the PDA state account - UserBalances
const STATE_SEED: &[u8] = b"state";

/// Program state struct
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct UserBalances {
    pub wallet: Pubkey, // The wallet address of the user who deposited
    pub balance: u64,   // The amount of tokens deposited
}

impl UserBalances {
    // Size of the state - UserBalances struct (wallet Pubkey + balance u64)
    pub const USERBALANCES_LEN: usize = 32 + 8;
}

/// Instructions supported by the program
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum TokenVaultInstruction {
    /// Deposit tokens into the locker
    /// Accounts:
    /// 0. `[signer]` User's wallet account
    /// 1. `[writable]` User's ATA
    /// 2. `[writable]` Current program's ATA which is PDA 'vault'
    /// 3. `[writable]` Current program's PDA state
    /// 4. `[]` Token's mint account
    /// 5. `[]` SPL Token program ID - many SPL tokens out there
    /// 6. `[]` System program ID - necessary
    /// 7. `[]` Rent sysvar - necessary during first time when program's PDA are not initialized
    Deposit {
        deposit_amount: u64,
    },
    Withdraw {
        withdraw_amount: u64,
    },
}

entrypoint!(process_instruction);
/// Process an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("TokenLocker program entrypoint");
    let instruction = TokenVaultInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        TokenVaultInstruction::Deposit { deposit_amount } => {
            process_deposit(program_id, accounts, deposit_amount)
        }
        TokenVaultInstruction::Withdraw { withdraw_amount } => {
            process_withdraw(program_id, accounts, withdraw_amount)
        }
    }
}

pub fn process_deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_amount: u64,
) -> ProgramResult {
    if deposit_amount == 0 {
        return Err(TokenVaultError::ZeroTokenDeposited.into());
    }
    let accounts_iter = &mut accounts.iter();

    // 0. Payer or user
    let user_account = next_account_info(accounts_iter)?;
    // Ensure the user's wallet is a signer
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 1. User's ATA
    let user_token_ata = next_account_info(accounts_iter)?;
    // Ensure it's writable
    if !user_token_ata.is_writable {
        return Err(ProgramError::Immutable);
    }

    // 2. Program's token vault PDA
    let program_vault = next_account_info(accounts_iter)?;
    // Ensure it's writable
    if !program_vault.is_writable {
        return Err(ProgramError::Immutable);
    }

    // 3. Program's state PDA
    let program_state = next_account_info(accounts_iter)?;
    // Ensure it's writable
    if !program_state.is_writable {
        return Err(ProgramError::Immutable);
    }

    // 4. Token mint account
    let token_mint = next_account_info(accounts_iter)?;

    // 5. SPL token program ID
    let token_program = next_account_info(accounts_iter)?;
    if token_program.key != &TOKEN_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // 6. System Program
    let system_program = next_account_info(accounts_iter)?;
    if system_program.key != &SYSTEM_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // 7. Rent sysvar
    // let rent_sysvar = next_account_info(accounts_iter)?;
    // let rent = &Rent::from_account_info(rent_sysvar)?;

    // --- PDA Derivation and Initialization ---

    // Derive token vault PDA and verify it
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, token_mint.key.as_ref()], program_id);
    if vault_pda != *program_vault.key {
        msg!("Vault PDA mismatch");
        return Err(TokenVaultError::IncorrectTokenVaultAccount.into());
    }

    // Derive state PDA and verify it
    let (state_pda, state_bump) =
        Pubkey::find_program_address(&[STATE_SEED, user_account.key.as_ref()], program_id);
    if state_pda != *program_state.key {
        msg!("State PDA mismatch");
        return Err(TokenVaultError::IncorrectUserStateAccount.into());
    }

    // Initialize Program's token vault account if it doesn't exist
    if program_vault.data_is_empty() {
        msg!("Creating program token vault account");
        invoke(
            &create_associated_token_account(
                user_account.key,
                user_account.key,
                token_mint.key,
                &TOKEN_PROGRAM_ID,
            ),
            &[
                program_vault.clone(),
                token_mint.clone(),
                rent_sysvar.clone(),
            ],
        )?;
    }

    if program_state.data_is_empty() {
        msg!("Creating program state account for user");

        // Calculate rent-exempt lamports for the state account
        let space = UserBalances::USERBALANCES_LEN;
        let lamports = Rent::get()?.minimum_balance(space);

        // Create the state account
        invoke_signed(
            &system_instruction::create_account(
                user_account.key,
                program_state.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[
                user_account.clone(),
                program_state.clone(),
                system_program.clone(),
            ],
            &[&[STATE_SEED, user_account.key.as_ref(), &[state_bump]]],
        )?;
    }

    // --- Perform Token Transfer ---
    msg!("Transferring {} tokens to vault", deposit_amount);
    invoke(
        &token_instruction::transfer(
            token_program.key,  // Token program ID
            user_token_ata.key, // Source account
            program_vault.key,  // Destination account
            user_account.key,   // Authority of source account (user's wallet)
            &[],                // Signers (empty because user_account is signer)
            deposit_amount,     // Amount
        )?,
        &[
            user_token_ata.clone(),
            program_vault.clone(),
            user_account.clone(),
            token_program.clone(),
        ],
    )?;

    // --- Update Program State ---
    // Deserialize, update, and serialize the state
    let mut state = UserBalances::try_from_slice(&program_state.data.borrow())?;
    state.balance = state
        .balance
        .checked_add(deposit_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?; // Handle overflow
    state.wallet = *user_account.key; // Ensure owner is set correctly

    state.serialize(&mut *program_state.data.borrow_mut())?;
    msg!("Deposit successful!");

    Ok(())
}

pub fn process_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    withdraw_amount: u64,
) -> ProgramResult {
    if withdraw_amount == 0 {
        return Err(TokenVaultError::ZeroTokenWithdraw.into());
    }
    let accounts_iter = &mut accounts.iter();

    // 0. Payer or user
    let user_account = next_account_info(accounts_iter)?;
    // Ensure the user's wallet is a signer
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 1. User's ATA
    let user_token_ata = next_account_info(accounts_iter)?;
    // Ensure it's writable
    if !user_token_ata.is_writable {
        return Err(ProgramError::Immutable);
    }

    // 2. Program's token vault PDA
    let program_vault = next_account_info(accounts_iter)?;
    // Ensure it's writable
    if !program_vault.is_writable {
        return Err(ProgramError::Immutable);
    }

    // 3. Program's state PDA
    let program_state = next_account_info(accounts_iter)?;
    // Ensure it's writable
    if !program_state.is_writable {
        return Err(ProgramError::Immutable);
    }

    // 4. Token mint account
    let token_mint = next_account_info(accounts_iter)?;

    // 5. SPL token program ID
    let token_program = next_account_info(accounts_iter)?;
    if token_program.key != &TOKEN_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // 6. System Program
    let system_program = next_account_info(accounts_iter)?;
    if system_program.key != &SYSTEM_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // // 7. Rent sysvar
    // let rent_sysvar = next_account_info(accounts_iter)?;
    // let rent = &Rent::from_account_info(rent_sysvar)?;

    // --- PDA Derivation and Initialization ---

    // Derive token vault PDA and verify it
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, token_mint.key.as_ref()], program_id);
    if vault_pda != *program_vault.key {
        msg!("Vault PDA mismatch");
        return Err(TokenVaultError::IncorrectTokenVaultAccount.into());
    }

    // Derive state PDA and verify it
    let (state_pda, state_bump) =
        Pubkey::find_program_address(&[STATE_SEED, user_account.key.as_ref()], program_id);
    if state_pda != *program_state.key {
        msg!("State PDA mismatch");
        return Err(TokenVaultError::IncorrectUserStateAccount.into());
    }

    // --- State Deserialization and Authorization Check ---

    // Deserialize the state to get the deposited amount and original owner
    let mut state = UserBalances::try_from_slice(&program_state.data.borrow())?;

    // Authorization check: Only the original depositor can withdraw
    if state.wallet != *user_account.key {
        msg!("Unauthorized withdraw attempt");
        return Err(TokenVaultError::UnauthorizedWithdraw.into());
    }

    // Check if there are tokens to withdraw
    if state.balance == 0 {
        msg!("No tokens deposited by this user");
        return Err(TokenVaultError::NoTokensDeposited.into());
    }

    // --- Perform Token Transfer from Vault ---

    msg!("Transferring {} tokens from vault to user", state.balance);

    // The program needs to sign for this transfer, using the PDA
    invoke_signed(
        &token_instruction::transfer(
            token_program.key,  // Token program ID
            program_vault.key,  // Source account (program's vault)
            user_token_ata.key, // Destination account (user's ATA)
            &vault_pda,         // Authority of source account (the PDA)
            &[],                // Signers (empty because invoke_signed uses PDA)
            withdraw_amount,    // Amount to transfer
        )?,
        &[
            program_vault.clone(),
            user_token_ata.clone(),
            token_program.clone(),
            // The PDA is the signer, but its AccountInfo is not passed here,
            // instead, the seeds are passed to invoke_signed.
        ],
        &[&[VAULT_SEED, token_mint.key.as_ref(), &[vault_bump]]],
    )?;

    // --- Update Program State ---
    state.balance = state
        .balance
        .checked_sub(withdraw_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // state.serialize(&mut &mut program_state.data.borrow_mut()[..])?;
    state.serialize(&mut *program_state.data.borrow_mut())?;


    // Optionally close the state account if balance is 0 and it's rent-exempt.
    // For simplicity, we'll leave it open for now.
    // However, if the account's SOL balance drops below rent-exempt, it could be reclaimed.
    // The user would need to pay rent on this account for it to persist empty.
    // Alternatively, the program could transfer its rent back to the user and close it.

    msg!("Withdrawal successful!");

    Ok(())
}
