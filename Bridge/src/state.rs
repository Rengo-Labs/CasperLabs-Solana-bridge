use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};
use std::collections::BTreeMap;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ConstructorArgs {
    pub w_pokt_address: Pubkey,
    pub verify_address: Pubkey,
    pub chain_id: u64,
    pub stable_fee: u64,
}

// #[repr(packed)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Bridge {
    // total size = (8*6) + (32*6) = 241 bytes
    pub is_initialized: bool,                  // 1 byte
    pub owner: Pubkey,                         // 32 bytes
    pub fee_update_duration: u64,              //8 bytes
    pub verify_address: Pubkey,                //32 bytes
    pub current_index: u64,                    //8 bytes
    pub chain_id: u64,                         //8 bytes
    pub stable_fee_update_time: u64,           //8 bytes
    pub stable_fee: u64,                       //8 bytes
    pub new_stable_fee: u64,                   //8 bytes
    pub claimed_dictionary: Pubkey,            //32 bytes
    pub token_list_dictionary: Pubkey,         //32 bytes
    pub daily_token_claims_dictionary: Pubkey, //32 bytes
    pub token_added_dictionary: Pubkey,        //32 bytes
}

impl IsInitialized for Bridge {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ClaimedDictionary {
    // total size = 10240 bytes = 10Mb
    // spans entire data field of account
    pub is_initialized: bool, // 1 byte
    pub claimed_dictionary: BTreeMap<u64, BTreeMap<u64, bool>>,
}
impl IsInitialized for ClaimedDictionary {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TokenListDictionary {
    // total size = 10240 bytes = 10Mb
    // spans entire data field of account
    pub is_initialized: bool, // 1 byte
    pub token_list_dictionary: BTreeMap<u64, Vec<u8>>,
}
impl IsInitialized for TokenListDictionary {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DailyTokenClaimsDictionary {
    // total size = 10240 bytes = 10Mb
    // spans entire data field of account
    pub is_initialized: bool, // 1 byte
    pub daily_token_claims_dictionary: BTreeMap<u64, u64>,
}
impl IsInitialized for DailyTokenClaimsDictionary {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TokenAddedDictionary {
    // total size = 10240 bytes = 10Mb
    // spans entire data field of account
    pub is_initialized: bool, // 1 byte
    pub token_added_dictionary: BTreeMap<Pubkey, bool>,
}
impl IsInitialized for TokenAddedDictionary {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
