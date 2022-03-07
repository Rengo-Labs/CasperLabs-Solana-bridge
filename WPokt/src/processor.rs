use crate::error::WPoktError;
use crate::instruction::WPoktInstruction;
use crate::state::WPokt;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent, Sysvar},
};
use spl_token;

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
            WPoktInstruction::Burn { amount } => Ok(()),
        }
    }
}

fn constructor(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();

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

    // create init mint instruction
    let mint_account = next_account_info(account_info_iter)?;
    let _mint_account: &Pubkey = mint_account.key;
    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        _mint_account,
        &owner.key,
        Some(&owner.key),
        9,
    )?;

    // init mint CPI
    let rent_sysvar_account = next_account_info(account_info_iter)?;
    program::invoke(
        &init_mint_ix,
        &[mint_account.clone(), rent_sysvar_account.clone()],
    )?;
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

    if wpokt_data.bridge_address == Pubkey::new(&[0_u8; 32]) {
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
        return Err(ProgramError::Custom(
            WPoktError::AccountNotOwnedByWPokt as u32,
        ));
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
    let token_program_account = next_account_info(account_info_iter)?;
    let receiver_account = next_account_info(account_info_iter)?;

    // mint instruction
    let mint_ix = spl_token::instruction::mint_to(
        token_program_account.key,
        mint_account.key,
        receiver_account.key,
        _program_id,
        &[owner_account.key],
        _amount,
    )?;

    // invoke instruction
    program::invoke(
        &mint_ix,
        &[
            mint_account.clone(),
            receiver_account.clone(),
            owner_account.clone(),
        ],
    )?;
    Ok(())
}