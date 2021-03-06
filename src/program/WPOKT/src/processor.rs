use crate::error::WPOKTError;
use crate::instruction::WPOKTInstruction;
use crate::state::{AuthorizationStateDictionary, NoncesDictionary, WPOKT};

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg, program,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use spl_token;

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
                msg!("WPOKTInstruction::Construct");
                construct(program_id, accounts, &initial_minter)
            }
            WPOKTInstruction::MintOnlyMinter { to, value } => {
                msg!("WPOKTInstruction::MintOnlyMinter");
                mint(program_id, accounts, to, value)
            }
            WPOKTInstruction::ChangeMinterOnlyMinter { new_minter } => {
                msg!("WPOKTInstruction::ChangeMinterOnlyMinter");
                change_minter(program_id, accounts, new_minter)
            }
            WPOKTInstruction::Permit {
                owner,
                spender,
                value,
                deadline,
            } => {
                msg!("WPOKTInstruction::Permit");
                permit(program_id, accounts, owner, spender, value, deadline)
            }
            WPOKTInstruction::TransferWithAuthorization {
                from,
                to,
                value,
                valid_after,
                valid_before,
                nonce,
            } => {
                msg!("WPOKTInstruction::TransferWithAuthorization");
                transfer_with_authorization(
                    program_id,
                    accounts,
                    from,
                    to,
                    value,
                    valid_after,
                    valid_before,
                    nonce,
                )
            }
            WPOKTInstruction::InitializeNoncePdaAccount { owner } => {
                msg!("WPOKTInstruction::InitializeNoncePdaAccount");
                initialize_nonce_pda_account(program_id, accounts, &owner)
            }
            WPOKTInstruction::InitializeAuthorizationStatePdaAccount { from, nonce } => {
                msg!("WPOKTInstruction::InitializeAuthorizationStatePdaAccount");
                initialize_authorization_state_pda_account(program_id, accounts, &from, &nonce)
            }
        }
    }
}

fn construct(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _initial_minter: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let payer = next_account_info(account_info_iter)?; // person paying for this and further instructions
    let mint_account = next_account_info(account_info_iter)?;
    let wpokt_account = next_account_info(account_info_iter)?; // PDA account
    let rent_sysvar_account = next_account_info(account_info_iter)?;
    let system_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;
    let initial_minter_account = next_account_info(account_info_iter)?;

    if !initial_minter_account.key.eq(_initial_minter) {
        return Err(ProgramError::Custom(
            WPOKTError::InitialMinterPubkeyMismatch as u32,
        ));
    }

    let (pda, bump_seed) = generate_wpokt_pda(_program_id, mint_account.key);

    if *wpokt_account.key != pda {
        return Err(ProgramError::Custom(WPOKTError::WPOKTPdaMismatch as u32));
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
    wpokt_data.pack_into_slice(&mut &mut wpokt_account.data.borrow_mut()[..]);
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
    let wpokt_account = next_account_info(account_info_iter)?; // PDA account
    let mint_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;
    let new_mint_authority_account = next_account_info(account_info_iter)?;

    let (pda, bump_seed) = generate_wpokt_pda(_program_id, mint_account.key);

    // onlyOwner
    if !minter.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !wpokt_account.key.eq(&pda) {
        return Err(ProgramError::Custom(WPOKTError::WPOKTPdaMismatch as u32));
    }

    let mut wpokt_data = WPOKT::unpack_from_slice(&wpokt_account.data.borrow())?;
    if !wpokt_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    if !minter.key.eq(&wpokt_data.minter) {
        return Err(ProgramError::Custom(WPOKTError::InvalidMinter as u32));
    }

    if *new_mint_authority_account.key != _new_minter {
        return Err(ProgramError::Custom(
            WPOKTError::NewMinterPubkeyMismatch as u32,
        ));
    }

    let change_mint_auth_ix = spl_token::instruction::set_authority(
        &spl_token::id(),
        mint_account.key,
        Some(new_mint_authority_account.key),
        spl_token::instruction::AuthorityType::MintTokens,
        &pda,
        &[&pda],
    )?;

    // let bump_ref = &[bump_seed];
    let pda_seeds = &[
        mint_account.key.as_ref(),
        b"WPOKT",
        b"global_state_account",
        &[bump_seed],
    ];

    program::invoke_signed(
        &change_mint_auth_ix,
        &[
            wpokt_account.clone(),
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
    let payer = next_account_info(account_info_iter)?; // the payer of the transaction, auth of delegate token account
    let src_token_account_owner = next_account_info(account_info_iter)?; //signer signed this offline - the 'owner'
    let nonces_account = next_account_info(account_info_iter)?;
    let src_token_account = next_account_info(account_info_iter)?;
    let delegate_token_account = next_account_info(account_info_iter)?;
    let mint_token_account = next_account_info(account_info_iter)?; // the WPOKT Mint account
    let token_program_account = next_account_info(account_info_iter)?;
    let clock_sysvar_account = next_account_info(account_info_iter)?;

    if !src_token_account_owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !src_token_account_owner.key.eq(&owner) {
        return Err(ProgramError::Custom(
            WPOKTError::TokenAuthorityMismatch as u32,
        ));
    }

    if *delegate_token_account.key != spender {
        return Err(ProgramError::Custom(
            WPOKTError::DelegateSpenderMismatch as u32,
        ));
    }

    let clock = Clock::from_account_info(clock_sysvar_account)?;
    let current_timestamp: u64 = clock.unix_timestamp.try_into().unwrap();
    if current_timestamp >= deadline {
        return Err(ProgramError::Custom(WPOKTError::AuthExpired as u32));
    }

    let (nonce_pda, _) = NoncesDictionary::generate_pda_key(
        _program_id,
        src_token_account_owner.key,
        mint_token_account.key,
    );

    if !nonces_account.owner.eq(_program_id) {
        return Err(ProgramError::IllegalOwner);
    }

    if !nonces_account.key.eq(&nonce_pda) {
        return Err(ProgramError::Custom(
            WPOKTError::NoncesDictionaryItemKeyMismatch as u32,
        ));
    }

    // update nonce
    let mut nonces_data = NoncesDictionary::unpack_from_slice(&nonces_account.data.borrow())?;
    if !nonces_data.owner.eq(&owner) {
        return Err(ProgramError::Custom(
            WPOKTError::NoncesDictionaryItemOwnerMismatch as u32,
        ));
    }
    nonces_data.nonce += 1;
    nonces_data.pack_into_slice(&mut &mut nonces_account.data.borrow_mut()[..]);

    // source token auth will sign this approve
    let approve_ix = spl_token::instruction::approve(
        &spl_token::id(),
        src_token_account.key,
        delegate_token_account.key,
        src_token_account_owner.key,
        &[src_token_account_owner.key],
        value,
    )?;

    program::invoke(
        &approve_ix,
        &[
            src_token_account.clone(),
            delegate_token_account.clone(),
            payer.clone(),
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
    _nonce: [u8; 32],
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let to = next_account_info(account_info_iter)?; // token auth of 'to_token_account' - the delegate
    let authorization_state_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let from_token_account = next_account_info(account_info_iter)?;
    let from = next_account_info(account_info_iter)?;
    let to_token_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;

    //verify correct auth state acc
    let (auth_state_pda, _) = AuthorizationStateDictionary::generate_pda_key(
        _program_id,
        from.key,
        mint_account.key,
        &_nonce,
    );
    if !auth_state_pda.eq(authorization_state_account.key) {
        return Err(ProgramError::Custom(
            WPOKTError::AuthStateDictionaryItemKeyMismatch as u32,
        ));
    }
    // verify payer signature
    if !to.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // validate transaction validity
    let clock = Clock::get()?;
    let current_timestamp: u64 = clock.unix_timestamp.try_into().unwrap();

    if current_timestamp >= _valid_before {
        return Err(ProgramError::Custom(WPOKTError::AuthExpired as u32));
    }
    if current_timestamp <= _valid_after {
        return Err(ProgramError::Custom(WPOKTError::AuthNotYetValid as u32));
    }
    // validate and update authorization state
    let mut authorization_state_data = AuthorizationStateDictionary::unpack_from_slice(
        &authorization_state_account.data.borrow(),
    )?;

    if !authorization_state_data.from.eq(from.key) {
        return Err(ProgramError::Custom(
            WPOKTError::AuthStateDictionaryFromKeyMismatch as u32,
        ));
    }

    if authorization_state_data.nonce != _nonce {
        return Err(ProgramError::Custom(
            WPOKTError::AuthStateDictionaryNonceMismatch as u32,
        ));
    }

    if authorization_state_data.authorization {
        return Err(ProgramError::Custom(WPOKTError::AuthAlreadyUsed as u32));
    }

    authorization_state_data.authorization = true;
    authorization_state_data
        .pack_into_slice(&mut &mut authorization_state_account.data.borrow_mut()[..]);
    // transfer tokens
    let mint_data = spl_token::state::Mint::unpack_from_slice(&mint_account.data.borrow())?;
    let transfer_ix = spl_token::instruction::transfer_checked(
        &spl_token::id(),
        from_token_account.key,
        mint_account.key,
        to_token_account.key,
        from.key,
        &[from.key],
        _value,
        mint_data.decimals,
    )?;

    program::invoke(
        &transfer_ix,
        &[
            from_token_account.clone(),
            mint_account.clone(),
            to_token_account.clone(),
            from.clone(),
            token_program_account.clone(),
        ],
    )?;
    Ok(())
}

fn initialize_nonce_pda_account(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _owner: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let payer = next_account_info(account_info_iter)?; // the payer of transaction
    let owner = next_account_info(account_info_iter)?; // the owner of the nonce
    let nonce_account = next_account_info(account_info_iter)?; // the PDA Nonce account to create
    let mint_account = next_account_info(account_info_iter)?; // the PDA Nonce account to create
    let rent_sysvar_account = next_account_info(account_info_iter)?; // WPOKT Mint account for PDA generation
    let system_account = next_account_info(account_info_iter)?; // WPOKT Mint account for PDA generation

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !owner.key.eq(_owner) {
        return Err(ProgramError::InvalidInstructionData);
    }

    let rent_sysvar = Rent::from_account_info(rent_sysvar_account)?;
    let (nonce_pda, bump) =
        NoncesDictionary::generate_pda_key(_program_id, owner.key, mint_account.key);

    if !(nonce_account.key.eq(&nonce_pda)) {
        return Err(ProgramError::Custom(
            WPOKTError::NoncesDictionaryItemKeyMismatch as u32,
        ));
    }

    let ix = system_instruction::create_account(
        payer.key,
        nonce_account.key,
        rent_sysvar.minimum_balance(NoncesDictionary::LEN),
        NoncesDictionary::LEN.try_into().unwrap(),
        _program_id,
    );

    let seeds = &[
        owner.key.as_ref(),
        mint_account.key.as_ref(),
        b"WPOKT",
        b"nonces_dictionary_key",
        &[bump],
    ];
    program::invoke_signed(
        &ix,
        &[nonce_account.clone(), payer.clone(), system_account.clone()],
        &[seeds],
    )?;

    // deserialize this bih
    let mut account_data =
        NoncesDictionary::unpack_from_slice(&mut &mut nonce_account.data.borrow_mut()[..])?;
    account_data.nonce = 0;
    account_data.owner = *owner.key;
    // serialize this bih
    account_data.pack_into_slice(&mut &mut nonce_account.data.borrow_mut()[..]);

    Ok(())
}

fn initialize_authorization_state_pda_account(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _from: &Pubkey, // the source token account
    _nonce: &[u8; 32],
) -> ProgramResult {
    let account_info_iter = &mut _accounts.iter();
    let payer = next_account_info(account_info_iter)?; // the account paying for the transaction submission
    let from = next_account_info(account_info_iter)?; // the source token account authority
                                                      // let source_token_account = next_account_info(account_info_iter)?; // the source token account
    let authorization_state_account = next_account_info(account_info_iter)?; // the PDA Authorization account to create
    let mint_account = next_account_info(account_info_iter)?; // the WPOKT PDA Mint account
    let rent_sysvar_account = next_account_info(account_info_iter)?;
    let system_account = next_account_info(account_info_iter)?; // WPOKT Mint account for PDA generation

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !from.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !from.key.eq(_from) {
        return Err(ProgramError::InvalidInstructionData);
    }

    let rent_sysvar = Rent::from_account_info(rent_sysvar_account)?;
    let (auth_state_pda, bump) = AuthorizationStateDictionary::generate_pda_key(
        _program_id,
        from.key,
        mint_account.key,
        _nonce,
    );

    let ix = system_instruction::create_account(
        payer.key,
        &auth_state_pda,
        rent_sysvar.minimum_balance(AuthorizationStateDictionary::LEN),
        AuthorizationStateDictionary::LEN.try_into().unwrap(),
        _program_id,
    );

    let seeds = &[
        from.key.as_ref(),
        _nonce.as_ref(),
        mint_account.key.as_ref(),
        b"WPOKT",
        b"authorization_dictionary_key",
        &[bump],
    ];
    program::invoke_signed(
        &ix,
        &[
            authorization_state_account.clone(),
            payer.clone(),
            system_account.clone(),
        ],
        &[seeds],
    )?;

    let mut auth_state_data = AuthorizationStateDictionary::unpack_from_slice(
        &mut &mut authorization_state_account.data.borrow_mut()[..],
    )?;
    auth_state_data.nonce = *_nonce;
    auth_state_data.authorization = false;
    auth_state_data.from = *_from;
    auth_state_data.pack_into_slice(&mut &mut authorization_state_account.data.borrow_mut()[..]);

    Ok(())
}

fn generate_wpokt_pda(program_id: &Pubkey, mint_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[mint_account.as_ref(), b"WPOKT", b"global_state_account"],
        program_id,
    )
}
