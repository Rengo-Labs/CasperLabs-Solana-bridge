use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::*;

#[derive(Default, Debug, Clone)]
pub struct WPOKT {
    pub is_initialized: bool,
    pub minter: Pubkey,
    pub mint: Pubkey,
}

impl Sealed for WPOKT {}
impl IsInitialized for WPOKT {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for WPOKT {
    const LEN: usize = 1 + 2 * 32;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, WPOKT::LEN];
        let (is_initialized_src, minter_src, mint_src) = array_refs![src, 1, 32, 32];
        let is_initialized = match is_initialized_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let minter: Pubkey = Pubkey::new_from_array(*minter_src);
        let mint: Pubkey = Pubkey::new_from_array(*mint_src);

        Ok(Self {
            is_initialized,
            minter,
            mint,
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, WPOKT::LEN];
        let (_is_initialized, _minter, _mint) = mut_array_refs![dst, 1, 32, 32];
        let WPOKT {
            is_initialized,
            minter,
            mint,
        } = self;
        _is_initialized[0] = *is_initialized as u8;
        _minter.copy_from_slice(minter.as_ref());
        _mint.copy_from_slice(mint.as_ref());
    }
}

#[derive(Default, Debug, Clone)]
pub struct NoncesDictionary {
    pub nonce: u64,
}
impl NoncesDictionary {
    pub fn generate_pda_key(program_id: Pubkey, address: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[address.as_ref()], &program_id)
    }
}

impl Sealed for NoncesDictionary {}
impl Pack for NoncesDictionary {
    const LEN: usize = 8;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, NoncesDictionary::LEN];
        Ok(Self {
            nonce: u64::from_le_bytes(*src),
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, NoncesDictionary::LEN];
        let NoncesDictionary{
            nonce
        } = self;
        *dst = nonce.to_le_bytes();
    }
}

#[derive(Default, Debug, Clone)]
pub struct AuthorizationStateDictionary {
    authorization: bool,
}
impl AuthorizationStateDictionary {
    pub fn generate_pda_key(program_id: Pubkey, from: Pubkey, nonce: String) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[from.as_ref(), nonce.as_ref()], &program_id)
    }
}

impl Sealed for AuthorizationStateDictionary {}
impl Pack for AuthorizationStateDictionary {
    const LEN: usize = 1;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, AuthorizationStateDictionary::LEN];
        Ok(Self {
            authorization: match src {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            },
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, AuthorizationStateDictionary::LEN];
        let AuthorizationStateDictionary { authorization } = self;
        dst[0] = *authorization as u8;
    }
}
