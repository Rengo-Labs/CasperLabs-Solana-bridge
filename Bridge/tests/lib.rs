use borsh::{BorshDeserialize, BorshSerialize};
use bridge::{
    entrypoint,
    instruction::BridgeInstruction,
    processor,
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
        system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
    ) = setup_program_test();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let bridge_account: Account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");

    assert_eq!(bridge_account.owner, program_id);
}

impl From<Infallible> for () {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[tokio::test]
async fn test_instruction_construct(){
    let (
        program_id,
        system_program_id,
        owner_account_key,
        bridge_account_key,
        claimed_account_key,
        dtc_account_key,
        token_added_account_key,
        token_list_account_key,
        program_test,
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
    let w_pokt_address =  Pubkey::new_unique();
    let verify_address =  Pubkey::new_unique();
    let chain_id: u64 = 90;
    let stable_fee: u64 = 180;
    let constructor_args = BridgeInstruction::Construct {
        w_pokt_address,
        verify_address,
        chain_id,
        stable_fee,
    };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &constructor_args,
            vec![
                AccountMeta::new(owner_account_key.pubkey(), true),
                AccountMeta::new(bridge_account_key.pubkey(), false),
                AccountMeta::new(claimed_account_key.pubkey(), false),
                AccountMeta::new(dtc_account_key.pubkey(), false),
                AccountMeta::new(token_added_account_key.pubkey(), false),
                AccountMeta::new(token_list_account_key.pubkey(), false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &owner_account_key], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let bridge_account: Account = banks_client
        .get_account(bridge_account_key.pubkey())
        .await
        .expect("get_account")
        .expect("bridge_account not found.");
    let mut bridge_data = Bridge::unpack_from_slice(&bridge_account.data).unwrap();
    assert_eq!(bridge_data.owner, owner_account_key.pubkey());
    assert_eq!(bridge_data.chain_id, chain_id);
    assert_eq!(bridge_data.stable_fee, stable_fee);

    let token_added_account: Account = banks_client
        .get_account(bridge_data.token_added_dictionary)
        .await
        .expect("get_account")
        .expect("token_added_account not found.");
    let mut token_added_data = TokenAddedDictionary::unpack_from_slice(&token_added_account.data).unwrap();
    let ret: &bool = token_added_data
        .token_added_dictionary
        .get(&w_pokt_address)
        .unwrap();
    assert_eq!(*ret, true);

    let token_list_account: Account = banks_client
        .get_account(bridge_data.token_list_dictionary)
        .await
        .expect("get_account")
        .expect("token_list_account not found.");
    let mut token_list_data = TokenListDictionary::unpack_from_slice(&token_list_account.data).unwrap();
    let ret: &Vec<u8> = token_list_data.token_list_dictionary.get(&1).unwrap();
    let ret: TokenData = TokenData::try_from_slice(ret).unwrap();
    assert_ne!(ret.limit_timestamp, 0);
    assert_eq!(ret.token_address, w_pokt_address);
}
