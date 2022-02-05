use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Bridge {
    fee_update_duration: u64,
    verify_address: Pubkey,
    current_index: u64,
    chain_id: u64,
    stable_fee_update_time: u64,
    stable_fee: u64,
    new_stable_fee: u64,
    claimed_dictionary_account: Pubkey,
    token_list_dictionary_account: Pubkey,
    daily_token_claims_dictionary_account: Pubkey,
    token_added_dictionary_account: Pubkey,
}
