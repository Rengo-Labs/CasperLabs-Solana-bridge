use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
// use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[derive(Default, Debug, Clone)]
pub struct WPokt {
    pub is_initialized: bool,
    pub bridge_address: Pubkey,
    pub owner: Pubkey,
    pub mint: Pubkey,
}

impl Sealed for WPokt {}
impl Pack for WPokt {
    const LEN: usize = 1 + 32 + 32 + 32;

    // for deserialization
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, WPokt::LEN];
        let (is_initialized_src, bridge_address_src, owner_src, mint_src) =
            array_refs![src, 1, 32, 32, 32];
        let is_initialized = match is_initialized_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let bridge_address: Pubkey = Pubkey::new_from_array(*bridge_address_src);
        let owner: Pubkey = Pubkey::new_from_array(*owner_src);
        let mint: Pubkey = Pubkey::new_from_array(*mint_src);
        Ok(Self {
            is_initialized,
            bridge_address,
            owner,
            mint,
        })
    }

    // for serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, WPokt::LEN];
        let (_is_initialized, _bridge_address, _owner, _mint) = mut_array_refs![dst, 1, 32, 32, 32];
        let WPokt {
            is_initialized,
            bridge_address,
            owner,
            mint,
        } = self;
        _is_initialized[0] = *is_initialized as u8;
        _bridge_address.copy_from_slice(bridge_address.as_ref());
        _owner.copy_from_slice(owner.as_ref());
        _mint.copy_from_slice(mint.as_ref());
    }
}

impl IsInitialized for WPokt {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
