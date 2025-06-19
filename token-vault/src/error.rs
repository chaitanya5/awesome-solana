use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenVaultError {
    // Invalid Instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    
    // Incorrect Token Vault Account
    #[error("Incorrect Token Vault Account")]
    IncorrectTokenVaultAccount,

    // Incorrect User State Account
    #[error("Incorrect State Account")]
    IncorrectUserStateAccount,
    
    #[error("No Tokens Deposited")]
    NoTokensDeposited,

    #[error("Zero Token Deposit")]
    ZeroTokenDeposited,

    #[error("Zero Token Withdraw")]
    ZeroTokenWithdraw,

    // Unauthorized Withdraw
    #[error("Unauthorized Withdraw")]
    UnauthorizedWithdraw,


    #[error("Not Enough Balance")]
    NotEnoughBalance,
}

impl From<TokenVaultError> for ProgramError {
    fn from(value: TokenVaultError) -> Self {
        ProgramError::Custom(value as u32)
    }
}
