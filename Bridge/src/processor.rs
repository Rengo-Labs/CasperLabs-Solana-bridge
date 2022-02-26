// use crate::error::BridgeError;
use crate::error::BridgeError;
use crate::instruction::BridgeInstruction;
use crate::state::{
    Bridge, CalcuateFeeResult, ClaimedDictionary, DailyTokenClaimsDictionary, TokenAddedDictionary,
    TokenData, TokenListDictionary,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use std::collections::BTreeMap;

pub const TEN_POW_18: u64 = 1000000000000000000;

pub struct Processor {}
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = BridgeInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            BridgeInstruction::Construct {
                w_pokt_address,
                verify_address,
                chain_id,
                stable_fee,
            } => construct(
                program_id,
                accounts,
                &w_pokt_address,
                &verify_address,
                &chain_id,
                &stable_fee,
            ),
            BridgeInstruction::TransferRequest {
                token_index,
                to,
                amount,
                chain_id,
            } => Ok(()),
            BridgeInstruction::TransferReceipt {
                token_index,
                from,
                to,
                amount,
                chain_id,
                index,
                signature,
            } => Ok(()),
            BridgeInstruction::UpdateVerifyAddressOnlyOwner { verify_address } => {
                update_verify_address(program_id, accounts, &verify_address)
            }

            BridgeInstruction::UpdateTokenLimitOnlyOwner { token_index, limit } => {
                update_token_limit(program_id, accounts, token_index, limit)
            }

            BridgeInstruction::SetTokenLimitTimeOnlyOwner {
                token_index,
                timestamp,
            } => set_token_limit_time(program_id, accounts, token_index, timestamp),
            BridgeInstruction::UpdateStableFeeOnlyOwner { new_stable_fee } => {
                update_stable_fee(program_id, accounts, new_stable_fee)
            }

            BridgeInstruction::UpdateTokenFeeOnlyOwner {
                index,
                new_token_fee,
            } => update_token_fee(program_id, accounts, index, new_token_fee),
            BridgeInstruction::UpdateFees { token_index } => {
                update_fees(program_id, accounts, token_index)
            }
            BridgeInstruction::WithdrawFeesOnlyOwner { index } => Ok(()),
            BridgeInstruction::AddTokenOnlyOwner {
                index,
                token_address,
                fee,
                limit,
            } => add_token(program_id, accounts, index, token_address, fee, limit),
            BridgeInstruction::PauseTokenOnlyOwner { token_index } => {
                pause_token(program_id, accounts, token_index)
            }
            BridgeInstruction::UnpauseTokenOnlyOwner { token_index } => {
                unpause_token(program_id, accounts, token_index)
            }
            BridgeInstruction::CalculateFee {
                token_index,
                amount,
            } => calculate_fee(program_id, accounts, token_index, amount),
            BridgeInstruction::Verify {
                token_index,
                from,
                to,
                amount,
                chain_id,
                index,
                signature,
            } => Ok(()),
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
    verify_transaction_signatures(_accounts)?;
    verify_program_accounts_ownership(_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let claimed_account = next_account_info(account_info_iter)?;
    let dtc_account = next_account_info(account_info_iter)?;
    let token_added_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut claimed_data = ClaimedDictionary::unpack_from_slice(&claimed_account.data.borrow())?;
    let mut dtc_data = DailyTokenClaimsDictionary::unpack_from_slice(&dtc_account.data.borrow())?;
    let mut token_added_data =
        TokenAddedDictionary::unpack_from_slice(&token_added_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    verify_program_accounts_initialization(
        Some(&bridge_data),
        Some(&claimed_data),
        Some(&dtc_data),
        Some(&token_list_data),
        Some(&token_added_data),
        false,
    )?;

    bridge_data.owner = *owner_account.key;
    bridge_data.fee_update_duration = 1;
    bridge_data.stable_fee = *_stable_fee;
    bridge_data.chain_id = *_chain_id;
    bridge_data.verify_address = *_verify_address;
    bridge_data.is_initialized = true;

    claimed_data.claimed_dictionary = BTreeMap::new();
    claimed_data.is_initialized = true;

    dtc_data.daily_token_claims_dictionary = BTreeMap::new();
    dtc_data.is_initialized = true;

    token_added_data.token_added_dictionary = BTreeMap::new();
    token_added_data
        .token_added_dictionary
        .insert(*_w_pokt_address, true);
    token_added_data.is_initialized = true;

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

    token_list_data.token_list_dictionary = BTreeMap::new();
    let token_data_bytes = token_data.try_to_vec().unwrap();
    token_list_data
        .token_list_dictionary
        .insert(1, token_data_bytes);
    token_list_data.is_initialized = true;

    bridge_data.claimed_dictionary = *claimed_account.key;
    bridge_data.daily_token_claims_dictionary = *dtc_account.key;
    bridge_data.token_added_dictionary = *token_added_account.key;
    bridge_data.token_list_dictionary = *token_list_account.key;

    claimed_data.pack_into_slice(&mut &mut claimed_account.data.borrow_mut()[..]);
    dtc_data.pack_into_slice(&mut &mut claimed_account.data.borrow_mut()[..]);
    token_added_data.pack_into_slice(&mut &mut token_added_account.data.borrow_mut()[..]);
    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    bridge_data.pack_into_slice(&mut &mut bridge_account.data.borrow_mut()[..]);

    Ok(())
}

fn update_verify_address(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _verify_address: &Pubkey,
) -> ProgramResult {
    verify_transaction_signatures(_accounts)?;
    verify_program_accounts_ownership(_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;

    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;

    verify_program_accounts_initialization(Some(&bridge_data), None, None, None, None, true)?;
    only_owner(&owner_account, &bridge_data)?;

    bridge_data.verify_address = *_verify_address;
    bridge_data.pack_into_slice(&mut &mut bridge_account.data.borrow_mut()[..]);
    Ok(())
}

fn update_token_limit(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
    _limit: u64,
) -> ProgramResult {
    verify_transaction_signatures(_accounts)?;
    verify_program_accounts_ownership(_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    verify_program_accounts_initialization(
        Some(&bridge_data),
        None,
        None,
        Some(&token_list_data),
        None,
        true,
    )?;
    only_owner(&owner_account, &bridge_data)?;

    if !token_list_data
        .token_list_dictionary
        .contains_key(&_token_index)
    {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }
    let ret: &Vec<u8> = token_list_data
        .token_list_dictionary
        .get(&_token_index)
        .unwrap();
    let mut ret: TokenData = TokenData::try_from_slice(ret)?;
    ret.limit = _limit;
    let ret: Vec<u8> = ret.try_to_vec()?;
    let _ = token_list_data
        .token_list_dictionary
        .insert(_token_index, ret);
    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    Ok(())
}

fn set_token_limit_time(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
    _timestamp: u64,
) -> ProgramResult {
    verify_transaction_signatures(_accounts)?;
    verify_program_accounts_ownership(_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    verify_program_accounts_initialization(
        Some(&bridge_data),
        None,
        None,
        Some(&token_list_data),
        None,
        true,
    )?;
    only_owner(&owner_account, &bridge_data)?;

    if !token_list_data
        .token_list_dictionary
        .contains_key(&_token_index)
    {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }
    let ret: &Vec<u8> = token_list_data
        .token_list_dictionary
        .get(&_token_index)
        .unwrap();
    let mut ret: TokenData = TokenData::try_from_slice(ret)?;
    ret.limit_timestamp = _timestamp;
    let ret: Vec<u8> = ret.try_to_vec()?;
    let _ = token_list_data
        .token_list_dictionary
        .insert(_token_index, ret);
    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    Ok(())
}

fn update_stable_fee(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _new_stable_fee: u64,
) -> ProgramResult {
    verify_transaction_signatures(_accounts)?;
    verify_program_accounts_ownership(_program_id, _accounts[1..].as_ref())?;
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;

    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    verify_program_accounts_initialization(Some(&bridge_data), None, None, None, None, true)?;

    only_owner(&owner_account, &bridge_data)?;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    // bridge_data.stable_fee_update_time
    let sum_result = bridge_data
        .fee_update_duration
        .checked_add(current_timestamp as u64);

    match sum_result {
        None => return Err(ProgramError::Custom(BridgeError::Overflow as u32)),
        Some(new_stable_fee_update_time) => {
            bridge_data.new_stable_fee = _new_stable_fee;
            bridge_data.stable_fee_update_time = new_stable_fee_update_time;
        }
    }

    bridge_data.pack_into_slice(&mut &mut bridge_account.data.borrow_mut()[..]);
    Ok(())
}

fn update_token_fee(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _index: u64,
    _new_token_fee: u64,
) -> ProgramResult {
    verify_transaction_signatures(_accounts)?;
    verify_program_accounts_ownership(_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    verify_program_accounts_initialization(
        Some(&bridge_data),
        None,
        None,
        Some(&token_list_data),
        None,
        true,
    )?;
    only_owner(&owner_account, &bridge_data)?;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    // bridge_data.stable_fee_update_time
    let sum_result = bridge_data
        .fee_update_duration
        .checked_add(current_timestamp as u64);

    match sum_result {
        None => return Err(ProgramError::Custom(BridgeError::Overflow as u32)),
        Some(new_fee_update_time) => {
            if !token_list_data.token_list_dictionary.contains_key(&_index) {
                return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
            }
            let ret: &Vec<u8> = token_list_data.token_list_dictionary.get(&_index).unwrap();
            let mut ret: TokenData = TokenData::try_from_slice(ret)?;

            ret.fee_update_time = new_fee_update_time;
            ret.new_fee = _new_token_fee;

            let ret: Vec<u8> = ret.try_to_vec()?;
            let _ = token_list_data.token_list_dictionary.insert(_index, ret);
            token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
        }
    }

    Ok(())
}

fn update_fees(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    verify_program_accounts_ownership(_program_id, _accounts)?;
    _update_stable_fee(_program_id, _accounts)?;
    _update_token_fee(_program_id, _accounts, _token_index)?;
    Ok(())
}

fn add_token(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _index: u64,
    _token_address: Pubkey,
    _fee: u64,
    _limit: u64,
) -> ProgramResult {
    verify_transaction_signatures(&_accounts[0..2])?;
    verify_program_accounts_ownership(&_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;
    let token_added_account = next_account_info(account_info_iter)?;

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;
    let mut token_added_data =
        TokenAddedDictionary::unpack_from_slice(&token_added_account.data.borrow())?;

    verify_program_accounts_initialization(
        Some(&bridge_data),
        None,
        None,
        Some(&token_list_data),
        Some(&token_added_data),
        true,
    )?;
    only_owner(&owner_account, &bridge_data)?;

    if token_list_data.token_list_dictionary.contains_key(&_index) == true {
        return Err(ProgramError::Custom(BridgeError::TokenAlreadyAdded as u32));
    }

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    // bridge_data.stable_fee_update_time
    let sum_result = SECONDS_PER_DAY.checked_add(current_timestamp as u64);

    match sum_result {
        None => return Err(ProgramError::Custom(BridgeError::Overflow as u32)),
        Some(limit_timestamp) => {
            let token_data: Vec<u8> = TokenData {
                token_address: _token_address,
                exists: true,
                paused: false,
                total_fees_collected: 0,
                fee: _fee,
                fee_update_time: 0,
                new_fee: 0,
                limit: _limit,
                limit_timestamp,
            }
            .try_to_vec()
            .unwrap();

            let _ = token_list_data
                .token_list_dictionary
                .insert(_index, token_data);
            let _ = token_added_data
                .token_added_dictionary
                .insert(_token_address, true);

            token_added_data.pack_into_slice(&mut &mut token_added_account.data.borrow_mut()[..]);
            token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
        }
    }
    Ok(())
}

fn pause_token(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    verify_transaction_signatures(&_accounts[..2])?;
    verify_program_accounts_ownership(&_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    verify_program_accounts_initialization(
        Some(&bridge_data),
        None,
        None,
        Some(&token_list_data),
        None,
        true,
    )?;
    only_owner(&owner_account, &bridge_data)?;

    if token_list_data
        .token_list_dictionary
        .contains_key(&_token_index)
        == false
    {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }

    let mut token_data: TokenData = TokenData::try_from_slice(
        &token_list_data
            .token_list_dictionary
            .get(&_token_index)
            .unwrap(),
    )
    .unwrap();

    if token_data.paused == true {
        return Err(ProgramError::Custom(BridgeError::TokenAlreadyPaused as u32));
    }

    token_data.paused = true;

    let _ = token_list_data
        .token_list_dictionary
        .insert(_token_index, token_data.try_to_vec().unwrap());

    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);

    Ok(())
}

fn unpause_token(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    verify_transaction_signatures(&_accounts[..2])?;
    verify_program_accounts_ownership(&_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    verify_program_accounts_initialization(
        Some(&bridge_data),
        None,
        None,
        Some(&token_list_data),
        None,
        true,
    )?;
    only_owner(&owner_account, &bridge_data)?;

    if token_list_data
        .token_list_dictionary
        .contains_key(&_token_index)
        == false
    {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }

    let mut token_data: TokenData = TokenData::try_from_slice(
        &token_list_data
            .token_list_dictionary
            .get(&_token_index)
            .unwrap(),
    )
    .unwrap();

    if token_data.paused == false {
        return Err(ProgramError::Custom(
            BridgeError::TokenAlreadyUnaused as u32,
        ));
    }

    token_data.paused = false;

    let _ = token_list_data
        .token_list_dictionary
        .insert(_token_index, token_data.try_to_vec().unwrap());

    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);

    Ok(())
}

fn _update_daily_limit(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    verify_transaction_signatures(&_accounts[..2])?;
    verify_program_accounts_ownership(&_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;
    let daily_token_claims_account = next_account_info(account_info_iter)?;

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;
    let mut daily_token_claims_data =
        DailyTokenClaimsDictionary::unpack_from_slice(&daily_token_claims_account.data.borrow())?;

    verify_program_accounts_initialization(
        Some(&bridge_data),
        None,
        None,
        Some(&token_list_data),
        None,
        true,
    )?;
    only_owner(&owner_account, &bridge_data)?;

    if token_list_data
        .token_list_dictionary
        .contains_key(&_token_index)
        == false
    {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }

    let mut token_data: TokenData = TokenData::try_from_slice(
        &token_list_data
            .token_list_dictionary
            .get(&_token_index)
            .unwrap(),
    )
    .unwrap();

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if current_timestamp as u64 <= token_data.limit_timestamp {
        return Ok(());
    }
    // bridge_data.stable_fee_update_time
    let sum_result = SECONDS_PER_DAY.checked_add(current_timestamp as u64);

    match sum_result {
        None => return Err(ProgramError::Custom(BridgeError::Overflow as u32)),
        Some(limit_timestamp) => {
            token_data.limit_timestamp = limit_timestamp;
            let _ = token_list_data
                .token_list_dictionary
                .insert(_token_index, token_data.try_to_vec().unwrap());

            let _ = daily_token_claims_data
                .daily_token_claims_dictionary
                .insert(_token_index, 0);

            token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
            daily_token_claims_data
                .pack_into_slice(&mut &mut daily_token_claims_account.data.borrow_mut()[..]);
        }
    }
    Ok(())
}
fn _update_token_fee(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    let token_list_account = &_accounts[1];

    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;
    verify_program_accounts_initialization(None, None, None, Some(&token_list_data), None, true)?;

    if !token_list_data
        .token_list_dictionary
        .contains_key(&_token_index)
    {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }
    let token_data: &Vec<u8> = token_list_data
        .token_list_dictionary
        .get(&_token_index)
        .unwrap();
    let mut token_data: TokenData = TokenData::try_from_slice(token_data)?;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if token_data.fee_update_time == 0 {
        return Ok(());
    }

    if current_timestamp as u64 > token_data.fee_update_time {
        token_data.fee = token_data.new_fee;
        token_data.fee_update_time = 0;

        let token_data: Vec<u8> = token_data.try_to_vec()?;
        let _ = token_list_data
            .token_list_dictionary
            .insert(_token_index, token_data);
        token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    }

    Ok(())
}

fn _update_stable_fee(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    let bridge_account: &AccountInfo = &_accounts[0];
    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;

    verify_program_accounts_initialization(Some(&bridge_data), None, None, None, None, true)?;

    if bridge_data.stable_fee_update_time == 0 {
        return Ok(());
    }

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if current_timestamp as u64 > bridge_data.stable_fee_update_time {
        bridge_data.stable_fee = bridge_data.new_stable_fee;
        bridge_data.stable_fee_update_time = 0;
        bridge_data.pack_into_slice(&mut &mut bridge_account.data.borrow_mut()[..]);
    }

    Ok(())
}

fn calculate_fee(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
    _amount: u64,
) -> ProgramResult {
    verify_program_accounts_ownership(&_program_id, _accounts)?;

    let account_info_iter = &mut _accounts.iter();
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;
    let calculate_fee_result_account = next_account_info(account_info_iter)?;

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;
    let mut calculate_fee_result_data =
        CalcuateFeeResult::unpack_from_slice(&calculate_fee_result_account.data.borrow())?;

    verify_program_accounts_initialization(
        Some(&bridge_data),
        None,
        None,
        Some(&token_list_data),
        None,
        true,
    )?;

    if token_list_data
        .token_list_dictionary
        .contains_key(&_token_index)
        == false
    {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }

    let token_data: TokenData = TokenData::try_from_slice(
        &token_list_data
            .token_list_dictionary
            .get(&_token_index)
            .unwrap(),
    )
    .unwrap();

    if token_data.fee != 0 {
        if token_data.fee >= TEN_POW_18 {
            calculate_fee_result_data.fee = 0;
            calculate_fee_result_data
                .pack_into_slice(&mut &mut calculate_fee_result_account.data.borrow_mut()[..]);

            return Ok(());
        }
        return match _amount.checked_mul(token_data.fee) {
            None => return Err(ProgramError::Custom(BridgeError::Overflow as u32)),
            Some(product) => {
                let quotient: u64 = product / TEN_POW_18;
                calculate_fee_result_data.fee = quotient;
                calculate_fee_result_data
                    .pack_into_slice(&mut &mut calculate_fee_result_account.data.borrow_mut()[..]);

                Ok(())
            }
        };
    }

    if bridge_data.stable_fee >= TEN_POW_18 {
        calculate_fee_result_data.fee = 0;
        calculate_fee_result_data
            .pack_into_slice(&mut &mut calculate_fee_result_account.data.borrow_mut()[..]);

        return Ok(());
    }

    let result: u64 = match _amount.checked_mul(bridge_data.stable_fee) {
        None => return Err(ProgramError::Custom(BridgeError::Overflow as u32)),
        Some(product) => product / TEN_POW_18,
    };

    calculate_fee_result_data.fee = result;
    calculate_fee_result_data
        .pack_into_slice(&mut &mut calculate_fee_result_account.data.borrow_mut()[..]);

    Ok(())
}
// ========================== Helper Functions ==================== //

// verifies that all given accounts signed the transaciton
fn verify_transaction_signatures(_accounts: &[AccountInfo]) -> ProgramResult {
    for account in _accounts.into_iter() {
        if !account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
    }
    Ok(())
}

// Checks that all provided accounts are owned by the provided program_id
fn verify_program_accounts_ownership(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    // verify all accounts are owned by current program
    for account in _accounts.into_iter() {
        if account.owner != _program_id {
            return Err(ProgramError::Custom(
                BridgeError::AccountNotOwnedByBridge as u32,
            ));
        }
    }
    Ok(())
}

// Verifies that all provided accounts's data has been initialized
fn verify_program_accounts_initialization(
    _bridge_data: Option<&Bridge>,
    _claimed_data: Option<&ClaimedDictionary>,
    _daily_token_claimes_data: Option<&DailyTokenClaimsDictionary>,
    _token_list: Option<&TokenListDictionary>,
    _token_added: Option<&TokenAddedDictionary>,
    _status: bool,
) -> ProgramResult {
    let err = match _status {
        true => ProgramError::UninitializedAccount,
        false => ProgramError::AccountAlreadyInitialized,
    };

    if _bridge_data.is_some() {
        if _bridge_data.unwrap().is_initialized != _status {
            return Err(err);
        }
    }

    if _claimed_data.is_some() {
        if _claimed_data.unwrap().is_initialized != _status {
            return Err(err);
        }
    }

    if _daily_token_claimes_data.is_some() {
        if _daily_token_claimes_data.unwrap().is_initialized != _status {
            return Err(err);
        }
    }

    if _token_list.is_some() {
        if _token_list.unwrap().is_initialized != _status {
            return Err(err);
        }
    }

    if _token_added.is_some() {
        if _bridge_data.unwrap().is_initialized != _status {
            return Err(err);
        }
    }
    Ok(())
}

// Verifies that Bridge account owner initiated the transaction
fn only_owner(_owner_account: &AccountInfo, bridge_data: &Bridge) -> ProgramResult {
    if !_owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if bridge_data.owner != *_owner_account.key {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}
