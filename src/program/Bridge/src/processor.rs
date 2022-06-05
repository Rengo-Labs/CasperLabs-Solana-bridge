// use crate::error::BridgeError;
use crate::error::BridgeError;
use crate::instruction::BridgeInstruction;
use crate::state::CalcuateFeeResult;
use crate::state::{
    Bridge, ClaimedDictionary, DailyTokenClaimsDictionary, TokenAddedDictionary,
    TokenListDictionary, COMMON_BASE_SEED,
};
use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    msg, program,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use spl_token;

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
            } => {
                msg!("BridgeInstruction::Construct");
                construct(
                    program_id,
                    accounts,
                    &w_pokt_address,
                    &verify_address,
                    &chain_id,
                    &stable_fee,
                )
            }
            BridgeInstruction::TransferRequest {
                token_index,
                to,
                amount,
                chain_id,
            } => {
                msg!("BridgeInstruction::TransferRequest");
                transfer_request(program_id, accounts, token_index, to, amount, chain_id)
            }
            BridgeInstruction::TransferReceipt {
                token_index,
                from,
                to,
                amount,
                chain_id,
                index,
                signature_account,
            } => transfer_receipt(
                program_id,
                accounts,
                token_index,
                &from,
                &to,
                amount,
                chain_id,
                index,
                &signature_account,
            ),
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
            BridgeInstruction::WithdrawFeesOnlyOwner { index } => {
                withdraw_fees(program_id, accounts, index)
            }
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
            BridgeInstruction::RenounceOwnership => renounce_ownership(program_id, accounts),
            BridgeInstruction::TransferOwnership { new_owner } => {
                transfer_ownership(program_id, accounts, new_owner)
            }
            BridgeInstruction::CreateClaimedDictionaryPdaAccount { index, chain_id } => {
                msg!("BridgeInstruction::CreateClaimedDictionaryPdaAccount");
                create_claimed_dictionary_pda_account(program_id, accounts, index, chain_id)
            }
            BridgeInstruction::CreateDailyTokenClaimsDictionaryPdaAccount { token_index } => {
                msg!("BridgeInstruction::CreateDailyTokenClaimsDictionaryPdaAccount");
                create_daily_token_claims_dictionary_pda_account(program_id, accounts, token_index)
            }
        }
    }
}

// /// Accounts Expected:
// ///
// /// 0. [signer] payer_account
// /// 1. [] system_account
// /// 2. [] rent_sysvar_account
// /// 3. [] pda_account_info
// fn create_pda_account<T: Pack + Sealed + GeneratePdaKey>(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     seeds: Vec<&[u8]>,
// ) -> ProgramResult {
//     let account_info_iter = &mut accounts.iter();
//     let payer_account = next_account_info(account_info_iter)?;
//     let system_account = next_account_info(account_info_iter)?;
//     let rent_sysvar_account = next_account_info(account_info_iter)?;
//     let pda_account_info = next_account_info(account_info_iter)?;

//     let (pda, bump) = T::generate_pda_key(program_id, &seeds);
//     if !pda_account_info.key.eq(&pda) {
//         return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
//     }

//     let mut signature_seeds: Vec<&[u8]> = vec![];
//     let bump_ref = &[bump];
//     for item in seeds {
//         signature_seeds.push(item);
//     }
//     signature_seeds.push(bump_ref);

//     // get string constant seeds
//     let str_const = T::get_constants(); // of lengts 2
//     let ref1: &[u8] = str_const[0].as_ref();
//     signature_seeds.push(ref1);
//     let ref2: &[u8] = str_const[1].as_ref();
//     signature_seeds.push(ref2);

//     let rent_sysvar = Rent::from_account_info(rent_sysvar_account)?;
//     let data_size = std::mem::size_of::<T>();
//     let ix = system_instruction::create_account(
//         payer_account.key,
//         pda_account_info.key,
//         rent_sysvar.minimum_balance(data_size),
//         data_size.try_into().unwrap(),
//         program_id,
//     );

//     program::invoke_signed(
//         &ix,
//         &[
//             pda_account_info.clone(),
//             payer_account.clone(),
//             system_account.clone(),
//         ],
//         &[signature_seeds.as_slice()],
//     )?;
//     Ok(())
// }

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
    let bridge_account = next_account_info(account_info_iter)?; // PDA account
    let token_added_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;
    let system_program_account = next_account_info(account_info_iter)?;
    let rent_system_account = next_account_info(account_info_iter)?;
    let bridge_token_account = next_account_info(account_info_iter)?;
    let w_pokt_mint_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;

    let rent_sysvar = Rent::from_account_info(rent_system_account)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !w_pokt_mint_account.key.eq(_w_pokt_address) {
        return Err(ProgramError::InvalidArgument);
    }

    // create Bridge PDA account
    let (bridge_pda, bridge_bump, bridge_seed1, bridge_seed2) =
        Bridge::generate_pda_key(_program_id);
    if !bridge_account.key.eq(&bridge_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let create_bridge_pda_ix = system_instruction::create_account(
        owner_account.key,
        bridge_account.key,
        rent_sysvar.minimum_balance(Bridge::LEN),
        Bridge::LEN.try_into().unwrap(),
        _program_id,
    );
    program::invoke_signed(
        &create_bridge_pda_ix,
        &[
            owner_account.clone(),
            bridge_account.clone(),
            system_program_account.clone(),
        ],
        &[&[bridge_seed1.as_ref(), bridge_seed2.as_ref(), &[bridge_bump]]],
    )?;

    // initialize bridge pda account
    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    bridge_data.owner = *owner_account.key;
    bridge_data.fee_update_duration = 1;
    bridge_data.stable_fee = *_stable_fee;
    bridge_data.chain_id = *_chain_id;
    bridge_data.verify_address = *_verify_address;
    bridge_data.is_initialized = true;
    bridge_data.current_index += 1;
    bridge_data.pack_into_slice(&mut &mut bridge_account.data.borrow_mut()[..]);

    // create and initialize TokenAdded dictionary item account
    let (token_added_pda, token_added_bump, token_added_seed1, token_added_seed2) =
        TokenAddedDictionary::generate_pda_key(_program_id, _w_pokt_address);
    if !token_added_account.key.eq(&token_added_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let create_token_added_pda_ix = system_instruction::create_account(
        owner_account.key,
        token_added_account.key,
        rent_sysvar.minimum_balance(TokenAddedDictionary::LEN),
        TokenAddedDictionary::LEN.try_into().unwrap(),
        _program_id,
    );
    program::invoke_signed(
        &create_token_added_pda_ix,
        &[
            owner_account.clone(),
            token_added_account.clone(),
            system_program_account.clone(),
        ],
        &[&[
            _w_pokt_address.as_ref(),
            token_added_seed1.as_ref(),
            token_added_seed2.as_ref(),
            &[token_added_bump],
        ]],
    )?;

    let mut token_added_data =
        TokenAddedDictionary::unpack_from_slice(&token_added_account.data.borrow())?;
    token_added_data.token_added = true;
    token_added_data.pack_into_slice(&mut &mut token_added_account.data.borrow_mut()[..]);

    // // create and initialize TokenList dictionary item account
    let index: u64 = 1;
    let (token_list_pda, token_list_bump, token_list_seed1, token_list_seed2) =
        TokenListDictionary::generate_pda_key(_program_id, index);

    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let create_token_list_pda_ix = system_instruction::create_account(
        owner_account.key,
        token_list_account.key,
        rent_sysvar.minimum_balance(TokenListDictionary::LEN),
        TokenListDictionary::LEN.try_into().unwrap(),
        _program_id,
    );
    program::invoke_signed(
        &create_token_list_pda_ix,
        &[
            owner_account.clone(),
            token_list_account.clone(),
            system_program_account.clone(),
        ],
        &[&[
            index.to_le_bytes().as_ref(),
            token_list_seed1.as_ref(),
            token_list_seed2.as_ref(),
            &[token_list_bump],
        ]],
    )?;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;
    let token_data_list = TokenListDictionary {
        is_initialized: true,
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
    token_data_list.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);

    // create Bridge Token account associated with WPokt Mint
    let (bridge_token_pda, bridge_token_bump, bridge_token_seed1, bridge_token_seed2) =
        generate_bridge_token_pda(_program_id, _w_pokt_address);
    if !bridge_token_account.key.eq(&bridge_token_pda) {
        return Err(ProgramError::Custom(
            BridgeError::TokenAccountKeyMismatch as u32,
        ));
    }

    let create_token_account_is = system_instruction::create_account(
        owner_account.key,
        bridge_token_account.key,
        rent_sysvar.minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN.try_into().unwrap(),
        &spl_token::id(),
    );
    program::invoke_signed(
        &create_token_account_is,
        &[
            bridge_token_account.clone(),
            owner_account.clone(),
            system_program_account.clone(),
        ],
        &[&[
            _w_pokt_address.as_ref(),
            bridge_token_seed1.as_ref(),
            bridge_token_seed2.as_ref(),
            &[bridge_token_bump],
        ]],
    )?;

    // initialize Bridge Token Account
    let initialize_token_account_ix = spl_token::instruction::initialize_account2(
        &spl_token::id(),
        bridge_token_account.key,
        w_pokt_mint_account.key,
        bridge_account.key,
    )?;

    program::invoke_signed(
        &initialize_token_account_ix,
        &[
            bridge_token_account.clone(),
            token_program_account.clone(),
            w_pokt_mint_account.clone(),
            rent_system_account.clone(),
        ],
        &[&[
            _w_pokt_address.as_ref(),
            bridge_token_seed1.as_ref(),
            bridge_token_seed2.as_ref(),
            &[bridge_token_bump],
        ]],
    )?;

    Ok(())
}

fn transfer_request(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
    _to: Pubkey,
    _amount: u64,
    _chain_id: u64,
) -> ProgramResult {
    verify_program_accounts_ownership(_program_id, _accounts[0..2].as_ref())?;
    let account_info_iter = &mut _accounts.iter();
    let bridge_account = next_account_info(account_info_iter)?; // PDA acount
    let token_list_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let source_account = next_account_info(account_info_iter)?;
    let calculate_fee_result_account = next_account_info(account_info_iter)?;
    let bridge_token_account = next_account_info(account_info_iter)?;
    let source_auth_account = next_account_info(account_info_iter)?;

    if !source_auth_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    if token_list_data.token_address != *mint_account.key {
        return Err(ProgramError::InvalidInstructionData);
    }

    if token_list_data.paused {
        return Err(ProgramError::Custom(BridgeError::TokenAlreadyPaused as u32));
    }
    if _chain_id == bridge_data.chain_id {
        return Err(ProgramError::Custom(BridgeError::RequestToSameChain as u32));
    }

    update_fees(
        _program_id,
        &[bridge_account.clone(), token_list_account.clone()],
        _token_index,
    )?;
    // get the fee calculated
    calculate_fee(
        _program_id,
        &[
            bridge_account.clone(),
            token_list_account.clone(),
            calculate_fee_result_account.clone(),
        ],
        _token_index,
        _amount,
    )?;
    let calculate_fee_result_data =
        CalcuateFeeResult::unpack_from_slice(&calculate_fee_result_account.data.borrow())?;
    let fee = calculate_fee_result_data.fee;

    // tokenData.totalFeesCollected = tokenData.totalFeesCollected.add(_fee);
    token_list_data.total_fees_collected = token_list_data
        .total_fees_collected
        .checked_add(fee)
        .ok_or(ProgramError::Custom(BridgeError::Overflow as u32))
        .unwrap();
    let transfer_from_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        source_account.key,
        bridge_token_account.key,
        source_auth_account.key,
        &[&source_auth_account.key],
        _amount,
    )?;

    // source auth account can also be the delegate
    program::invoke(
        &transfer_from_ix,
        &[
            source_account.clone(),
            mint_account.clone(),
            bridge_token_account.clone(),
            source_auth_account.clone(),
        ],
    )?;
    bridge_data.current_index += 1;

    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    bridge_data.pack_into_slice(&mut &mut bridge_account.data.borrow_mut()[..]);
    Ok(())
}

fn transfer_receipt(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
    _from: &Pubkey,
    _to: &Pubkey,
    _amount: u64,
    _chain_id: u64,
    _index: u64,
    _signature_account: &Pubkey,
) -> ProgramResult {
    verify_program_accounts_ownership(_program_id, _accounts[1..5].as_ref())?;
    let account_info_iter = &mut _accounts.iter();
    let destination_auth = next_account_info(account_info_iter)?; // The account the submitting and paying for the transaction
    let bridge_account = next_account_info(account_info_iter)?; // PDA Account
    let claimed_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;
    let daily_token_claims_account = next_account_info(account_info_iter)?;
    let source_auth = next_account_info(account_info_iter)?; // The account the transaction creator signed this transaction with - the Aithority of source token
    let source_token_account = next_account_info(account_info_iter)?;
    let destination_token_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;

    if !destination_auth.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !destination_auth.key.eq(_to) {
        return Err(ProgramError::InvalidArgument);
    }

    let (bridge_pda, _, _, _) = Bridge::generate_pda_key(_program_id);
    if !bridge_account.key.eq(&bridge_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    if !source_auth.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if *source_auth.key != *_signature_account {
        return Err(ProgramError::InvalidArgument);
    }

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let token_data = TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;
    let mut claimed_data = ClaimedDictionary::unpack_from_slice(*claimed_account.data.borrow())?;
    let mut daily_token_claims_data =
        DailyTokenClaimsDictionary::unpack_from_slice(&daily_token_claims_account.data.borrow())?;

    if !token_data.exists {
        return Err(ProgramError::Custom(BridgeError::NonExistantToken as u32));
    }
    if token_data.paused {
        return Err(ProgramError::Custom(BridgeError::TokenAlreadyPaused as u32));
    }
    if bridge_data.chain_id == _chain_id {
        return Err(ProgramError::Custom(BridgeError::RequestToSameChain as u32));
    }

    if claimed_data.claimed {
        return Err(ProgramError::Custom(BridgeError::AlreadyClaimed as u32));
    }
    if token_data.limit > 0 {
        _update_daily_limit(
            _program_id,
            &[
                token_list_account.clone(),
                daily_token_claims_account.clone(),
            ],
            _index,
        )?;
        if daily_token_claims_data.daily_token_claims + _amount <= token_data.limit {
            return Err(ProgramError::Custom(
                BridgeError::ClaimAboveDailyLimit as u32,
            ));
        }
    }

    if token_data.token_address != *mint_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        source_token_account.key,
        destination_token_account.key,
        source_auth.key,
        &[&source_auth.key],
        _amount,
    )?;

    program::invoke(
        &transfer_ix,
        &[
            source_token_account.clone(),
            destination_token_account.clone(),
            source_auth.clone(),
            token_program_account.clone(),
        ],
    )?;

    claimed_data.claimed = true;
    daily_token_claims_data.daily_token_claims += _amount;

    claimed_data.pack_into_slice(&mut &mut claimed_account.data.borrow_mut()[..]);
    daily_token_claims_data
        .pack_into_slice(&mut &mut daily_token_claims_account.data.borrow_mut()[..]);

    Ok(())
}

fn update_verify_address(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _verify_address: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !bridge_account.owner.eq(_program_id) {
        return Err(ProgramError::IllegalOwner);
    }

    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;

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
    verify_program_accounts_ownership(_program_id, _accounts[1..3].as_ref())?;
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    only_owner(&owner_account, &bridge_data)?;

    let (token_list_pda, _, _, _) =
        TokenListDictionary::generate_pda_key(_program_id, _token_index);
    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }

    token_list_data.limit = _limit;
    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    Ok(())
}

fn set_token_limit_time(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
    _timestamp: u64,
) -> ProgramResult {
    verify_program_accounts_ownership(_program_id, _accounts[1..3].as_ref())?;
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    only_owner(&owner_account, &bridge_data)?;

    let (token_list_pda, _, _, _) =
        TokenListDictionary::generate_pda_key(_program_id, _token_index);
    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }

    token_list_data.limit_timestamp = _timestamp;
    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    Ok(())
}

fn update_stable_fee(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _new_stable_fee: u64,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !bridge_account.owner.eq(_program_id) {
        return Err(ProgramError::IllegalOwner);
    }

    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    only_owner(&owner_account, &bridge_data)?;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

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
    verify_program_accounts_ownership(_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    only_owner(&owner_account, &bridge_data)?;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    // bridge_data.stable_fee_update_time
    let sum_result = bridge_data
        .fee_update_duration
        .checked_add(current_timestamp as u64);

    let (token_list_pda, _, _, _) = TokenListDictionary::generate_pda_key(_program_id, _index);
    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }

    match sum_result {
        None => return Err(ProgramError::Custom(BridgeError::Overflow as u32)),
        Some(new_fee_update_time) => {
            token_list_data.fee_update_time = new_fee_update_time;
            token_list_data.new_fee = _new_token_fee;
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

fn withdraw_fees(_program_id: &Pubkey, _accounts: &[AccountInfo], _index: u64) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let owner_token_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?; // the PDA account
    let bridge_token_account = next_account_info(account_info_iter)?; // PDA token account
    let mint_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    let (pda, bump, seed1, seed2) = Bridge::generate_pda_key(_program_id);

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;

    only_owner(owner_account, &bridge_data)?;

    let (token_list_pda, _, _, _) = TokenListDictionary::generate_pda_key(_program_id, _index);
    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::Custom(BridgeError::MapKeyNotFound as u32));
    }

    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    if !(token_list_data.total_fees_collected > 0) {
        return Err(ProgramError::Custom(BridgeError::NothingToWithdraw as u32));
    }

    let to_transfer = token_list_data.total_fees_collected;
    token_list_data.total_fees_collected = 0;

    if token_list_data.token_address != *mint_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let pda_seeds: &[&[u8]] = &[seed1.as_bytes(), seed2.as_bytes(), &[bump]];
    let mint_data = spl_token::state::Mint::unpack_from_slice(&mint_account.data.borrow())?;
    let transfer_ix = spl_token::instruction::transfer_checked(
        &spl_token::id(),
        bridge_token_account.key,
        mint_account.key,
        owner_token_account.key,
        &pda,
        &[&pda],
        to_transfer,
        mint_data.decimals,
    )?;

    program::invoke_signed(
        &transfer_ix,
        &[
            bridge_token_account.clone(),
            mint_account.clone(),
            owner_token_account.clone(),
            bridge_account.clone(),
        ],
        &[pda_seeds],
    )?;

    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
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
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?; // PDA Account
    let token_list_account = next_account_info(account_info_iter)?; // PDA Account
    let token_added_account = next_account_info(account_info_iter)?; // PDA Account
    let mint = next_account_info(account_info_iter)?;
    let rent_sysvar_account = next_account_info(account_info_iter)?;
    let system_program_account = next_account_info(account_info_iter)?;

    let rent_sysvar = Rent::from_account_info(rent_sysvar_account)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    only_owner(&owner_account, &bridge_data)?;

    // create and initialize TokenAdded dictionary item account
    let (token_added_pda, token_added_bump, token_added_seed1, token_added_seed2) =
        TokenAddedDictionary::generate_pda_key(_program_id, mint.key);
    if !token_added_account.key.eq(&token_added_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let create_token_added_pda_ix = system_instruction::create_account(
        owner_account.key,
        token_added_account.key,
        rent_sysvar.minimum_balance(TokenAddedDictionary::LEN),
        TokenAddedDictionary::LEN.try_into().unwrap(),
        _program_id,
    );
    program::invoke_signed(
        &create_token_added_pda_ix,
        &[
            owner_account.clone(),
            token_added_account.clone(),
            system_program_account.clone(),
        ],
        &[&[
            _token_address.as_ref(),
            token_added_seed1.as_ref(),
            token_added_seed2.as_ref(),
            &[token_added_bump],
        ]],
    )?;

    let mut token_added_data =
        TokenAddedDictionary::unpack_from_slice(&token_added_account.data.borrow())?;
    token_added_data.token_added = true;
    token_added_data.pack_into_slice(&mut &mut token_added_account.data.borrow_mut()[..]);

    // // create and initialize TokenList dictionary item account
    let (token_list_pda, token_list_bump, token_list_seed1, token_list_seed2) =
        TokenListDictionary::generate_pda_key(_program_id, _index);

    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let create_token_list_pda_ix = system_instruction::create_account(
        owner_account.key,
        token_list_account.key,
        rent_sysvar.minimum_balance(TokenListDictionary::LEN),
        TokenListDictionary::LEN.try_into().unwrap(),
        _program_id,
    );
    program::invoke_signed(
        &create_token_list_pda_ix,
        &[
            owner_account.clone(),
            token_list_account.clone(),
            system_program_account.clone(),
        ],
        &[&[
            _index.to_le_bytes().as_ref(),
            token_list_seed1.as_ref(),
            token_list_seed2.as_ref(),
            &[token_list_bump],
        ]],
    )?;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;
    let limit_timestamp = SECONDS_PER_DAY
        .checked_add(current_timestamp as u64)
        .unwrap();

    let token_data_list = TokenListDictionary {
        is_initialized: true,
        token_address: _token_address,
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
        limit_timestamp: limit_timestamp,
    };
    token_data_list.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    Ok(())
}

fn pause_token(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    verify_program_accounts_ownership(&_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    only_owner(&owner_account, &bridge_data)?;

    let (token_list_pda, _, _, _) =
        TokenListDictionary::generate_pda_key(_program_id, _token_index);

    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    if token_list_data.paused == true {
        return Err(ProgramError::Custom(BridgeError::TokenAlreadyPaused as u32));
    }

    token_list_data.paused = true;
    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    Ok(())
}

fn unpause_token(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    verify_program_accounts_ownership(&_program_id, _accounts[1..].as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    only_owner(&owner_account, &bridge_data)?;

    let (token_list_pda, _, _, _) =
        TokenListDictionary::generate_pda_key(_program_id, _token_index);

    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    if token_list_data.paused == false {
        return Err(ProgramError::Custom(
            BridgeError::TokenAlreadyUnaused as u32,
        ));
    }

    token_list_data.paused = false;
    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    Ok(())
}

fn _update_daily_limit(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    verify_program_accounts_ownership(&_program_id, _accounts.as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let token_list_account = next_account_info(account_info_iter)?;
    let daily_token_claims_account = next_account_info(account_info_iter)?;

    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;
    let mut daily_token_claims_data =
        DailyTokenClaimsDictionary::unpack_from_slice(&daily_token_claims_account.data.borrow())?;

    let (token_list_pda, _, _, _) =
        TokenListDictionary::generate_pda_key(_program_id, _token_index);
    let (dtc_pda, _) = DailyTokenClaimsDictionary::generate_pda_key(_program_id, _token_index);
    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::InvalidSeeds);
    }
    if !daily_token_claims_account.key.eq(&dtc_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if current_timestamp as u64 <= token_list_data.limit_timestamp {
        return Ok(());
    }
    let limit_timestamp = SECONDS_PER_DAY
        .checked_add(current_timestamp as u64)
        .unwrap();

    token_list_data.limit_timestamp = limit_timestamp;
    daily_token_claims_data.daily_token_claims = 0;

    token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    daily_token_claims_data
        .pack_into_slice(&mut &mut daily_token_claims_account.data.borrow_mut()[..]);
    Ok(())
}

/// Accounts expected
/// 1. `[writable]` the TokenList account
fn _update_token_fee(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    let token_list_account = &_accounts[1];

    let (token_list_pda, _, _, _) =
        TokenListDictionary::generate_pda_key(_program_id, _token_index);
    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let mut token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if token_list_data.fee_update_time == 0 {
        return Ok(());
    }

    if current_timestamp as u64 > token_list_data.fee_update_time {
        token_list_data.fee = token_list_data.new_fee;
        token_list_data.fee_update_time = 0;
        token_list_data.pack_into_slice(&mut &mut token_list_account.data.borrow_mut()[..]);
    }

    Ok(())
}

/// Accounts Expected
/// 0. `[writable]` the Bridge account
fn _update_stable_fee(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    let bridge_account: &AccountInfo = &_accounts[0];
    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;

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
    verify_program_accounts_ownership(&_program_id, _accounts.as_ref())?;

    let account_info_iter = &mut _accounts.iter();
    let bridge_account = next_account_info(account_info_iter)?;
    let token_list_account = next_account_info(account_info_iter)?;
    let calculate_fee_result_account = next_account_info(account_info_iter)?;

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data.borrow())?;
    let mut calculate_fee_result_data =
        CalcuateFeeResult::unpack_from_slice(&calculate_fee_result_account.data.borrow())?;

    let (token_list_pda, _, _, _) =
        TokenListDictionary::generate_pda_key(_program_id, _token_index);
    if !token_list_account.key.eq(&token_list_pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data.borrow())?;

    if token_list_data.fee != 0 {
        if token_list_data.fee >= TEN_POW_18 {
            calculate_fee_result_data.fee = 0;
            calculate_fee_result_data
                .pack_into_slice(&mut &mut calculate_fee_result_account.data.borrow_mut()[..]);

            return Ok(());
        }
        return match _amount.checked_mul(token_list_data.fee) {
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

fn renounce_ownership(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let wpokt_account = next_account_info(account_info_iter)?;

    if wpokt_account.owner != _program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut bridge_data = Bridge::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !bridge_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    only_owner(owner_account, &bridge_data)?;

    bridge_data.owner = Pubkey::new_from_array([0_u8; 32]);
    bridge_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);
    Ok(())
}

fn transfer_ownership(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _new_owner: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let wpokt_account = next_account_info(account_info_iter)?;

    if wpokt_account.owner != _program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut bridge_data = Bridge::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !bridge_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    only_owner(owner_account, &bridge_data)?;

    if _new_owner == Pubkey::new_from_array([0_u8; 32]) {
        return Err(ProgramError::InvalidArgument);
    }
    bridge_data.owner = _new_owner;
    bridge_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);
    Ok(())
}

fn create_claimed_dictionary_pda_account(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _index: u64,
    _chain_id: u64,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let payer = next_account_info(account_info_iter)?;
    let claimed_pda_account = next_account_info(account_info_iter)?;
    let rent_sysvar_account = next_account_info(account_info_iter)?;
    let system_account = next_account_info(account_info_iter)?;

    let (pda, bump) = ClaimedDictionary::generate_pda_key(_program_id, _chain_id, _index);

    if !claimed_pda_account.key.eq(&pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent_sysvar = Rent::from_account_info(rent_sysvar_account)?;
    let ix = system_instruction::create_account(
        payer.key,
        claimed_pda_account.key,
        rent_sysvar.minimum_balance(ClaimedDictionary::LEN),
        ClaimedDictionary::LEN.try_into().unwrap(),
        _program_id,
    );

    let index_bytes = _index.to_le_bytes();
    let chain_id_bytes = _chain_id.to_le_bytes();
    let signature_seeds = &[
        index_bytes.as_ref(),
        chain_id_bytes.as_ref(),
        COMMON_BASE_SEED.as_bytes(),
        ClaimedDictionary::BASE_SEED.as_bytes(),
        &[bump],
    ];

    program::invoke_signed(
        &ix,
        &[
            payer.clone(),
            claimed_pda_account.clone(),
            system_account.clone(),
        ],
        &[signature_seeds],
    )?;
    Ok(())
}

fn create_daily_token_claims_dictionary_pda_account(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _token_index: u64,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let payer = next_account_info(account_info_iter)?;
    let dtc_pda_account = next_account_info(account_info_iter)?;
    let rent_sysvar_account = next_account_info(account_info_iter)?;
    let system_account = next_account_info(account_info_iter)?;

    let (pda, bump) = DailyTokenClaimsDictionary::generate_pda_key(_program_id, _token_index);

    if !dtc_pda_account.key.eq(&pda) {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent_sysvar = Rent::from_account_info(rent_sysvar_account)?;
    let ix = system_instruction::create_account(
        payer.key,
        dtc_pda_account.key,
        rent_sysvar.minimum_balance(DailyTokenClaimsDictionary::LEN),
        DailyTokenClaimsDictionary::LEN.try_into().unwrap(),
        _program_id,
    );

    let index_bytes = _token_index.to_le_bytes();
    let signature_seeds = &[
        index_bytes.as_ref(),
        COMMON_BASE_SEED.as_bytes(),
        DailyTokenClaimsDictionary::BASE_SEED.as_bytes(),
        &[bump],
    ];

    program::invoke_signed(
        &ix,
        &[
            payer.clone(),
            dtc_pda_account.clone(),
            system_account.clone(),
        ],
        &[signature_seeds],
    )?;
    Ok(())
}
// ========================== Helper Functions ==================== //

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

fn generate_bridge_token_pda(_program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8, String, String) {
    let seed1 = "bridge";
    let seed2 = "bridge_token_account";
    let seeds = &[
        mint.as_ref(),
        seed1.as_bytes().as_ref(),
        seed2.as_bytes().as_ref(),
    ];
    let (pda, bump) = Pubkey::find_program_address(seeds, _program_id);
    return (pda, bump, seed1.to_string(), seed2.to_string());
}
