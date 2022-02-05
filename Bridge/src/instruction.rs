use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum BridgeInstruction {
    Construct,
    TransferRequest,
    TransferReceipt,
    UpdateVerifyAddressOnlyOwner,
    UpdateTokenLimitOnlyOwner,
    SetTokenLimitTimeOnlyOwner,
    UpdateStableFeeOnlyOwner,
    UpdateTokenFeeOnlyOwner,
    UpdateFees,
    WithdrawFeesOnlyOwner,
    AddTokenOnlyOwner,
    PauseTokenOnlyOwner,
    UnpauseTokenOnlyOwner,
    CalculateFee,
    Verify,
}
