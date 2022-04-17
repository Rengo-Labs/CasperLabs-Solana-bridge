use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum WPOKTInstruction {
    /// Accounts Expected
    /// 0. `[writable]` The account to initialize as mint account
    /// 1. `[writable]` The WPOKT global state account
    Construct { initial_minter: Pubkey },
    /// Accounts Expected
    /// 0. `[]` The WPOKT global state account
    /// 1. `[writeable]` The Mint account
    /// 2. `[signer]` The mint authority
    /// 3. `[writeable]` the token account to mint to
    MintOnlyMinter { to: Pubkey, value: u64 },
    /// Accounts Expected
    /// 0. `[writeable]` The WPOKT global state account
    /// 1. `[writeable]` The Mint account
    /// 2. `[signer]` The mint authority
    /// 3. `[writeable]` The new mint authority
    ChangeMinterOnlyMinter { new_minter: Pubkey },
    ///   0. `[writable]` The NoncesDictionary account
    ///   1. `[writable]` The source account.
    ///   2. `[]` The delegate.
    ///   3. `[signer]` The source account owner.
    Permit {
        owner: Pubkey,
        spender: Pubkey,
        value: u64,
        deadline: u64,
    },
    ///   0. `[writable]` The AuthorizationState account
    ///   1. `[writable]` The mint account.
    ///   2. `[writable]` The source token account.
    ///   3. `[signer]` The source token account owner.
    ///   3. `[writable]` The destination token account.
    TransferWithAuthorization {
        from: Pubkey,
        to: Pubkey,
        value: u64,
        valid_after: u64,
        valid_before: u64,
        nonce: String,
    },
}
