use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum BridgeInstruction {
    /// Initialize storage accounts for Bridge
    ///
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[writeable]` The account used as global storage of bridge
    /// 2. `[writeable]` The account used as 'claimed' dictionary
    /// 3. `[writeable]` The account used as 'token_list' dictionary
    /// 4. `[writeable]` The account used as 'daily_token_claims' dictionary
    /// 5. `[writeable]` The account used as 'token_added' dictionary
    Construct {
        w_pokt_address: Pubkey,
        verify_address: Pubkey,
        chain_id: u64,
        stable_fee: u64,
    },
    TransferRequest,
    TransferReceipt,
    UpdateVerifyAddressOnlyOwner,
    UpdateTokenLimitOnlyOwner,
    SetTokenLimitTimeOnlyOwner,
    UpdateStableFeeOnlyOwner,
    UpdateTokenFeeOnlyOwner,
    UpdateFees,
    WithdrawFeesOnlyOwner,
    AddTokenOnlyOwner,
    PauseTokenOnlyOwner,
    UnpauseTokenOnlyOwner,
    CalculateFee,
    Verify,
}
