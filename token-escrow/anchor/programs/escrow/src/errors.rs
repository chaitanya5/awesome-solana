use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowErrorCode {
    #[msg("Account Not Mutable")]
    AccountNotMutable,

    #[msg("Insufficient token balance in maker's account")]
    InsufficientMakerBalance,

    #[msg("Insufficient token balance in taker's account")]
    InsufficientTakerBalance,

    #[msg("Invalid token mint - must be different from offered token")]
    InvalidTokenMint,

    #[msg("Amount must be greater than zero")]
    InvalidAmount,

    #[msg("Token transfer failed")]
    TokenTransferFailed,

    #[msg("Failed to withdraw tokens from vault")]
    FailedVaultWithdrawal,

    #[msg("Failed to close vault account")]
    FailedVaultClosure,

    #[msg("Failed to refund tokens from vault")]
    FailedRefundTransfer,

    #[msg("Failed to close vault during refund")]
    FailedRefundClosure,
}