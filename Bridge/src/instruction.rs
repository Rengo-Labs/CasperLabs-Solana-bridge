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
    TransferRequest {
        token_index: u64,
        to: Pubkey,
        amount: u64,
        chain_id: u64,
    },
    TransferReceipt {
        token_index: u64,
        from: Pubkey,
        to: Pubkey,
        amount: u64,
        chain_id: u64,
        index: u64,
        signature: Vec<u8>,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[signer - writeable]` The account used as global storage of bridge
    UpdateVerifyAddressOnlyOwner {
        verify_address: Pubkey,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[signer]` The account used as global storage of bridge
    /// 2. `[signer - writable]` The account used as 'token_list' dictionary
    UpdateTokenLimitOnlyOwner {
        token_index: u64,
        limit: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[signer]` The account used as global storage of bridge
    /// 2. `[signer - writable]` The account used as 'token_list' dictionary
    SetTokenLimitTimeOnlyOwner {
        token_index: u64,
        timestamp: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[signer - writable]` The account used as global storage of bridge
    UpdateStableFeeOnlyOwner {
        new_stable_fee: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[signer]` The account used as global storage of bridge
    /// 2. `[signer - writable]` The account used as 'token_list' dictionary
    UpdateTokenFeeOnlyOwner {
        index: u64,
        new_token_fee: u64,
    },
    /// Accounts expected
    /// 0. `[]` The account used as global storage of bridge
    /// 1. `[writable]` The account used as 'token_list' dictionary
    UpdateFees {
        token_index: u64,
    },
    WithdrawFeesOnlyOwner {
        index: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[signer]` The account used as global storage of bridge
    /// 2. `[writable]` The account used as 'token_list' dictionary
    /// 3. `[writable]` The account used as 'token_added' dictionary
    AddTokenOnlyOwner {
        index: u64,
        token_address: Pubkey,
        fee: u64,
        limit: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[signer]` The account used as global storage of bridge
    /// 2. `[writable]` The account used as 'token_list' dictionary
    PauseTokenOnlyOwner {
        token_index: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[signer]` The account used as global storage of bridge
    /// 2. `[writable]` The account used as 'token_list' dictionary
    /// 3. `[writable]` The account used as 'daily_token_claims' dictionary
    ///
    UnpauseTokenOnlyOwner {
        token_index: u64,
    },
    /// Accounts expected
    /// 0. `[]` The account used as global storage of bridge
    /// 1. `[]` The account used as 'token_list' dictionary
    /// 2. `[]` The account used as 'calculate_fee_result' account
    CalculateFee {
        token_index: u64,
        amount: u64,
    },
    Verify {
        token_index: u64,
        from: Pubkey,
        to: Pubkey,
        amount: u64,
        chain_id: u64,
        index: u64,
        signature: Vec<u8>,
    },
}
