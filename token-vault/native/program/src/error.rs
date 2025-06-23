use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("Vault is already initialized")]
    AlreadyInitialized,
    #[error("Invalid Vault account PDA")]
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