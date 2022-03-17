use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum WPOKTInstruction {
    /// Accounts Expected
    /// 0. `[writable]` The account to initialize as mint account
    /// 1. `[writable]` The WPOKT global state account
    Construct {
        initial_minter: Pubkey,
    },
    GetChainId,
    GetDomainSeparator,
    /// Accounts Expected
    /// 0. `[]` The WPOKT global state account
    /// 1. `[writeable]` The Mint account
    /// 2. `[signer]` The mint authority
    /// 3. `[writeable]` the token account to mint to
    MintOnlyMinter {
        to: Pubkey,
        value: u64,
    },
    /// Accounts Expected
    /// 0. `[writeable]` The WPOKT global state account
    /// 1. `[writeable]` The Mint account
    /// 2. `[signer]` The mint authority
    /// 3. `[writeable]` The new mint authority
    ChangeMinterOnlyMinter {
        new_minter: Pubkey,
    },
    Permit,
    TransferWithAuthorization,
}
