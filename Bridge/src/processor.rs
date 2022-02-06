use crate::error::BridgeError;
use crate::instruction::BridgeInstruction;
use crate::state::{Bridge, ClaimedDictionary};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::collections::BTreeMap;
pub struct Processor {}
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Deserializing instruction.");
        let instruction = BridgeInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            BridgeInstruction::Construct {
                w_pokt_address,
                verify_address,
                chain_id,
                stable_fee,
            } => {
                msg!("Construct Instruction.");
                construct(
                    program_id,
                    accounts,
                    &w_pokt_address,
                    &verify_address,
                    &chain_id,
                    &stable_fee,
                )
            }
            BridgeInstruction::TransferRequest => Ok(()),
            BridgeInstruction::TransferReceipt => Ok(()),
            BridgeInstruction::UpdateVerifyAddressOnlyOwner => Ok(()),
            BridgeInstruction::UpdateTokenLimitOnlyOwner => Ok(()),
            BridgeInstruction::SetTokenLimitTimeOnlyOwner => Ok(()),
            BridgeInstruction::UpdateStableFeeOnlyOwner => Ok(()),
            BridgeInstruction::UpdateTokenFeeOnlyOwner => Ok(()),
            BridgeInstruction::UpdateFees => Ok(()),
            BridgeInstruction::WithdrawFeesOnlyOwner => Ok(()),
            BridgeInstruction::AddTokenOnlyOwner => Ok(()),
            BridgeInstruction::PauseTokenOnlyOwner => Ok(()),
            BridgeInstruction::UnpauseTokenOnlyOwner => Ok(()),
            BridgeInstruction::CalculateFee => Ok(()),
            BridgeInstruction::Verify => Ok(()),
        }
    }
}

pub fn assert_with_msg(statement: bool, err: ProgramError, msg: &str) -> ProgramResult {
    if !statement {
        msg!(msg);
        Err(err)
    } else {
        Ok(())
    }
}

fn construct(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _w_pokt_address: &Pubkey,
    _verify_address: &Pubkey,
    _chain_id: &u64,
    _stable_fee: &u64,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();

    msg!("Validating Owner AccountInfo.");
    let owner_account = next_account_info(account_info_iter)?;
    assert_with_msg(
        owner_account.is_signer,
        ProgramError::MissingRequiredSignature,
        "Owner didn't sign.",
    )?;

    msg!("Validating Bridge AccountInfo.");
    let bridge_account = next_account_info(account_info_iter)?;
    assert_with_msg(
        bridge_account.owner == _program_id,
        ProgramError::InvalidAccountData,
        "Bridge account not owned by program.",
    )?;
    assert_with_msg(
        bridge_account.is_writable,
        BridgeError::NotWritable.into(),
        "Bridge account is not writeable.",
    )?;
    let mut bridge_data = Bridge::try_from_slice(&bridge_account.data.borrow())?;
    assert_with_msg(
        !bridge_data.is_initialized,
        ProgramError::AccountAlreadyInitialized,
        "Bridge account already initialized.",
    )?;

    msg!("Initializing Bridge Account data.");
    bridge_data.owner = *owner_account.key;
    bridge_data.fee_update_duration = 1;
    bridge_data.stable_fee = *_stable_fee;
    bridge_data.chain_id = *_chain_id;
    bridge_data.verify_address = *_verify_address;
    bridge_data.is_initialized = true;
    bridge_data.serialize(&mut &mut bridge_account.data.borrow_mut()[..])?;

    msg!("Validating Claimed Account");
    let mut claimed_account = next_account_info(account_info_iter)?;
    let mut claimed_data = ClaimedDictionary::try_from_slice(&claimed_account.data.borrow())?;
    msg!("Init Claimed Data dictionary.");
    claimed_data.claimed_dictionary = BTreeMap::new();
    claimed_data.is_initialized = true;

    // claimed_data.claimed_dictionary.insert(1, true);
    claimed_data
        .claimed_dictionary
        .insert(1, BTreeMap::new()).unwrap();
    let _ = claimed_data.claimed_dictionary.get_mut(&1).unwrap().insert(1, true).unwrap();
    // inner_dict.or_insert(1, true);
    msg!("Serializing Claimed Data.");
    claimed_data.serialize(&mut &mut claimed_account.data.borrow_mut()[..])?;

    Ok(())
}

// fn authorized_call(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
//     let account_info_iter = &mut accounts.iter();

//     let owner_account = next_account_info(account_info_iter)?;
//     if !owner_account.is_signer {
//         return Err(ProgramError::MissingRequiredSignature);
//     }
//     assert_with_msg(
//         owner_account.is_signer,
//         ProgramError::MissingRequiredSignature,
//         "Owner didn't sign.",
//     )?;

//     let bridge_account = next_account_info(account_info_iter)?;

//     let mut bridge_data = Bridge::try_from_slice(&bridge_account.data.borrow())?;
//     assert_with_msg(
//         bridge_data.is_initialized,
//         ProgramError::AccountAlreadyInitialized,
//         "Bridge account already initialized.",
//     )?;

//     Ok(())
// }
