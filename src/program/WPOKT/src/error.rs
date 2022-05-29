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
    #[error("wPOKT:INITIAL_MINTER_PUBKEY_MISMATCH")]
    InitialMinterPubkeyMismatch,
    #[error("wPOKT:WPOKT_PDA_MISMATCH")]
    WPOKTPdaMismatch,
    #[error("wPOKT:NEW_MINTER_PUBKEY_MISMATCH")]
    NewMinterPubkeyMismatch,
    #[error("wPOKT:DELEGATE_SPENDER_MISMATCH")]
    DelegateSpenderMismatch,
    #[error("wPOKT:TOKEN_AUTHORITY_MISMATCH")]
    TokenAuthorityMismatch,
    #[error("wPOKT:NONCES_DICTIONARY_ITEM_KEY_MISMATCH")]
    NoncesDictionaryItemKeyMismatch,
    #[error("wPOKT:NONCES_DICTIONARY_OWNER_KEY_MISMATCH")]
    NoncesDictionaryItemOwnerMismatch,
    #[error("wPOKT:AUTH_STATE_DICTIONARY_ITEM_KEY_MISMATCH")]
    AuthStateDictionaryItemKeyMismatch,
    #[error("wPOKT:AUTH_STATE_DICTIONARY_FROM_KEY_MISMATCH")]
    AuthStateDictionaryFromKeyMismatch,
    #[error("wPOKT:AUTH_STATE_DICTIONARY_NONCE_MISMATCH")]
    AuthStateDictionaryNonceMismatch,
}

impl From<WPOKTError> for ProgramError {
    fn from(e: WPOKTError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
