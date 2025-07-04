use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Account not mutable")]
    AccountNotMutable,
    #[msg("Vault is already initialized")]
    AlreadyInitialized,
    #[msg("Invalid Vault account PDA")]
    InvalidVaultAuthority,
    #[msg("Token Transfer Failed")]
    TokenTransferFailed,
    #[msg("Invalid User account PDA")]
    InvalidUserAccount,
    #[msg("Invalid token mint")]
    InvalidMint,
    #[msg("Invalid token account owner")]
    InvalidOwner,
    #[msg("Source and destination mint mismatch")]
    MintMismatch,
    #[msg("Zero Deposit Amount")]
    ZeroAmount,
    #[msg("Insufficient user balance")]
    InsufficientFunds,
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Account not rent exempt")]
    NotRentExempt,
    #[msg("Invalid instruction")]
    InvalidInstruction,
    #[msg("Missing required signer")]
    NotSigner,
    #[msg("General failure")]
    GenericError,
}

