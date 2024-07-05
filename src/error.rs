use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum TokenError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Not rent exempt")]
    NotRentExempt,
    #[error("Expected amount mismatch")]
    ExpectedAmountMismatch,
    #[error("Amount overflow")]
    AmountOverflow,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Account frozen")]
    AccountFrozen,
    #[error("Invalid authority")]
    InvalidAuthority,
    #[error("Invalid program address")]
    InvalidProgramAddress,
    #[error("Already initialized")]
    AlreadyInitialized,
    #[error("Vesting not started")]
    VestingNotStarted,
    #[error("No tokens to release")]
    NoTokensToRelease,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
