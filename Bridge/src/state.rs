use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::{IsInitialized, Pack, Sealed}, pubkey::Pubkey, program_error::ProgramError};
use std::collections::BTreeMap;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

const INITIALIZED_BYTES: usize = 1;
const TRACKING_CHUNK_LENGTH: usize = 4;
const TRACKING_CHUNK_BYTES: usize = 10235;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ConstructorArgs {
    pub w_pokt_address: Pubkey,
    pub verify_address: Pubkey,
    pub chain_id: u64,
    pub stable_fee: u64,
}

// #[repr(packed)]
//#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Bridge {
    // total size = (8*6) + (32*6) + 1 = 241 bytes
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

impl Sealed for Bridge {}

impl Pack for Bridge {

    const LEN: usize = 241;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Bridge::LEN];

        let (
            is_initialized,
            owner,
            fee_update_duration,
            verify_address,
            current_index,
            chain_id,
            stable_fee_update_time,
            stable_fee,
            new_stable_fee,
            claimed_dictionary,
            token_list_dictionary,
            daily_token_claims_dictionary,
            token_added_dictionary,
        ) = array_refs![src, 1, 32, 8, 32, 8, 8, 8, 8, 8, 32 ,32 ,32, 32];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Bridge {
            is_initialized,
            owner: Pubkey::new_from_array(*owner),
            fee_update_duration: u64::from_le_bytes(*fee_update_duration),
            verify_address: Pubkey::new_from_array(*verify_address),
            current_index: u64::from_le_bytes(*current_index),
            chain_id: u64::from_le_bytes(*chain_id),
            stable_fee_update_time: u64::from_le_bytes(*stable_fee_update_time),
            stable_fee: u64::from_le_bytes(*stable_fee),
            new_stable_fee: u64::from_le_bytes(*new_stable_fee),
            claimed_dictionary: Pubkey::new_from_array(*claimed_dictionary),
            token_list_dictionary: Pubkey::new_from_array(*token_list_dictionary),
            daily_token_claims_dictionary: Pubkey::new_from_array(*daily_token_claims_dictionary),
            token_added_dictionary: Pubkey::new_from_array(*token_added_dictionary)

        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {

        let dst = array_mut_ref![dst, 0, Bridge::LEN];

        let (
            is_initialized_dst,
            owner_dst,
            fee_update_duration_dst,
            verify_address_dst,
            current_index_dst,
            chain_id_dst,
            stable_fee_update_time_dst,
            stable_fee_dst,
            new_stable_fee_dst,
            claimed_dictionary_dst,
            token_list_dictionary_dst,
            daily_token_claims_dictionary_dst,
            token_added_dictionary_dst,
        ) = mut_array_refs![dst, 1, 32, 8, 32, 8, 8, 8, 8, 8, 32 ,32 ,32, 32];

        let Bridge {
            is_initialized,
            owner,
            fee_update_duration,
            verify_address,
            current_index,
            chain_id,
            stable_fee_update_time,
            stable_fee,
            new_stable_fee,
            claimed_dictionary,
            token_list_dictionary,
            daily_token_claims_dictionary,
            token_added_dictionary,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        owner_dst.copy_from_slice(owner.as_ref());
        *fee_update_duration_dst = fee_update_duration.to_le_bytes();
        verify_address_dst.copy_from_slice(verify_address.as_ref());
        *current_index_dst = current_index.to_le_bytes();
        *chain_id_dst = chain_id.to_le_bytes();
        *stable_fee_update_time_dst = stable_fee_update_time.to_le_bytes();
        *stable_fee_dst = stable_fee.to_le_bytes();
        *new_stable_fee_dst = new_stable_fee.to_le_bytes();
        claimed_dictionary_dst.copy_from_slice(claimed_dictionary.as_ref());
        token_list_dictionary_dst.copy_from_slice(token_list_dictionary.as_ref());
        daily_token_claims_dictionary_dst.copy_from_slice(daily_token_claims_dictionary.as_ref());
        token_added_dictionary_dst.copy_from_slice(token_added_dictionary.as_ref());
    }
}

impl IsInitialized for Bridge {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

//#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ClaimedDictionary {
    // total size = 10240 bytes = 10Mb
    // spans entire data field of account
    pub is_initialized: bool, // 1 byte
    pub claimed_dictionary: BTreeMap<u64, BTreeMap<u64, bool>>,
}
impl Sealed for ClaimedDictionary {}

impl Pack for ClaimedDictionary {

    const LEN: usize = 10240;       // total size 10MB - 10240 Bytes

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {

        let src = array_ref![src, 0, ClaimedDictionary::LEN];
        let (is_initialized_src, hmap_len, hmap_src) = array_refs![
            src,
            INITIALIZED_BYTES,
            TRACKING_CHUNK_LENGTH,
            TRACKING_CHUNK_BYTES
        ];
        let is_initialized = match is_initialized_src {
            [0] => false,
            [1] => true,
            _ => panic!(),
        };
        let mut map_dser = BTreeMap::<u64, BTreeMap<u64, bool>>::new();
        let hmap_length = u32::from_le_bytes(*hmap_len) as usize;
        if hmap_length > 0 {
            map_dser = BTreeMap::<u64, BTreeMap<u64, bool>>::try_from_slice(&hmap_src[0..hmap_length]).unwrap()
        }
        Ok (Self {
            is_initialized,
            claimed_dictionary: map_dser,
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {

        let dst = array_mut_ref![dst, 0, ClaimedDictionary::LEN];

        let (is_initialized_dst, hmap_len, hmap_dst) = mut_array_refs![
            dst,
            INITIALIZED_BYTES,
            TRACKING_CHUNK_LENGTH,
            TRACKING_CHUNK_BYTES
        ];

        is_initialized_dst[0] = self.is_initialized as u8;
        let data_ser = self.claimed_dictionary.try_to_vec().unwrap();
        //hmap_len[..].copy_from_slice(&transform_u32_to_array_of_u8(data_ser.len() as u32));
        hmap_len[..].copy_from_slice(&(data_ser.len() as u32).to_le_bytes());
        hmap_dst[..data_ser.len()].copy_from_slice(&data_ser);
    }
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
