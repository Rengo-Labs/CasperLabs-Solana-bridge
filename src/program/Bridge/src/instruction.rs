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
    /// Accounts expected
    /// 0. `[]` The account of person initializing bridge - the 'owner'.
    /// 1. `[writeable]` The account used as global storage of Bridge program
    /// 2. `[writeable]` The account used as 'token_list' dictionary
    /// 3. `[]` the token mint account found at 'token_index' in token_list dictionary
    /// 4. `[writable]` the token account of token sender
    /// 5. `[writeable]` the CalculateFeeResult account
    /// 6. `[writable]` the Token account of Bridge
    /// 7. `[signer]` the sender token account's owner
    TransferRequest {
        token_index: u64,
        to: Pubkey,
        amount: u64,
        chain_id: u64,
    },
    /// Accounts expected
    /// 0. `[writeable]` The account used as global storage of Bridge program
    /// 1. `[writeable]` The account used as 'claimed' dictionary
    /// 2. `[writeable]` The account used as 'token_list' dictionary
    /// 3. `[writable]` The account used as 'daily_token_claims' dictionary
    /// 4. `[signer]` The signatory account.
    /// 5. `[writeable]` The token mint account for this token data's mint.
    /// 6. `[signer, writeable]` Bridge's associated token account for this mint.
    /// 7. `[signer, writeable]` receiver's associated token account for this mint.
    /// 8. `[signer]` The bridge PDA account
    TransferReceipt {
        token_index: u64,
        from: Pubkey,
        to: Pubkey,
        amount: u64,
        chain_id: u64,
        index: u64,
        signature_account: Pubkey,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[writeable]` The account used as global storage of bridge
    UpdateVerifyAddressOnlyOwner {
        verify_address: Pubkey,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[]` The account used as global storage of bridge
    /// 2. `[writable]` The account used as 'token_list' dictionary
    UpdateTokenLimitOnlyOwner {
        token_index: u64,
        limit: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[]` The account used as global storage of bridge
    /// 2. `[writable]` The account used as 'token_list' dictionary
    SetTokenLimitTimeOnlyOwner {
        token_index: u64,
        timestamp: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[writable]` The account used as global storage of bridge
    UpdateStableFeeOnlyOwner {
        new_stable_fee: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[]` The account used as global storage of bridge
    /// 2. `[ writable]` The account used as 'token_list' dictionary
    UpdateTokenFeeOnlyOwner {
        index: u64,
        new_token_fee: u64,
    },
    /// Accounts expected
    /// 0. `[]` The account used as global storage of bridge
    /// 1. `[]` The account used as 'token_list' dictionary
    UpdateFees {
        token_index: u64,
    },
    /// 0. `[signer, writeable]` Owner Account
    /// 1. `[writeable]` Owner Token Account
    /// 2. `[]` Bridge Account
    /// 3. `[signer]` Bridge Pda Account
    /// 4. `[writeable]` Bridge Token Account, To be created in the same transaction as WithdrawFees, authority set to Bridge PDA Account
    /// 5. `[writeable]` Mint Account at token_index
    /// 6. `[writeable]` Token List Account
    WithdrawFeesOnlyOwner {
        index: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[]` The account used as global storage of bridge
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
    /// 1. `[]` The account used as global storage of bridge
    /// 2. `[writable]` The account used as 'token_list' dictionary
    PauseTokenOnlyOwner {
        token_index: u64,
    },
    /// Accounts expected
    /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
    /// 1. `[]` The account used as global storage of bridge
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
    /// Accounts expected:
    /// 0. `[signer]` The program owner's account.
    /// 1. `[writeable]` The account used as WPokt's global state
    RenounceOwnership,
    /// Accounts expected:
    /// 0. `[signer]` The program owner's account.
    /// 1. `[writeable]` The account used as WPokt's global state
    TransferOwnership {
        new_owner: Pubkey,
    },
    CreateClaimedDictionaryPdaAccount {
        index: u64,
        chain_id: u64,
    },
    CreateDailyTokenClaimsDictionaryPdaAccount {
        token_index: u64,
    },
}
