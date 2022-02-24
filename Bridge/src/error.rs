use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum BridgeError {
    #[error("Underflow.")]
    Underflow,
    #[error("Overflow.")]
    Overflow,
    #[error("Account is not owned by Bridge.")]
    AccountNotOwnedByBridge,
    #[error("Given key doesn't map to any value.")]
    MapKeyNotFound,
    #[error("Token already added.")]
    TokenAlreadyAdded,
}

impl From<BridgeError> for ProgramError {
    fn from(e: BridgeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
