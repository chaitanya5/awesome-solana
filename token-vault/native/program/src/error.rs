use thiserror::Error;
use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError, msg,
    program_error::ProgramError, entrypoint::ProgramResult,
};
use solana_program::msg;
use solana_program::program_error::PrintProgramError;
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum VaultError {
    #[error("Vault is already initialized")]
    AlreadyInitialized,
    #[error("Invalid Vault account PDA")]
    InvalidVaultState,
    #[error("Invalid Vault authority PDA")]
    InvalidVaultAuthority,
    #[error("Invalid User account PDA")]
    InvalidUserAccount,
    #[error("Invalid token mint")]
    InvalidMint,
    #[error("Invalid token account owner")]
    InvalidOwner,
    #[error("Source and destination mint mismatch")]
    MintMismatch,
    #[error("Insufficient user balance")]
    InsufficientFunds,
    #[error("Overflow occurred")]
    Overflow,
    #[error("Account not rent exempt")]
    NotRentExempt,
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Missing required signer")]
    NotSigner,
    #[error("General failure")]
    GenericError,
}

impl From<VaultError> for ProgramError {
    fn from(e: VaultError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for VaultError {
    fn type_of() -> &'static str { "VaultError" }
}

impl PrintProgramError for VaultError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + num_traits::FromPrimitive,
    {
        match self {
            VaultError::AlreadyInitialized => msg!("Error: Vault already initialized"),
            VaultError::InvalidVaultState => msg!("Error: Vault state PDA mismatch"),
            VaultError::InvalidVaultAuthority => msg!("Error: Vault authority PDA mismatch"),
            VaultError::InvalidUserAccount => msg!("Error: User account PDA mismatch"),
            VaultError::InvalidMint => msg!("Error: Token mint mismatch"),
            VaultError::InvalidOwner => msg!("Error: Account owner invalid"),
            VaultError::MintMismatch => msg!("Error: Account mint mismatch"),
            VaultError::InsufficientFunds => msg!("Error: Insufficient funds in user account"),
            VaultError::Overflow => msg!("Error: Arithmetic overflow"),
            VaultError::NotRentExempt => msg!("Error: Account not rent exempt"),
            VaultError::InvalidInstruction => msg!("Error: Invalid instruction"),
            VaultError::NotSigner => msg!("Error: Missing required signer"),
            VaultError::GenericError => msg!("Error: Generic failure"),
        }
    }
}
