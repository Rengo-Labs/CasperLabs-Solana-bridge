use crate::error::WPoktError;
use crate::instruction::WPoktInstruction;
use crate::state::WPokt;
use borsh::{BorshDeserialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token_2022;

pub struct Processor {}
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = WPoktInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        match instruction {
            WPoktInstruction::Construct => constructor(program_id, accounts),
            WPoktInstruction::SetBridgeOnlyOwner { bridge_address } => {
                set_bridge(program_id, accounts, bridge_address)
            }
            WPoktInstruction::MintOnlyBridge { amount } => mint(program_id, accounts, amount),
            WPoktInstruction::Burn { amount } => burn(program_id, accounts, amount),
            WPoktInstruction::RenounceOwnership => renounce_ownership(program_id, accounts),
            WPoktInstruction::TransferOwnership { new_owner } => {
                transfer_ownership(program_id, accounts, new_owner)
            }
        }
    }
}

fn constructor(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let mint_account = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let wpokt_account = next_account_info(account_info_iter)?;
    if wpokt_account.owner != _program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let mut wpokt_data = WPokt::unpack_from_slice(&wpokt_account.data.borrow())?;
    wpokt_data.owner = *owner.key;
    wpokt_data.bridge_address = Pubkey::new(&[0_u8; 32]);
    wpokt_data.is_initialized = true;
    wpokt_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);

    let pda_seed = format!("{}WPokt", *wpokt_account.key);
    let (pda, _nonce) = Pubkey::find_program_address(&[pda_seed.as_bytes()], _program_id);
    // create init mint instruction

    let init_mint_ix = spl_token_2022::instruction::initialize_mint2(
        &spl_token_2022::id(),
        mint_account.key,
        &pda,
        None,
        9,
    )?;
    program::invoke(&init_mint_ix, &[mint_account.clone()])?;
    Ok(())
}

fn set_bridge(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _bridge_address: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();

    let owner = next_account_info(account_info_iter)?;
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let wpokt_account = next_account_info(account_info_iter)?;
    if wpokt_account.owner != _program_id {
        return Err(ProgramError::Custom(
            WPoktError::AccountNotOwnedByWPokt as u32,
        ));
    }
    let mut wpokt_data = WPokt::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !wpokt_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    //onlyOwner
    if wpokt_data.owner != *owner.key {
        return Err(ProgramError::IllegalOwner);
    }

    if wpokt_data.bridge_address != Pubkey::new(&[0_u8; 32]) {
        return Err(ProgramError::Custom(WPoktError::AlreadySet as u32));
    }

    wpokt_data.bridge_address = _bridge_address;
    wpokt_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);

    Ok(())
}

fn mint(_program_id: &Pubkey, _accounts: &[AccountInfo], _amount: u64) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let wpokt_account = next_account_info(account_info_iter)?;
    if wpokt_account.owner != _program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let wpokt_data = WPokt::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !wpokt_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    // onlyBridge
    let bridge_account = next_account_info(account_info_iter)?;
    if !bridge_account.is_signer || wpokt_data.bridge_address != *bridge_account.key {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mint_account = next_account_info(account_info_iter)?;
    let receiver_account = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;

    let seeds = format!("{}WPokt", *wpokt_account.key);
    let (pda, nonce) = Pubkey::find_program_address(&[seeds.as_bytes()], _program_id);
    // // mint instruction
    let mint_ix = spl_token_2022::instruction::mint_to(
        &spl_token_2022::id(),
        mint_account.key,
        receiver_account.key,
        &pda,
        &[&pda],
        _amount,
    )?;

    program::invoke_signed(
        &mint_ix,
        &[
            mint_account.clone(),
            receiver_account.clone(),
            pda_account.clone(),
        ],
        &[&[seeds.as_bytes(), &[nonce]]],
    )?;
    Ok(())
}

fn burn(_program_id: &Pubkey, _accounts: &[AccountInfo], _amount: u64) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let source_account = next_account_info(account_info_iter)?;
    let source_auth_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;

    let burn_ix = spl_token_2022::instruction::burn(
        &spl_token_2022::id(),
        source_account.key,
        mint_account.key,
        source_auth_account.key,
        &[&source_auth_account.key],
        _amount,
    )?;

    program::invoke(
        &burn_ix,
        &[
            source_account.clone(),
            mint_account.clone(),
            source_auth_account.clone(),
        ],
    )?;
    Ok(())
}

fn renounce_ownership(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let wpokt_account = next_account_info(account_info_iter)?;

    if wpokt_account.owner != _program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut wpokt_data = WPokt::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !wpokt_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    // only owner
    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if wpokt_data.owner != *owner_account.key {
        return Err(ProgramError::IllegalOwner);
    }

    wpokt_data.owner = Pubkey::new_from_array([0_u8; 32]);
    wpokt_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);
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
    let mut wpokt_data = WPokt::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !wpokt_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    // only owner
    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if wpokt_data.owner != *owner_account.key {
        return Err(ProgramError::IllegalOwner);
    }

    if _new_owner == Pubkey::new_from_array([0_u8; 32]) {
        return Err(ProgramError::InvalidArgument);
    }
    wpokt_data.owner = _new_owner;
    wpokt_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);
    Ok(())
}