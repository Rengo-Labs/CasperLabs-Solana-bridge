use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum WPoktInstruction {
    Construct,
    SetBridgeOnlyOwner,
    MintOnlyBridge,
    Burn,
}
