use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum WPoktInstruction {
    /// Accounts expected:
    /// 0. `[signer]` The account of person deploying WPokt - the 'owner'.
    /// 1. `[writable]` The account used as WPokt's global state
    /// 2. `[]` the Mint account created by 'owner'.
    /// 3. `[]` the Rent sysvar
    Construct,
    /// 0. `[signer]` The account of person deploying WPokt - the 'owner'.
    SetBridgeOnlyOwner {
        bridge_address: Pubkey,
    },
    /// Accounts expected:
    /// 0. `[]` The program owner's account.
    /// 1. `[]` The account used as WPokt's global state
    /// 2. `[signer]` the account used by Bridge as global state.
    /// 3. `[writable]` the Mint account created by 'owner'.
    /// 4. `[]` the Token program.
    /// 5. `[writeable]` the token account to mint to.
    MintOnlyBridge {
        amount: u64,
    },
    Burn {
        amount: u64,
    },
}
