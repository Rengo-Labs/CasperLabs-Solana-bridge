use crate::instruction::WPOKTInstruction;
use crate::state::WPOKT;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
};
use spl_token_2022;

const EIP712DOMAIN_HASH: [u8; 32] = [0_u8; 32];
const NAME_HASH: [u8; 32] = [0_u8; 32];
const VERSION_HASH: [u8; 32] = [0_u8; 32];
const PERMIT_TYPEHASH: [u8; 32] = [0_u8; 32];
const TRANSFER_WITH_AUTHORIZATION_TYPEHASH: [u8; 32] = [0_u8; 32];

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

            WPOKTInstruction::GetChainId => get_chain_id(program_id, accounts),
            WPOKTInstruction::GetDomainSeparator => Ok(()),
            WPOKTInstruction::MintOnlyMinter { to, value } => mint(program_id, accounts, to, value),
            WPOKTInstruction::ChangeMinterOnlyMinter { new_minter } => {
                change_minter(program_id, accounts, new_minter)
            }
            WPOKTInstruction::Permit => Ok(()),
            WPOKTInstruction::TransferWithAuthorization => Ok(()),
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

    if *wpokt_account.owner != *_program_id {
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
    wpokt_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);

    Ok(())
}

fn get_chain_id(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
fn get_domain_separator(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
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

fn permit(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
fn transfer_with_authorization(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
