use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

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
