use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar::Sysvar,
};

use borsh::{BorshDeserialize, BorshSerialize};
use spl_token::{ID as TOKEN_PROGRAM_ID, instruction as token_instruction};

mod error;
use error::TokenVaultError;

// Define the program ID. Replace with your actual program ID after building and deploying.
solana_program::declare_id!("CDbgE8B3ZRfoKKUhCjePByGHQG2LSBd2JcTNz3eeBdMt");

// Seed for the PDA vault
const VAULT_SEED: &[u8] = b"vault";

// Seed for the PDA state account
const STATE_SEED: &[u8] = b"state";

/// Program state struct
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct ProgramState {
    pub owner: Pubkey, // The wallet address of the user who deposited
    pub balance: u64,  // The amount of tokens deposited
}

impl ProgramState {
    // Size of the state account (owner Pubkey + amount u64)
    pub const LEN: usize = 32 + 8;
}

/// Instructions supported by the program
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum TokenVaultInstruction {
    /// Deposit tokens into the locker
    /// Accounts:
    ///   0. `[signer]` The user's wallet account
    ///   1. `[writable]` The user's source token account (ATA)
    ///   2. `[writable]` The program's token vault PDA account
    ///   3. `[]` The token mint account
    ///   4. `[writable]` The program's state PDA account
    ///   5. `[]` The SPL Token program ID
    ///   6. `[]` The System program ID
    ///   7. `[]` Rent sysvar
    Deposit {
        amount: u64,
    },
    Withdraw {
        amount: u64,
    },
}

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("TokenLocker program entrypoint");

    // Deserialize the instruction or error it out
    let instruction: TokenVaultInstruction =
        TokenVaultInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        TokenVaultInstruction::Deposit { amount } => {
            process_deposit(program_id, accounts, amount);
        }
        TokenVaultInstruction::Withdraw { amount } => {
            process_withdraw(program_id, accounts, amount);
        }
    }

    Ok(())
}

pub fn process_deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // 0. The user's wallet account
    let user_wallet_account = next_account_info(account_info_iter)?;
    // Ensure the user's wallet is a signer
    if !user_wallet_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 1. The user's source token account (ATA)
    let user_token_account = next_account_info(account_info_iter)?;
    // Ensure it's writable
    if !user_token_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // 2. The program's token vault PDA account
    let program_vault_account = next_account_info(account_info_iter)?;
    // Ensure it's writable
    if !program_vault_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // 3. The token mint account
    let token_mint_account = next_account_info(account_info_iter)?;

    // 4. The program's state PDA account
    let program_state_account = next_account_info(account_info_iter)?;
    // Ensure it's writable
    if !program_state_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // 5. The SPL Token program ID
    let token_program = next_account_info(account_info_iter)?;
    if token_program.key != &TOKEN_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // 6. The System program ID
    let system_program = next_account_info(account_info_iter)?;
    if system_program.key != &SYSTEM_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // 7. Rent sysvar
    let rent_sysvar = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_sysvar)?;

    // --- PDA Derivation and Initialization ---

    // Derive the PDA for the program's vault token account
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, token_mint_account.key.as_ref()], program_id);

    // Verify the vault account is the correct PDA
    if vault_pda != *program_vault_account.key {
        msg!("Vault PDA mismatch");
        return Err(ProgramError::InvalidSeeds);
    }

    // Derive the PDA for the program's state account
    let (state_pda, state_bump) =
        Pubkey::find_program_address(&[STATE_SEED, user_wallet_account.key.as_ref()], program_id);

    // Verify the vault account is the correct PDA
    if state_pda != *program_state_account.key {
        msg!("State PDA mismatch");
        return Err(ProgramError::InvalidSeeds);
    }

    // Initialize the program's state account if it doesn't exist
    if program_state_account.data_len() == 0 {
        msg!("Creating program state account for user");

        // Calculate rent-exempt lamports for the state account
        let space = ProgramState::LEN;
        let lamports = rent.minimum_balance(space);

        // Create the state account
        invoke_signed(
            &solana_program::system_instruction::create_account(
                user_wallet_account.key,   // Payer
                program_state_account.key, // New account address
                lamports,                  // Lamports
                space as u64,              // Space
                program_id,                // Owner program
            ),
            &[
                user_wallet_account.clone(),
                program_state_account.clone(),
                system_program.clone(),
            ],
            &[&[STATE_SEED, user_wallet_account.key.as_ref(), &[state_bump]]],
        )?;
        // Initialize state to 0 for this user
        let initial_state = ProgramState {
            owner: *user_wallet_account.key,
            balance: 0,
        };
        initial_state.serialize(&mut &mut program_state_account.data.borrow_mut()[..])?;
    } else {
        // Account already initiated - Deserialize the current state
        let current_state = ProgramState::try_from_slice(&program_state_account.data.borrow())?;
        if current_state.owner != *user_wallet_account.key {
            return Err(TokenVaultError::UnauthorizedWithdraw.into()); // This should ideally be caught by PDA derivation, but good to double check
        }
    }

    // Initialize the program's token vault account if it doesn't exist
    if program_vault_account.data_len() == 0 {
        msg!("Creating program token vault account");
        invoke_signed(
            &token_instruction::initialize_account2(
                token_program.key,         // Token program ID
                program_vault_account.key, // Account to initialize
                token_mint_account.key,    // Mint
                &vault_pda,                // Owner (the PDA itself)
            )?,
            &[
                program_vault_account.clone(),
                token_mint_account.clone(),
                rent_sysvar.clone(),
            ],
            &[&[VAULT_SEED, token_mint_account.key.as_ref(), &[vault_bump]]],
        )?;
    }

    // --- Perform Token Transfer ---
    msg!("Transferring {} tokens to vault", amount);
    invoke(
        &token_instruction::transfer(
            token_program.key,         // Token program ID
            user_token_account.key,    // Source account
            program_vault_account.key, // Destination account
            user_wallet_account.key,   // Authority of source account (user's wallet)
            &[],                       // Signers (empty because user_wallet_account is signer)
            amount,                    // Amount
        )?,
        &[
            user_token_account.clone(),
            program_vault_account.clone(),
            user_wallet_account.clone(),
            token_program.clone(),
        ],
    )?;

    // --- Update Program State ---

    // Deserialize, update, and serialize the state
    let mut state = ProgramState::try_from_slice(&program_state_account.data.borrow())?;
    state.balance = state
        .balance
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?; // Handle overflow
    state.owner = *user_wallet_account.key; // Ensure owner is set correctly

    state.serialize(&mut &mut program_state_account.data.borrow_mut()[..])?;

    msg!("Deposit successful!");

    Ok(())
}


fn process_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // 0. The user's wallet account (signer)
    let user_wallet_account = next_account_info(account_info_iter)?;
    if !user_wallet_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 1. The program's token vault PDA account
    let program_vault_account = next_account_info(account_info_iter)?;
    if !program_vault_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // 2. The user's destination token account (ATA)
    let user_token_account = next_account_info(account_info_iter)?;
    if !user_token_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // 3. The token mint account
    let token_mint_account = next_account_info(account_info_iter)?;

    // 4. The program's state PDA account
    let program_state_account = next_account_info(account_info_iter)?;
    if !program_state_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // 5. The SPL Token program ID
    let token_program = next_account_info(account_info_iter)?;
    if token_program.key != &TOKEN_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // 6. The System program ID (not directly used for withdrawal, but often passed)
    let _system_program = next_account_info(account_info_iter)?;

    // 7. Rent sysvar (not directly used for withdrawal, but often passed)
    let _rent_sysvar = next_account_info(account_info_iter)?;


    // --- PDA Derivation and Verification ---

    // Derive and verify the vault PDA
    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[VAULT_SEED, token_mint_account.key.as_ref()],
        program_id,
    );
    if vault_pda != *program_vault_account.key {
        msg!("Vault PDA mismatch");
        return Err(ProgramError::InvalidSeeds);
    }
    // Check that the program is the owner of the vault account (important for security)
    if program_vault_account.owner != program_id {
        msg!("Vault account not owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }


    // Derive and verify the state PDA
    let (state_pda, state_bump) = Pubkey::find_program_address(
        &[STATE_SEED, user_wallet_account.key.as_ref()],
        program_id,
    );
    if state_pda != *program_state_account.key {
        msg!("State PDA mismatch");
        return Err(ProgramError::InvalidSeeds);
    }
    // Check that the program is the owner of the state account
    if program_state_account.owner != program_id {
        msg!("State account not owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }


    // --- State Deserialization and Authorization Check ---

    // Deserialize the state to get the deposited amount and original owner
    let mut state = ProgramState::try_from_slice(&program_state_account.data.borrow())?;

    // Authorization check: Only the original depositor can withdraw
    if state.owner != *user_wallet_account.key {
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
            token_program.key,          // Token program ID
            program_vault_account.key,  // Source account (program's vault)
            user_token_account.key,     // Destination account (user's ATA)
            &vault_pda,                 // Authority of source account (the PDA)
            &[],                        // Signers (empty because invoke_signed uses PDA)
            amount,               // Amount to transfer
        )?,
        &[
            program_vault_account.clone(),
            user_token_account.clone(),
            token_program.clone(),
            // The PDA is the signer, but its AccountInfo is not passed here,
            // instead, the seeds are passed to invoke_signed.
        ],
        &[&[VAULT_SEED, token_mint_account.key.as_ref(), &[vault_bump]]],
    )?;

    // --- Update Program State ---
    state.balance = state.balance.checked_sub(amount).ok_or(ProgramError::ArithmeticOverflow)?;

    state.serialize(&mut &mut program_state_account.data.borrow_mut()[..])?;

    // Optionally close the state account if balance is 0 and it's rent-exempt.
    // For simplicity, we'll leave it open for now.
    // However, if the account's SOL balance drops below rent-exempt, it could be reclaimed.
    // The user would need to pay rent on this account for it to persist empty.
    // Alternatively, the program could transfer its rent back to the user and close it.

    msg!("Withdrawal successful!");

    Ok(())
}