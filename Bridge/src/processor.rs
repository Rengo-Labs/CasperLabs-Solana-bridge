// use crate::error::BridgeError;
use crate::instruction::BridgeInstruction;
use crate::state::{
    Bridge, ClaimedDictionary, DailyTokenClaimsDictionary, TokenAddedDictionary, TokenData,
    TokenListDictionary,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
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

fn construct(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _w_pokt_address: &Pubkey,
    _verify_address: &Pubkey,
    _chain_id: &u64,
    _stable_fee: &u64,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();

    let owner_account = next_account_info(account_info_iter)?;
    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let bridge_account = next_account_info(account_info_iter)?;
    if bridge_account.owner != _program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    if bridge_data.is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    bridge_data.owner = *owner_account.key;
    bridge_data.fee_update_duration = 1;
    bridge_data.stable_fee = *_stable_fee;
    bridge_data.chain_id = *_chain_id;
    bridge_data.verify_address = *_verify_address;
    bridge_data.is_initialized = true;

    let claimed_account = next_account_info(account_info_iter)?;
    if claimed_account.owner != _program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let mut claimed_data = ClaimedDictionary::unpack_from_slice(&claimed_account.data.borrow())?;
    claimed_data.claimed_dictionary = BTreeMap::new();
    claimed_data.is_initialized = true;
    bridge_data.claimed_dictionary = *claimed_account.key;

    let dtc_account = next_account_info(account_info_iter)?;
    if dtc_account.owner != _program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let mut dtc_data = DailyTokenClaimsDictionary::unpack_from_slice(&dtc_account.data.borrow())?;
    dtc_data.daily_token_claims_dictionary = BTreeMap::new();
    dtc_data.is_initialized = true;
    bridge_data.daily_token_claims_dictionary = *dtc_account.key;

    let token_added_account = next_account_info(account_info_iter)?;
    if token_added_account.owner != _program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let mut token_added_data =
        TokenAddedDictionary::unpack_from_slice(&token_added_account.data.borrow())?;
    token_added_data.token_added_dictionary = BTreeMap::new();
    token_added_data
        .token_added_dictionary
        .insert(*_w_pokt_address, true);
    token_added_data.is_initialized = true;
    bridge_data.token_added_dictionary = *token_added_account.key;

    let token_list_account = next_account_info(account_info_iter)?;
    if token_list_account.owner != _program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;
    token_list_data.token_list_dictionary = BTreeMap::new();
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;
    let token_data = TokenData {
        token_address: *_w_pokt_address,
        exists: true,
        paused: false,
        // total fees collected
        total_fees_collected: 0,
        // current fee
        fee: 0,
        // fee update time
        fee_update_time: 0,
        // new fee
        new_fee: 0,
        // daily limit
        limit: 0,
        // daily limit time
        limit_timestamp: current_timestamp as u64 + SECONDS_PER_DAY,
    };

    let token_data_bytes = token_data.try_to_vec().unwrap();
    token_list_data
        .token_list_dictionary
        .insert(1, token_data_bytes);
    bridge_data.token_list_dictionary = *token_list_account.key;

    claimed_data.pack_into_slice(&mut &mut claimed_account.data.borrow_mut()[..]);
    dtc_data.pack_into_slice(&mut &mut claimed_account.data.borrow_mut()[..]);
    token_added_data.pack_into_slice(&mut &mut token_added_account.data.borrow_mut()[..]);
    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    bridge_data.pack_into_slice(&mut &mut bridge_account.data.borrow_mut()[..]);

    Ok(())
}
