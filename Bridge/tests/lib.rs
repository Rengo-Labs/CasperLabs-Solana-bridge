use borsh::BorshDeserialize;
use bridge::{
    entrypoint,
    instruction::BridgeInstruction,
    state::{Bridge, TokenAddedDictionary, TokenData, TokenListDictionary},
};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Signer,
    signer::keypair::Keypair,
    transaction::Transaction,
};
// use solana_program::program_pack::Pack;
use std::mem;

pub const LIB_NAME: &str = "bridge";

fn program_test(_program_id: Pubkey) -> ProgramTest {
    ProgramTest::new(
        LIB_NAME,
        _program_id,
        processor!(entrypoint::process_instruction),
    )
}

fn setup_program_test() -> (
    Pubkey,
    Pubkey,
    Keypair,
    Keypair,
    Keypair,
    Keypair,
    Keypair,
    Keypair,
    ProgramTest,
) {
    let program_id = Pubkey::new_unique();
    let system_program_id = Pubkey::new_from_array([1_u8; 32]);
    let owner_account_key = Keypair::new();
    let bridge_account_key = Keypair::new();
    let claimed_account_key = Keypair::new();
    let dtc_account_key = Keypair::new();
    let token_added_account_key = Keypair::new();
    let token_list_account_key = Keypair::new();

    let mut program_test = program_test(program_id);

    program_test.add_account(
        owner_account_key.pubkey(),
        Account {
            lamports: 5,
            data: vec![0_u8; mem::size_of::<u32>()],
            owner: system_program_id,
            ..Account::default()
        },
    );
    program_test.add_account(
        bridge_account_key.pubkey(),
        Account {
            lamports: 5,
            data: vec![0_u8; mem::size_of::<Bridge>()],
            owner: program_id,
            ..Account::default()
        },
    );

    let dictionaries = vec![
        claimed_account_key.pubkey(),
        dtc_account_key.pubkey(),
        token_added_account_key.pubkey(),
        token_list_account_key.pubkey(),
    ];

    // initialize all dictionary accounts
    for account in dictionaries.iter() {
        program_test.add_account(
            *account,
            Account {
                lamports: 10,
                data: vec![0_u8; 10240],
                owner: program_id,
                ..Account::default()
            },
        );
    }

    (
        program_id,
        system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    )
}

#[tokio::test]
async fn test_setup_program_test() {
    let (
        program_id,
        _system_program_id,
        _owner_account_key,
        bridge_account_key,
        _claimed_account_key,
        _dtc_account_key,
       _token_added_account_key,
        _token_list_account_key,
        program_test,
    ) = setup_program_test();
    let (mut banks_client, _payer, _recent_blockhash) = program_test.start().await;

    let bridge_account: Account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");

    assert_eq!(bridge_account.owner, program_id);
}

async fn send_instruction(
    _program_id: &Pubkey,
    _accounts: &[(&Keypair, bool)],
    _instruction_args: &BridgeInstruction,
    _recent_blockhash: Hash,
    _banks_client: &mut BanksClient,
) {
    let mut _account_metas: Vec<AccountMeta> = Vec::new();
    let mut _signers: Vec<&Keypair> = Vec::new();

    _signers.push(_accounts[0].0);
    for item in _accounts[1..].iter() {
        _account_metas.push(AccountMeta::new(item.0.pubkey(), item.1));
        if item.1 == true {
            _signers.push(item.0);
        }
    }

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            *_program_id,
            &_instruction_args,
            _account_metas,
        )],
        Some(&_accounts[0].0.pubkey()),
    );
    transaction.sign(&_signers, _recent_blockhash);
    _banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_instruction_construct() {
    let (
        program_id,
        _system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    ): (
        Pubkey,
        Pubkey,
        Keypair,
        Keypair,
        Keypair,
        Keypair,
        Keypair,
        Keypair,
        ProgramTest,
    ) = setup_program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // verify accounts as uninitialized
    let bridge_account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");

    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data).unwrap();
    assert_eq!(bridge_data.is_initialized, false);
    assert_eq!(bridge_data.owner, Pubkey::new_from_array([0_u8; 32]));
    // create construction transaction
    let w_pokt_address = Pubkey::new_unique();
    let verify_address = Pubkey::new_unique();
    let chain_id: u64 = 90;
    let stable_fee: u64 = 180;
    let instruction_args = BridgeInstruction::Construct {
        w_pokt_address,
        verify_address,
        chain_id,
        stable_fee,
    };
    send_instruction(
        &program_id,
        &[
            (&payer, true),
            (&owner_account_key, true),
            (&bridge_account_key, true),
            (&claimed_account_key, true),
            (&dtc_account_key, true),
            (&token_added_account_key, true),
            (&token_list_account_key, true),
        ],
        &instruction_args,
        recent_blockhash,
        &mut banks_client,
    )
    .await;

    let bridge_account: Account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");
    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data).unwrap();
    assert_eq!(bridge_data.owner, owner_account_key.pubkey());
    assert_eq!(bridge_data.chain_id, chain_id);
    assert_eq!(bridge_data.stable_fee, stable_fee);

    let token_added_account: Account = banks_client
        .get_account(token_added_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("token_added_account not found.");

    let token_added_data =
        TokenAddedDictionary::unpack_from_slice(&token_added_account.data).unwrap();
    let ret: &bool = token_added_data
        .token_added_dictionary
        .get(&w_pokt_address)
        .unwrap();
    assert_eq!(*ret, true);

    let token_list_account: Account = banks_client
        .get_account(token_list_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("token_list_account not found.");
    let token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data).unwrap();
    let ret: &Vec<u8> = token_list_data.token_list_dictionary.get(&1).unwrap();
    let ret: TokenData = TokenData::try_from_slice(ret).unwrap();
    assert_ne!(ret.limit_timestamp, 0);
    assert_eq!(ret.token_address, w_pokt_address);
}

#[tokio::test]
async fn test_instruction_update_verify_address() {
    let w_pokt_address = Pubkey::new_unique();
    let verify_address = Pubkey::new_unique();
    let chain_id: u64 = 90;
    let stable_fee: u64 = 180;

    let (
        program_id,
        _system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    ) = setup_program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Construct bridge first
    let instruction_args = BridgeInstruction::Construct {
        w_pokt_address: w_pokt_address,
        verify_address: verify_address,
        chain_id: chain_id,
        stable_fee: stable_fee,
    };
    send_instruction(
        &program_id,
        &[
            (&payer, true),
            (&owner_account_key, true),
            (&bridge_account_key, true),
            (&claimed_account_key, true),
            (&dtc_account_key, true),
            (&token_added_account_key, true),
            (&token_list_account_key, true),
        ],
        &instruction_args,
        recent_blockhash,
        &mut banks_client,
    )
    .await;

    let bridge_account: Account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");
    // let bridge_data = Bridge::unpack_from_slice(&bridge_account.data).unwrap();

    let _verify_address: Pubkey = Pubkey::new_unique();
    // run test instruction
    let instruction_args: BridgeInstruction = BridgeInstruction::UpdateVerifyAddressOnlyOwner {
        verify_address: _verify_address,
    };
    send_instruction(
        &program_id,
        &[
            (&payer, true),
            (&owner_account_key, true),
            (&bridge_account_key, true),
        ],
        &instruction_args,
        recent_blockhash,
        &mut banks_client,
    )
    .await;

    let bridge_account: Account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");
    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data).unwrap();

    assert_ne!(bridge_data.verify_address, verify_address);
    assert_eq!(bridge_data.verify_address, _verify_address);
}

#[tokio::test]
async fn test_instruction_update_token_limit() {
    let w_pokt_address = Pubkey::new_unique();
    let verify_address = Pubkey::new_unique();
    let chain_id: u64 = 90;
    let stable_fee: u64 = 180;

    let (
        program_id,
        _system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    ) = setup_program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Construct bridge first
    let instruction_args = BridgeInstruction::Construct {
        w_pokt_address: w_pokt_address,
        verify_address: verify_address,
        chain_id: chain_id,
        stable_fee: stable_fee,
    };
    send_instruction(
        &program_id,
        &[
            (&payer, true),
            (&owner_account_key, true),
            (&bridge_account_key, true),
            (&claimed_account_key, true),
            (&dtc_account_key, true),
            (&token_added_account_key, true),
            (&token_list_account_key, true),
        ],
        &instruction_args,
        recent_blockhash,
        &mut banks_client,
    )
    .await;

    // test transaction
    let token_index: u64 = 1;
    let limit: u64 = 1000000;
    let instruction_args = BridgeInstruction::UpdateTokenLimitOnlyOwner { token_index, limit };
    send_instruction(
        &program_id,
        &[
            (&payer, true),
            (&owner_account_key, true),
            (&bridge_account_key, true),
            (&token_list_account_key, true),
        ],
        &instruction_args,
        recent_blockhash,
        &mut banks_client,
    )
    .await;

    let token_list_account: Account = banks_client
        .get_account(token_list_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("token_list_account not found.");
    let token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data).unwrap();

    let ret: &Vec<u8> = token_list_data
        .token_list_dictionary
        .get(&token_index)
        .unwrap();
    let ret: TokenData = TokenData::try_from_slice(ret).unwrap();

    assert_eq!(ret.limit, limit);
}

#[tokio::test]
async fn test_instruction_set_token_limit_time() {
    let w_pokt_address = Pubkey::new_unique();
    let verify_address = Pubkey::new_unique();
    let chain_id: u64 = 90;
    let stable_fee: u64 = 180;

    let (
        program_id,
        _system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    ) = setup_program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Construct bridge first
    let instruction_args = BridgeInstruction::Construct {
        w_pokt_address: w_pokt_address,
        verify_address: verify_address,
        chain_id: chain_id,
        stable_fee: stable_fee,
    };
    send_instruction(
        &program_id,
        &[
            (&payer, true),
            (&owner_account_key, true),
            (&bridge_account_key, true),
            (&claimed_account_key, true),
            (&dtc_account_key, true),
            (&token_added_account_key, true),
            (&token_list_account_key, true),
        ],
        &instruction_args,
        recent_blockhash,
        &mut banks_client,
    )
    .await;

    // test transaction
    let token_index: u64 = 1;
    let timestamp: u64 = 1234;
    let instruction_args = BridgeInstruction::SetTokenLimitTimeOnlyOwner {
        token_index,
        timestamp,
    };

    send_instruction(
        &program_id,
        &[
            (&payer, true),
            (&owner_account_key, true),
            (&bridge_account_key, true),
            (&token_list_account_key, true),
        ],
        &instruction_args,
        recent_blockhash,
        &mut banks_client,
    )
    .await;

    let token_list_account: Account = banks_client
        .get_account(token_list_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("token_list_account not found.");
    let token_list_data =
        TokenListDictionary::unpack_from_slice(&token_list_account.data).unwrap();

    let ret: &Vec<u8> = token_list_data
        .token_list_dictionary
        .get(&token_index)
        .unwrap();
    let ret: TokenData = TokenData::try_from_slice(ret).unwrap();

    assert_eq!(ret.limit_timestamp, timestamp);
}

#[tokio::test]
async fn test_instruction_update_stable_fee() {
    let w_pokt_address = Pubkey::new_unique();
    let verify_address = Pubkey::new_unique();
    let chain_id: u64 = 90;
    let stable_fee: u64 = 180;

    let (
        program_id,
        _system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    ) = setup_program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Construct bridge first
    let instruction_args = BridgeInstruction::Construct {
        w_pokt_address: w_pokt_address,
        verify_address: verify_address,
        chain_id: chain_id,
        stable_fee: stable_fee,
    };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction_args,
            vec![
                AccountMeta::new(owner_account_key.pubkey(), true),
                AccountMeta::new(bridge_account_key.pubkey(), true),
                AccountMeta::new(claimed_account_key.pubkey(), true),
                AccountMeta::new(dtc_account_key.pubkey(), true),
                AccountMeta::new(token_added_account_key.pubkey(), true),
                AccountMeta::new(token_list_account_key.pubkey(), true),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(
        &vec![
            &payer,
            &owner_account_key,
            &bridge_account_key,
            &claimed_account_key,
            &dtc_account_key,
            &token_added_account_key,
            &token_list_account_key,
        ],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let bridge_account: Account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");
    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data).unwrap();

    let current_stable_fee = bridge_data.new_stable_fee;
    let current_stable_fee_update_time = bridge_data.stable_fee_update_time;

    let new_stable_fee: u64 = 420;
    let instruction_args = BridgeInstruction::UpdateStableFeeOnlyOwner { new_stable_fee };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction_args,
            vec![
                AccountMeta::new(owner_account_key.pubkey(), true),
                AccountMeta::new(bridge_account_key.pubkey(), true),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(
        &[&payer, &owner_account_key, &bridge_account_key],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let bridge_account: Account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");
    let bridge_data = Bridge::unpack_from_slice(&bridge_account.data).unwrap();

    assert_ne!(bridge_data.new_stable_fee, current_stable_fee);
    assert_ne!(
        bridge_data.stable_fee_update_time,
        current_stable_fee_update_time
    );
    assert_eq!(bridge_data.new_stable_fee, new_stable_fee);
}

#[tokio::test]
async fn test_instruction_update_token_fee() {
    let w_pokt_address = Pubkey::new_unique();
    let verify_address = Pubkey::new_unique();
    let chain_id: u64 = 90;
    let stable_fee: u64 = 180;
    let index: u64 = 1;
    let new_token_fee: u64 = 1000;

    let (
        program_id,
        _system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    ) = setup_program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Construct bridge first
    let instruction_args = BridgeInstruction::Construct {
        w_pokt_address: w_pokt_address,
        verify_address: verify_address,
        chain_id: chain_id,
        stable_fee: stable_fee,
    };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction_args,
            vec![
                AccountMeta::new(owner_account_key.pubkey(), true),
                AccountMeta::new(bridge_account_key.pubkey(), true),
                AccountMeta::new(claimed_account_key.pubkey(), true),
                AccountMeta::new(dtc_account_key.pubkey(), true),
                AccountMeta::new(token_added_account_key.pubkey(), true),
                AccountMeta::new(token_list_account_key.pubkey(), true),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(
        &vec![
            &payer,
            &owner_account_key,
            &bridge_account_key,
            &claimed_account_key,
            &dtc_account_key,
            &token_added_account_key,
            &token_list_account_key,
        ],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let token_list_account: Account = banks_client
        .get_account(token_list_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("token_list_account not found.");
    let token_list_data = TokenListDictionary::unpack_from_slice(&token_list_account.data).unwrap();

    let ret: &Vec<u8> = token_list_data.token_list_dictionary.get(&index).unwrap();
    let ret: TokenData = TokenData::try_from_slice(ret).unwrap();
    let current_new_token_fee = ret.new_fee;

    // test transaction
    let instruction_args = BridgeInstruction::UpdateTokenFeeOnlyOwner {
        index,
        new_token_fee,
    };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction_args,
            vec![
                AccountMeta::new(owner_account_key.pubkey(), true),
                AccountMeta::new_readonly(bridge_account_key.pubkey(), true),
                AccountMeta::new(token_list_account_key.pubkey(), true),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(
        &[
            &payer,
            &owner_account_key,
            &bridge_account_key,
            &token_list_account_key,
        ],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let token_list_account: Account = banks_client
        .get_account(token_list_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("token_list_account not found.");
    let token_list_data = TokenListDictionary::unpack_from_slice(&token_list_account.data).unwrap();

    let ret: &Vec<u8> = token_list_data.token_list_dictionary.get(&index).unwrap();
    let ret: TokenData = TokenData::try_from_slice(ret).unwrap();

    assert_ne!(ret.new_fee, current_new_token_fee);
    assert_eq!(ret.new_fee, new_token_fee);
}

#[tokio::test]
async fn test_instruction_update_fees() {
    let w_pokt_address = Pubkey::new_unique();
    let verify_address = Pubkey::new_unique();
    let chain_id: u64 = 90;
    let stable_fee: u64 = 180;
    let token_index: u64 = 1;

    let (
        program_id,
        _system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    ) = setup_program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Construct bridge first
    let instruction_args = BridgeInstruction::Construct {
        w_pokt_address: w_pokt_address,
        verify_address: verify_address,
        chain_id: chain_id,
        stable_fee: stable_fee,
    };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction_args,
            vec![
                AccountMeta::new(owner_account_key.pubkey(), true),
                AccountMeta::new(bridge_account_key.pubkey(), true),
                AccountMeta::new(claimed_account_key.pubkey(), true),
                AccountMeta::new(dtc_account_key.pubkey(), true),
                AccountMeta::new(token_added_account_key.pubkey(), true),
                AccountMeta::new(token_list_account_key.pubkey(), true),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(
        &vec![
            &payer,
            &owner_account_key,
            &bridge_account_key,
            &claimed_account_key,
            &dtc_account_key,
            &token_added_account_key,
            &token_list_account_key,
        ],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let bridge_account: Account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");
    // let bridge_data = Bridge::unpack_from_slice(&bridge_account.data).unwrap();

    let token_list_data: TokenListDictionary = banks_client
        .get_packed_account_data(token_list_account_key.pubkey())
        .await
        .expect("token list data error.");

    let current_token_data: &Vec<u8> = token_list_data
        .token_list_dictionary
        .get(&token_index)
        .unwrap();
    let current_token_data: TokenData = TokenData::try_from_slice(current_token_data).unwrap();

    let instruction_args = BridgeInstruction::UpdateFees { token_index };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction_args,
            vec![
                AccountMeta::new(bridge_account_key.pubkey(), false),
                AccountMeta::new(token_list_account_key.pubkey(), false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let bridge_data: Bridge = banks_client
        .get_packed_account_data(bridge_account_key.pubkey())
        .await
        .expect("get_packed_account_data for Bridge failed.");

    assert_eq!(bridge_data.stable_fee, stable_fee); // unchanged as stable_fee_update_time is 0 as initialized

    let token_list_data: TokenListDictionary = banks_client
        .get_packed_account_data(token_list_account_key.pubkey())
        .await
        .expect("token list data error.");

    let token_data: &Vec<u8> = token_list_data
        .token_list_dictionary
        .get(&token_index)
        .unwrap();
    let token_data: TokenData = TokenData::try_from_slice(token_data).unwrap();

    assert_eq!(token_data.fee, current_token_data.fee);
}

#[tokio::test]
async fn test_instruction_add_token() {
    let w_pokt_address = Pubkey::new_unique();
    let verify_address = Pubkey::new_unique();
    let chain_id: u64 = 90;
    let stable_fee: u64 = 180;

    let (
        program_id,
        _system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    ) = setup_program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Construct bridge first
    let instruction_args = BridgeInstruction::Construct {
        w_pokt_address: w_pokt_address,
        verify_address: verify_address,
        chain_id: chain_id,
        stable_fee: stable_fee,
    };
    send_instruction(
        &program_id,
        &[
            (&payer, true),
            (&owner_account_key, true),
            (&bridge_account_key, true),
            (&claimed_account_key, true),
            (&dtc_account_key, true),
            (&token_added_account_key, true),
            (&token_list_account_key, true),
        ],
        &instruction_args,
        recent_blockhash,
        &mut banks_client,
    )
    .await;

    let index: u64 = 10;
    let token_address: Pubkey = Pubkey::new_unique();
    let fee: u64 = 11;
    let limit: u64 = 12;

    let instruction_args = BridgeInstruction::AddTokenOnlyOwner {
        index,
        token_address,
        fee,
        limit,
    };

    send_instruction(
        &program_id,
        &[
            (&payer, true),
            (&owner_account_key, true),
            (&bridge_account_key, true),
            (&token_list_account_key, true),
            (&token_added_account_key, true),
        ],
        &instruction_args,
        recent_blockhash,
        &mut banks_client,
    )
    .await;

    let token_list_data: TokenListDictionary = banks_client
        .get_packed_account_data(token_list_account_key.pubkey())
        .await
        .expect("token_list_data query error.");

    let token_added_data: TokenAddedDictionary = banks_client
        .get_packed_account_data(token_added_account_key.pubkey())
        .await
        .expect("token_added_data query error.");

    let token_data: TokenData =
        TokenData::try_from_slice(token_list_data.token_list_dictionary.get(&index).unwrap())
            .unwrap();

    let addition: bool = *token_added_data
        .token_added_dictionary
        .get(&token_address)
        .unwrap();

    assert_eq!(addition, true);
    assert_eq!(token_data.fee, fee);
    assert_eq!(token_data.limit, limit);
}
