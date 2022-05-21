use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum WPoktError {
    #[error("Underflow.")]
    Underflow,
    #[error("Overflow.")]
    Overflow,
    #[error("Account is not owned by WPokt.")]
    AccountNotOwnedByWPokt,
    #[error("Already set.")]
    AlreadySet,
    #[error("Invalid Caller.")]
    InvalidCaller,
}

impl From<WPoktError> for ProgramError {
    fn from(e: WPoktError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
