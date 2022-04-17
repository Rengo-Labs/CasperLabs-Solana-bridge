use crate::error::WPOKTError;
use crate::instruction::WPOKTInstruction;
use crate::state::{AuthorizationStateDictionary, NoncesDictionary, WPOKT};

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token_2022;
use std::collections::BTreeMap;

pub struct Processor {}
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = WPOKTInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        match instruction {
            WPOKTInstruction::Construct { initial_minter } => {
                construct(program_id, accounts, &initial_minter)
            }
            WPOKTInstruction::MintOnlyMinter { to, value } => mint(program_id, accounts, to, value),
            WPOKTInstruction::ChangeMinterOnlyMinter { new_minter } => {
                change_minter(program_id, accounts, new_minter)
            }
            WPOKTInstruction::Permit {
                owner,
                spender,
                value,
                deadline,
            } => permit(program_id, accounts, owner, spender, value, deadline),
            WPOKTInstruction::TransferWithAuthorization {
                from,
                to,
                value,
                valid_after,
                valid_before,
                nonce,
            } => transfer_with_authorization(
                program_id,
                accounts,
                from,
                to,
                value,
                valid_after,
                valid_before,
                nonce,
            ),
        }
    }
}

fn construct(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _initial_minter: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let mint_account = next_account_info(account_info_iter)?;
    let wpokt_account = next_account_info(account_info_iter)?;
    let nonces_account = next_account_info(account_info_iter)?;
    let authorization_state_account = next_account_info(account_info_iter)?;

    if *wpokt_account.owner != *_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    if *nonces_account.owner != *_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    if *authorization_state_account.owner != *_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mint_ix = spl_token_2022::instruction::initialize_mint2(
        &spl_token_2022::id(),
        mint_account.key,
        _initial_minter,
        None,
        6,
    )?;
    program::invoke(&mint_ix, &[mint_account.clone()])?;

    let mut wpokt_data = WPOKT::unpack_from_slice(&wpokt_account.data.borrow())?;
    wpokt_data.minter = *_initial_minter;
    wpokt_data.mint = *mint_account.key;
    wpokt_data.is_initialized = true;

    let mut nonces_data = NoncesDictionary::unpack_from_slice(&nonces_account.data.borrow())?;
    nonces_data.nonces_dictionary = BTreeMap::new();
    nonces_data.is_initialized = true;

    let mut authorization_state_data = AuthorizationStateDictionary::unpack_from_slice(
        &authorization_state_account.data.borrow(),
    )?;
    authorization_state_data.authorization_state_dictionary = BTreeMap::new();
    authorization_state_data.is_initialized = true;

    wpokt_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);
    nonces_data.pack_into_slice(&mut &mut nonces_account.data.borrow_mut()[..]);
    authorization_state_data
        .pack_into_slice(&mut &mut authorization_state_account.data.borrow_mut()[..]);

    Ok(())
}

fn mint(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _to: Pubkey,
    _value: u64,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let wpokt_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let mint_authority = next_account_info(account_info_iter)?;
    let receiver_account = next_account_info(account_info_iter)?;

    if *wpokt_account.owner != *_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !mint_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let wpokt_data = WPOKT::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !wpokt_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    if *mint_authority.key != wpokt_data.minter || *receiver_account.key != _to {
        return Err(ProgramError::InvalidAccountData);
    }

    // mint instruction
    let mint_ix = spl_token_2022::instruction::mint_to(
        &spl_token_2022::id(),
        mint_account.key,
        receiver_account.key,
        mint_authority.key,
        &[mint_authority.key],
        _value,
    )?;

    // invoke instruction
    program::invoke(
        &mint_ix,
        &[
            mint_account.clone(),
            receiver_account.clone(),
            mint_authority.clone(),
        ],
    )?;
    Ok(())
}

fn change_minter(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _new_minter: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let wpokt_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let mint_authority = next_account_info(account_info_iter)?;
    let new_mint_authority_account = next_account_info(account_info_iter)?;

    // onlyOwner
    if !mint_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut wpokt_data = WPOKT::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !wpokt_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    if *mint_authority.key != wpokt_data.minter || *new_mint_authority_account.key != _new_minter {
        return Err(ProgramError::InvalidAccountData);
    }

    let change_mint_auth_ix = spl_token_2022::instruction::set_authority(
        &spl_token_2022::id(),
        mint_account.key,
        Some(new_mint_authority_account.key),
        spl_token_2022::instruction::AuthorityType::MintTokens,
        mint_authority.key,
        &[mint_authority.key],
    )?;

    program::invoke(
        &change_mint_auth_ix,
        &[mint_account.clone(), new_mint_authority_account.clone()],
    )?;

    wpokt_data.minter = *new_mint_authority_account.key;
    wpokt_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);
    Ok(())
}

fn permit(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    owner: Pubkey,
    spender: Pubkey,
    value: u64,
    deadline: u64,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let nonces_account = next_account_info(account_info_iter)?;
    let src_token_account_owner = next_account_info(account_info_iter)?;
    let src_token_account = next_account_info(account_info_iter)?;
    let delegate_account = next_account_info(account_info_iter)?;

    if *delegate_account.key != spender {
        return Err(ProgramError::InvalidInstructionData);
    }
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp as u64;

    if deadline >= current_timestamp {
        return Err(ProgramError::Custom(WPOKTError::AuthExpired as u32));
    }

    let mut nonces_data = NoncesDictionary::unpack_from_slice(&nonces_account.data.borrow())?;
    let current_nonce = nonces_data.nonces_dictionary.get_mut(&owner).unwrap();
    *current_nonce += 1;

    nonces_data.pack_into_slice(&mut &mut nonces_account.data.borrow_mut()[..]);

    let approve_ix = spl_token_2022::instruction::approve(
        &spl_token_2022::id(),
        src_token_account.key,
        delegate_account.key,
        src_token_account_owner.key,
        &[
            &src_token_account.key,
            &delegate_account.key,
            &src_token_account_owner.key,
        ],
        value,
    )?;
    program::invoke(
        &approve_ix,
        &[
            src_token_account.clone(),
            delegate_account.clone(),
            src_token_account_owner.clone(),
        ],
    )?;
    Ok(())
}

fn transfer_with_authorization(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _from: Pubkey,
    _to: Pubkey,
    _value: u64,
    _valid_after: u64,
    _valid_before: u64,
    _nonce: String,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let authorization_state_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let src_token_account = next_account_info(account_info_iter)?;
    let src_token_account_owner_account = next_account_info(account_info_iter)?;
    let destination_account = next_account_info(account_info_iter)?;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp as u64;

    if current_timestamp >= _valid_before {
        return Err(ProgramError::Custom(WPOKTError::AuthExpired as u32));
    }
    if current_timestamp <= _valid_after {
        return Err(ProgramError::Custom(WPOKTError::AuthNotYetValid as u32));
    }

    let mut authorization_state_data = AuthorizationStateDictionary::unpack_from_slice(
        &authorization_state_account.data.borrow(),
    )?;

    let auth_state = authorization_state_data
        .authorization_state_dictionary
        .get_mut(&AuthorizationStateDictionary::generate_key(_from, _nonce))
        .unwrap();

    if !*auth_state {
        return Err(ProgramError::Custom(WPOKTError::AuthAlreadyUsed as u32));
    }

    *auth_state = true;
    authorization_state_data
        .pack_into_slice(&mut &mut authorization_state_account.data.borrow_mut()[..]);

    let mint_data = spl_token_2022::state::Mint::unpack_from_slice(&mint_account.data.borrow())?;
    let transfer_ix = spl_token_2022::instruction::transfer_checked(
        &spl_token_2022::id(),
        src_token_account.key,
        mint_account.key,
        destination_account.key,
        src_token_account_owner_account.key,
        &[src_token_account_owner_account.key],
        _value,
        mint_data.decimals,
    )?;

    program::invoke(
        &transfer_ix,
        &[
            src_token_account.clone(),
            mint_account.clone(),
            destination_account.clone(),
            src_token_account_owner_account.clone(),
        ],
    )?;

    Ok(())
}
