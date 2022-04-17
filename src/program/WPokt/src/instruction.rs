use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum WPoktInstruction {
    /// Accounts expected:
    /// 0. `[signer]` The account of person deploying WPokt - the 'owner'.
    /// 1. `[writable]` The account used as WPokt's global state
    /// 2. `[]` the Mint account created by 'owner'.
    Construct,
    /// 0. `[signer]` The account of person deploying WPokt - the 'owner'.
    SetBridgeOnlyOwner { bridge_address: Pubkey },
    /// Accounts expected:
    /// 0. `[]` The program owner's account.
    /// 1. `[]` The account used as WPokt's global state
    /// 2. `[signer]` the account used by Bridge as global state.
    /// 3. `[writable]` the Mint account created by 'owner'.
    /// 4. `[writeable]` the token account to mint to.
    /// 5. `[]` The PDA account of WPokt to sign for mint
    MintOnlyBridge { amount: u64 },
    /// Accounts expected:
    /// 0. `[writable]` The token account to burn from.
    /// 1. `[signer]` the 0th token account's owner/delegate
    /// 2. `[writable]` the mint account
    Burn { amount: u64 },
    /// Accounts expected:
    /// 0. `[signer]` The program owner's account.
    /// 1. `[writeable]` The account used as WPokt's global state
    RenounceOwnership,
    /// Accounts expected:
    /// 0. `[signer]` The program owner's account.
    /// 1. `[writeable]` The account used as WPokt's global state
    TransferOwnership { new_owner: Pubkey },
}
