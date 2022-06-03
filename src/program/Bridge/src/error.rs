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
    #[error("Token already paused.")]
    TokenAlreadyPaused,
    #[error("Token already unpaused.")]
    TokenAlreadyUnaused,
    #[error("Cannot request to same chain.")]
    RequestToSameChain,
    #[error("Nothing to withdraw.")]
    NothingToWithdraw,
    #[error("Token doesn't exist.")]
    NonExistantToken,
    #[error("Token already claimed.")]
    AlreadyClaimed,
    #[error("Cannot claim above daily limit.")]
    ClaimAboveDailyLimit,
    #[error("Bridge Token account key mismatch.")]
    TokenAccountKeyMismatch,
}

impl From<BridgeError> for ProgramError {
    fn from(e: BridgeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
