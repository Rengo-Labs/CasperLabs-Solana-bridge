use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

const INITIALIZED_BYTES: usize = 1;
const TRACKING_CHUNK_LENGTH: usize = 4;
const TRACKING_CHUNK_BYTES: usize = 10235;

pub trait GeneratePdaKey {
    fn generate_pda_key(program_id: &Pubkey, seeds: &Vec<&[u8]>) -> (Pubkey, u8);
}

#[derive(Debug, Default, Clone)]
pub struct Bridge {
    // total size = (8*6) + (32*2) + 1
    pub is_initialized: bool,        // 1 byte
    pub owner: Pubkey,               // 32 bytes
    pub fee_update_duration: u64,    //8 bytes
    pub verify_address: Pubkey,      //32 bytes
    pub current_index: u64,          //8 bytes
    pub chain_id: u64,               //8 bytes
    pub stable_fee_update_time: u64, //8 bytes
    pub stable_fee: u64,             //8 bytes
    pub new_stable_fee: u64,         //8 bytes
}
impl Sealed for Bridge {}
impl Pack for Bridge {
    const LEN: usize = 241 - 32 * 4;

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
        ) = array_refs![src, 1, 32, 8, 32, 8, 8, 8, 8, 8];
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
        ) = mut_array_refs![dst, 1, 32, 8, 32, 8, 8, 8, 8, 8];

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
    }
}
impl IsInitialized for Bridge {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl GeneratePdaKey for Bridge {
    /// seeds are unused
    fn generate_pda_key(program_id: &Pubkey, _seeds: &Vec<&[u8]>) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"bridge", b"signature_account"], program_id)
    }
}

#[derive(Default, Debug, Clone)]
pub struct ClaimedDictionary {
    claimed: bool,
}
impl GeneratePdaKey for ClaimedDictionary {
    /// seeds[0] chain_ud: u64
    /// seeds[1] index: u64
    fn generate_pda_key(program_id: &Pubkey, seeds: &Vec<&[u8]>) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[seeds[0], seeds[1], b"bridge", b"claimed_dictionary_key"],
            program_id,
        )
    }
}
impl ClaimedDictionary {
    pub fn generate_pda_seeds_vec(chain_id: u64, index: u64) -> Vec<&'static [u8]> {
        vec![
            chain_id.to_le_bytes().as_ref(),
            index.to_le_bytes().as_ref(),
        ]
    }
}

impl Sealed for ClaimedDictionary {}
impl Pack for ClaimedDictionary {
    const LEN: usize = 1;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, ClaimedDictionary::LEN];
        let claimed = match src {
            [0] => false,
            [1] => true,
            _ => panic!(),
        };
        Ok(Self { claimed })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, ClaimedDictionary::LEN];
        dst[0] = self.claimed as u8;
    }
}

#[derive(Default, Debug, Clone)]
pub struct TokenListDictionary {
    pub is_initialized: bool,  // 1B
    pub token_address: Pubkey, // 32B
    pub exists: bool,          // 1B
    pub paused: bool,          // 1B
    // total fees collected
    pub total_fees_collected: u64, //8B
    // current fee
    pub fee: u64, //8B
    // fee update time
    pub fee_update_time: u64, //8B
    // new fee
    pub new_fee: u64, //8B
    // daily limit
    pub limit: u64, //8B
    // daily limit time
    pub limit_timestamp: u64, //8B
}
impl GeneratePdaKey for TokenListDictionary {
    /// seeds[0] index: u64
    fn generate_pda_key(program_id: &Pubkey, seeds: &Vec<&[u8]>) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[seeds[0], b"bridge", b"token_list_dictionary_key"],
            program_id,
        )
    }
}
impl TokenListDictionary {
    pub fn generate_pda_seeds_vec(index: u64) -> Vec<&'static [u8]> {
        vec![index.to_le_bytes().as_ref()]
    }
}

impl Sealed for TokenListDictionary {}
impl Pack for TokenListDictionary {
    const LEN: usize = 1 + 32 + 1 + 1 + 8 + 8 + 8 + 8 + 8 + 8;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, TokenListDictionary::LEN];
        let (
            is_initialized_src,
            token_address_src,
            exists_src,
            paused_src,
            total_fees_collected_src,
            fee_src,
            fee_update_time_src,
            new_fee_src,
            limit_src,
            limit_timestamp_src,
        ) = array_refs![src, 1, 32, 1, 1, 8, 8, 8, 8, 8, 8];
        let is_initialized = match is_initialized_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let token_address = Pubkey::new_from_array(*token_address_src);
        let exists = match exists_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let paused = match paused_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let total_fees_collected = u64::from_le_bytes(*total_fees_collected_src);
        let fee = u64::from_le_bytes(*fee_src);
        let fee_update_time = u64::from_le_bytes(*fee_update_time_src);
        let new_fee = u64::from_le_bytes(*new_fee_src);
        let limit = u64::from_le_bytes(*limit_src);
        let limit_timestamp = u64::from_le_bytes(*limit_timestamp_src);
        Ok(Self {
            is_initialized,
            token_address,
            exists,
            paused,
            total_fees_collected,
            fee,
            fee_update_time,
            new_fee,
            limit,
            limit_timestamp,
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, TokenListDictionary::LEN];

        let (
            is_initialized_dst,
            token_address_dst,
            exists_dst,
            paused_dst,
            total_fees_collected_dst,
            fee_dst,
            fee_update_time_dst,
            new_fee_dst,
            limit_dst,
            limit_timestamp_dst,
        ) = mut_array_refs![dst, 1, 32, 1, 1, 8, 8, 8, 8, 8, 8];

        is_initialized_dst[0] = self.is_initialized as u8;
        token_address_dst.copy_from_slice(self.token_address.as_ref());
        exists_dst[0] = self.exists as u8;
        paused_dst[0] = self.paused as u8;
        *total_fees_collected_dst = self.total_fees_collected.to_le_bytes();
        *fee_dst = self.fee.to_le_bytes();
        *fee_update_time_dst = self.fee_update_time.to_le_bytes();
        *new_fee_dst = self.new_fee.to_le_bytes();
        *limit_dst = self.limit.to_le_bytes();
        *limit_timestamp_dst = self.limit_timestamp.to_le_bytes();
    }
}

#[derive(Default, Debug, Clone)]
pub struct DailyTokenClaimsDictionary {
    pub daily_token_claims: u64,
}
impl Sealed for DailyTokenClaimsDictionary {}
impl Pack for DailyTokenClaimsDictionary {
    const LEN: usize = 8;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, DailyTokenClaimsDictionary::LEN];
        Ok(Self {
            daily_token_claims: u64::from_le_bytes(*src),
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, DailyTokenClaimsDictionary::LEN];
        *dst = self.daily_token_claims.to_le_bytes();
    }
}

impl GeneratePdaKey for DailyTokenClaimsDictionary {
    /// seeds[0] index: u64
    fn generate_pda_key(program_id: &Pubkey, seeds: &Vec<&[u8]>) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[seeds[0], b"bridge", b"daily_token_claims_dictionary_key"],
            program_id,
        )
    }
}
impl DailyTokenClaimsDictionary {
    pub fn generate_pda_seeds_vec(index: u64) -> Vec<&'static [u8]> {
        vec![index.to_le_bytes().as_ref()]
    }
}

#[derive(Default, Debug, Clone)]
pub struct TokenAddedDictionary {
    pub token_added: bool,
}
impl Sealed for TokenAddedDictionary {}
impl Pack for TokenAddedDictionary {
    const LEN: usize = 1;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, TokenAddedDictionary::LEN];
        let token_added = match src {
            [0] => false,
            [1] => true,
            _ => panic!(),
        };
        Ok(Self { token_added })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, TokenAddedDictionary::LEN];
        dst[0] = self.token_added as u8;
    }
}
impl GeneratePdaKey for TokenAddedDictionary {
    /// seeds[0] index: u64
    fn generate_pda_key(program_id: &Pubkey, seeds: &Vec<&[u8]>) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[seeds[0], b"bridge", b"token_added_dictionary"],
            program_id,
        )
    }
}
impl TokenAddedDictionary {
    pub fn generate_pda_seeds_vec(token_mint_address: &Pubkey) -> Vec<&'static [u8]> {
        vec![token_mint_address.to_bytes().as_ref()]
    }
}

#[derive(Default, Debug, Clone)]
pub struct CalcuateFeeResult {
    pub fee: u64,
}
impl Sealed for CalcuateFeeResult {}
impl Pack for CalcuateFeeResult {
    const LEN: usize = 8;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src: &[u8; 8] = array_ref![src, 0, CalcuateFeeResult::LEN];
        let fee: u64 = u64::from_le_bytes(*src);
        Ok(Self { fee })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let fee_dst: &mut [u8; 8] = array_mut_ref![dst, 0, CalcuateFeeResult::LEN];

        let CalcuateFeeResult { fee } = self;

        *fee_dst = fee.to_le_bytes();
    }
}
