use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum WPOKTError {
    #[error("wPOKT:AUTH_EXPIRED")]
    AuthExpired,
    #[error("wPOKT:AUTH_NOT_YET_VALID")]
    AuthNotYetValid,
    #[error("wPOKT:AUTH_ALREADY_USED")]
    AuthAlreadyUsed,
    #[error("wPOKT:INVALID_MINTER")]
    InvalidMinter,
}

impl From<WPOKTError> for ProgramError {
    fn from(e: WPOKTError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
