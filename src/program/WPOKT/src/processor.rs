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
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use spl_token;
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
    let payer = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let wpokt_account = next_account_info(account_info_iter)?;
    let nonces_account = next_account_info(account_info_iter)?;
    let authorization_state_account = next_account_info(account_info_iter)?;
    let rent_sysvar_account = next_account_info(account_info_iter)?;
    let system_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;
    let initial_minter_account = next_account_info(account_info_iter)?;

    if *wpokt_account.owner != *_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    if *nonces_account.owner != *_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    if *authorization_state_account.owner != *_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !initial_minter_account.key.eq(_initial_minter) {
        return Err(ProgramError::InvalidInstructionData);
    }

    let (pda, bump_seed) = generate_wpokt_pda(_program_id, mint_account.key);

    if *wpokt_account.key != pda {
        return Err(ProgramError::InvalidInstructionData);
    }

    let rent_sysvar = Rent::from_account_info(rent_sysvar_account)?;

    // create PDA account
    let create_pda_acc_ix = system_instruction::create_account(
        payer.key,
        wpokt_account.key,
        rent_sysvar.minimum_balance(WPOKT::LEN),
        WPOKT::LEN.try_into().unwrap(),
        _program_id,
    );

    let bump = &[bump_seed];
    let pda_seeds = &[
        mint_account.key.as_ref(),
        b"WPOKT",
        b"global_state_account",
        bump,
    ][..];

    program::invoke_signed(
        &create_pda_acc_ix,
        &[wpokt_account.clone(), payer.clone(), system_account.clone()],
        &[pda_seeds],
    )?;

    // let mut rent_sysvar_account = AccountInfo::
    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        mint_account.key,
        wpokt_account.key,
        None,
        0,
    )?;

    program::invoke(
        &init_mint_ix,
        &[
            mint_account.clone(),
            rent_sysvar_account.clone(),
            token_program_account.clone(),
        ],
    )?;

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
    let minter_account = next_account_info(account_info_iter)?; // signer and payer
    let wpokt_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let receiver_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;
    let (pda, bump_seed) = generate_wpokt_pda(_program_id, mint_account.key);

    if !minter_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if *wpokt_account.key != pda {
        return Err(ProgramError::InvalidInstructionData);
    }

    if *wpokt_account.owner != *_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !receiver_account.key.eq(&_to) {
        return Err(ProgramError::InvalidInstructionData);
    }

    let wpokt_data = WPOKT::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !wpokt_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    if !wpokt_data.minter.eq(minter_account.key) {
        return Err(ProgramError::Custom(WPOKTError::InvalidMinter as u32));
    }

    // mint instruction
    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint_account.key,
        receiver_account.key,
        &pda,
        &[&pda],
        _value,
    )?;

    let bump_ref = &[bump_seed];
    let pda_seeds = &[
        mint_account.key.as_ref(),
        b"WPOKT",
        b"global_state_account",
        bump_ref,
    ][..];

    program::invoke_signed(
        &mint_ix,
        &[
            mint_account.clone(),
            receiver_account.clone(),
            wpokt_account.clone(),
            token_program_account.clone(),
        ],
        &[pda_seeds],
    )?;
    Ok(())
}

fn change_minter(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _new_minter: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let minter = next_account_info(account_info_iter)?; // signer and payer
    let wpokt_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;
    let new_mint_authority_account = next_account_info(account_info_iter)?;

    let (pda, bump_seed) = generate_wpokt_pda(_program_id, mint_account.key);

    // onlyOwner
    if !minter.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !wpokt_account.key.eq(&pda) {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut wpokt_data = WPOKT::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !wpokt_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    if *minter.key != wpokt_data.minter || *new_mint_authority_account.key != _new_minter {
        return Err(ProgramError::InvalidInstructionData);
    }

    let change_mint_auth_ix = spl_token::instruction::set_authority(
        &spl_token::id(),
        mint_account.key,
        Some(new_mint_authority_account.key),
        spl_token::instruction::AuthorityType::MintTokens,
        &pda,
        &[&pda],
    )?;

    let bump_ref = &[bump_seed];
    let pda_seeds = &[
        mint_account.key.as_ref(),
        b"WPOKT",
        b"global_state_account",
        bump_ref,
    ][..];

    program::invoke_signed(
        &change_mint_auth_ix,
        &[
            mint_account.clone(),
            new_mint_authority_account.clone(),
            token_program_account.clone(),
        ],
        &[pda_seeds],
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
    let src_token_account_owner = next_account_info(account_info_iter)?; //signer signed this offline
    let nonces_account = next_account_info(account_info_iter)?;
    let src_token_account = next_account_info(account_info_iter)?;
    let delegate_token_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;

    if *delegate_token_account.key != spender {
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

    let approve_ix = spl_token::instruction::approve(
        &spl_token::id(),
        src_token_account.key,
        delegate_token_account.key,
        src_token_account_owner.key,
        &[
            &src_token_account.key,
            &delegate_token_account.key,
            &src_token_account_owner.key,
        ],
        value,
    )?;
    program::invoke(
        &approve_ix,
        &[
            src_token_account.clone(),
            delegate_token_account.clone(),
            src_token_account_owner.clone(),
            token_program_account.clone(),
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
    let token_program_account = next_account_info(account_info_iter)?;

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

    let mint_data = spl_token::state::Mint::unpack_from_slice(&mint_account.data.borrow())?;
    let transfer_ix = spl_token::instruction::transfer_checked(
        &spl_token::id(),
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
            token_program_account.clone(),
        ],
    )?;

    Ok(())
}

fn generate_wpokt_pda(program_id: &Pubkey, mint_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[mint_account.as_ref(), b"WPOKT", b"global_state_account"],
        program_id,
    )
}
