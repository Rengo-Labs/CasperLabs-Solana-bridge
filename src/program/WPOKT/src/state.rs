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
    pub owner: Pubkey,
    pub nonce: u64,
}

impl NoncesDictionary {
    pub fn generate_pda_key(program_id: Pubkey, address: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[address.as_ref(), b"WPOKT", b"nonces_dictionary_key"],
            &program_id,
        )
    }
}

impl Sealed for NoncesDictionary {}
impl Pack for NoncesDictionary {
    const LEN: usize = 32 + 8;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, NoncesDictionary::LEN];
        let (owner_src, nonce_src) = array_refs![src, 32, 8];
        Ok(Self {
            owner: Pubkey::new_from_array(*owner_src),
            nonce: u64::from_le_bytes(*nonce_src),
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, NoncesDictionary::LEN];
        let (owner_dst, nonce_dst) = mut_array_refs![dst, 32, 8];
        let NoncesDictionary { owner, nonce } = self;

        owner_dst.copy_from_slice(owner.as_ref());
        *nonce_dst = nonce.to_le_bytes();
    }
}

#[derive(Default, Debug, Clone)]
pub struct AuthorizationStateDictionary {
    pub from: Pubkey,
    pub nonce: [u8; 32],
    pub authorization: bool,
}

impl AuthorizationStateDictionary {
    pub fn generate_pda_key(program_id: Pubkey, from: Pubkey, nonce: [u8; 32]) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                from.as_ref(),
                nonce.as_ref(),
                b"WPOKT",
                b"authorization_dictionary_key",
            ],
            &program_id,
        )
    }
}

impl Sealed for AuthorizationStateDictionary {}
impl Pack for AuthorizationStateDictionary {
    const LEN: usize = 32 + 32 + 1;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, AuthorizationStateDictionary::LEN];
        let (from_src, nonce_src, authorization_src) = array_refs![src, 32, 32, 1];
        Ok(Self {
            from: Pubkey::new_from_array(*from_src),
            nonce: *nonce_src,
            authorization: match authorization_src {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            },
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, AuthorizationStateDictionary::LEN];
        let (from_dst, nonce_dst, authorization_dst) = mut_array_refs![dst, 32, 32, 1];

        let AuthorizationStateDictionary {
            from,
            nonce,
            authorization,
        } = self;
        from_dst.copy_from_slice(from.as_ref());
        nonce_dst.copy_from_slice(nonce.as_ref());
        authorization_dst[0] = *authorization as u8;
    }
}
