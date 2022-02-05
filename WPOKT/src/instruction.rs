use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum WPOKTInstruction {
    Construct,
    GetChainId,
    GetDomainSeparator,
    MintOnlyMinter,
    ChangeMinterOnlyMinter,
    Burn,
    Approve,
    Transfer,
    TransferFrom,
    Permit,
    TransferWithAuthorization,
}
