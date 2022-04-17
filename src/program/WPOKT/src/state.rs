use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::collections::BTreeMap;

const INITIALIZED_BYTES: usize = 1;
const TRACKING_CHUNK_LENGTH: usize = 4;
const TRACKING_CHUNK_BYTES: usize = 10235;

#[derive(Default, Debug, Clone)]
pub struct WPOKT {
    pub is_initialized: bool,
    pub minter: Pubkey,
    pub mint: Pubkey,
    pub nonces_dict: Pubkey,
    pub authorization_state_dict: Pubkey,
}
impl Sealed for WPOKT {}
impl IsInitialized for WPOKT {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for WPOKT {
    const LEN: usize = 1 + 4 * 32;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, WPOKT::LEN];
        let (
            is_initialized_src,
            minter_src,
            mint_src,
            nonces_dict_src,
            authorization_state_dict_src,
        ) = array_refs![src, 1, 32, 32, 32, 32];
        let is_initialized = match is_initialized_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let minter: Pubkey = Pubkey::new_from_array(*minter_src);
        let mint: Pubkey = Pubkey::new_from_array(*mint_src);
        let nonces_dict: Pubkey = Pubkey::new_from_array(*nonces_dict_src);
        let authorization_state_dict: Pubkey =
            Pubkey::new_from_array(*authorization_state_dict_src);

        Ok(Self {
            is_initialized,
            minter,
            mint,
            nonces_dict,
            authorization_state_dict,
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, WPOKT::LEN];
        let (_is_initialized, _minter, _mint, _nonces_dict, _authorization_state_dict) =
            mut_array_refs![dst, 1, 32, 32, 32, 32];
        let WPOKT {
            is_initialized,
            minter,
            mint,
            nonces_dict,
            authorization_state_dict,
        } = self;
        _is_initialized[0] = *is_initialized as u8;
        _minter.copy_from_slice(minter.as_ref());
        _mint.copy_from_slice(mint.as_ref());
        _nonces_dict.copy_from_slice(nonces_dict.as_ref());
        _authorization_state_dict.copy_from_slice(authorization_state_dict.as_ref());
    }
}

pub struct NoncesDictionary {
    pub is_initialized: bool,
    pub nonces_dictionary: BTreeMap<Pubkey, u64>,
}

impl IsInitialized for NoncesDictionary {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Sealed for NoncesDictionary {}
impl Pack for NoncesDictionary {
    const LEN: usize = 10240; // total size 10MB - 10240 Bytes

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, NoncesDictionary::LEN];
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
        let mut map_dser = BTreeMap::<Pubkey, u64>::new();
        let hmap_length = u32::from_le_bytes(*hmap_len) as usize;
        if hmap_length > 0 {
            map_dser = BTreeMap::<Pubkey, u64>::try_from_slice(&hmap_src[0..hmap_length]).unwrap()
        }
        Ok(Self {
            is_initialized,
            nonces_dictionary: map_dser,
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, NoncesDictionary::LEN];

        let (is_initialized_dst, hmap_len, hmap_dst) = mut_array_refs![
            dst,
            INITIALIZED_BYTES,
            TRACKING_CHUNK_LENGTH,
            TRACKING_CHUNK_BYTES
        ];

        is_initialized_dst[0] = self.is_initialized as u8;
        let data_ser = self.nonces_dictionary.try_to_vec().unwrap();
        //hmap_len[..].copy_from_slice(&transform_u32_to_array_of_u8(data_ser.len() as u32));
        hmap_len[..].copy_from_slice(&(data_ser.len() as u32).to_le_bytes());
        hmap_dst[..data_ser.len()].copy_from_slice(&data_ser);
    }
}

#[derive(Default, Debug, Clone)]
pub struct AuthorizationStateDictionary {
    // total size = 10240 bytes = 10Mb
    // spans entire data field of account
    pub is_initialized: bool, // 1 byte
    pub authorization_state_dictionary: BTreeMap<String, bool>,
}
impl AuthorizationStateDictionary {
    pub fn generate_key(x: Pubkey, y: String) -> String {
        format!("{}{}", x, y)
    }
}
impl Sealed for AuthorizationStateDictionary {}
impl Pack for AuthorizationStateDictionary {
    const LEN: usize = 10240; // total size 10MB - 10240 Bytes

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, AuthorizationStateDictionary::LEN];
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
        let mut map_dser = BTreeMap::<String, bool>::new();
        let hmap_length = u32::from_le_bytes(*hmap_len) as usize;
        if hmap_length > 0 {
            map_dser = BTreeMap::<String, bool>::try_from_slice(&hmap_src[0..hmap_length]).unwrap()
        }
        Ok(Self {
            is_initialized,
            authorization_state_dictionary: map_dser,
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, AuthorizationStateDictionary::LEN];

        let (is_initialized_dst, hmap_len, hmap_dst) = mut_array_refs![
            dst,
            INITIALIZED_BYTES,
            TRACKING_CHUNK_LENGTH,
            TRACKING_CHUNK_BYTES
        ];

        is_initialized_dst[0] = self.is_initialized as u8;
        let data_ser = self.authorization_state_dictionary.try_to_vec().unwrap();
        //hmap_len[..].copy_from_slice(&transform_u32_to_array_of_u8(data_ser.len() as u32));
        hmap_len[..].copy_from_slice(&(data_ser.len() as u32).to_le_bytes());
        hmap_dst[..data_ser.len()].copy_from_slice(&data_ser);
    }
}
impl IsInitialized for AuthorizationStateDictionary {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
