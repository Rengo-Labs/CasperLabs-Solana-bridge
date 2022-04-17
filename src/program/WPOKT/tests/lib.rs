use borsh::BorshDeserialize;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Signer,
    signer::keypair::Keypair,
    system_program,
    transaction::Transaction,
};
use wpokt;
// use solana_program::program_pack::Pack;
use spl_token_2022;
use std::mem;

pub const LIB_NAME: &str = "wpokt";
pub const SPL_TOKEN_2022_LIB_NAME: &str = "spl_token_2022";

// async fn send_instruction<T>(
//     _program_id: &Pubkey,
//     _accounts: &[(&Keypair, bool)],
//     _instruction_args: &BridgeInstruction,
//     _recent_blockhash: Hash,
//     _banks_client: &mut BanksClient,
// ) where
//     T: Pack,
// {
//     let mut _account_metas: Vec<AccountMeta> = Vec::new();
//     let mut _signers: Vec<&Keypair> = Vec::new();

//     _signers.push(_accounts[0].0);
//     for item in _accounts[1..].iter() {
//         _account_metas.push(AccountMeta::new(item.0.pubkey(), item.1));
//         if item.1 == true {
//             _signers.push(item.0);
//         }
//     }

//     let mut transaction = Transaction::new_with_payer(
//         &[Instruction::new_with_borsh(
//             *_program_id,
//             &_instruction_args,
//             _account_metas,
//         )],
//         Some(&_accounts[0].0.pubkey()),
//     );
//     transaction.sign(&_signers, _recent_blockhash);
//     _banks_client
//         .process_transaction(transaction)
//         .await
//         .unwrap();
// }

fn setup_program_test() -> (Pubkey, ProgramTest) {
    let wpokt_program_id = Pubkey::new_unique();
    let system_program_id = Pubkey::new_from_array([1_u8; 32]);
    let mint_account_key = Keypair::new();

    let mut program_test = ProgramTest::new(
        LIB_NAME,
        wpokt_program_id,
        processor!(wpokt::entrypoint::process_instruction),
    );
    let mut program_test = ProgramTest::default();
    program_test.add_program(
        SPL_TOKEN_2022_LIB_NAME,
        spl_token_2022::id(),
        processor!(spl_token_2022::processor::Processor::process),
    );

    (wpokt_program_id, program_test)
}

// #[tokio::test]
// async fn test_program_test_creation() {
//     let (_, program_test) = setup_program_test();
//     let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

//     let spl_token_2022_account: Account = banks_client
//         .get_account(spl_token_2022::id())
//         .await
//         .expect("get_account")
//         .expect("token acc not found");
//     assert_eq!(spl_token_2022_account.executable, true);
// }

// #[tokio::test]
// async fn test_wpokt_instruction_construct() {
//     let (wpokt_program_id, mut program_test) = setup_program_test();
//     let wpokt_account = Keypair::new();
//     let mint = Keypair::new();
//     let initial_minter = Keypair::new();

//     program_test.add_account(
//         mint.pubkey(),
//         Account {
//             lamports: 1000000,
//             owner: spl_token_2022::id(),
//             data: vec![0_u8; spl_token_2022::state::Mint::LEN],
//             ..Account::default()
//         },
//     );
//     program_test.add_account(
//         wpokt_account.pubkey(),
//         Account {
//             lamports: 1000000,
//             owner: wpokt_program_id,
//             data: vec![0_u8; wpokt::state::WPOKT::LEN],
//             ..Account::default()
//         },
//     );
//     program_test.add_account(
//         initial_minter.pubkey(),
//         Account {
//             lamports: 1000000,
//             ..Account::default()
//         },
//     );
//     let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
//     // setup tx
//     let instruction_args = wpokt::instruction::WPOKTInstruction::Construct {
//         initial_minter: initial_minter.pubkey(),
//     };
//     let mut tx = Transaction::new_with_payer(
//         &[Instruction::new_with_borsh(
//             wpokt_program_id,
//             &instruction_args,
//             vec![
//                 AccountMeta::new(mint.pubkey(), false),
//                 AccountMeta::new(wpokt_account.pubkey(), false),
//             ],
//         )],
//         Some(&payer.pubkey()),
//     );
//     tx.sign(&vec![&payer], recent_blockhash);
//     // send tx
//     banks_client.process_transaction(tx).await.unwrap();
//     // banks_client.send_transaction_with_context(
//     //     program_test_context,
//     //     tx
//     // ).await;
//     // let mint_account: Account = banks_client
//     //     .get_account(mint.pubkey())
//     //     .await
//     //     .expect("get_account")
//     //     .expect("mint acc not found");

//     // let mint_acc_data = spl_token_2022::state::Mint::unpack_from_slice(&mint_account.data).unwrap();
//     // assert_eq!(
//     //     mint_acc_data
//     //         .mint_authority
//     //         .contains(&initial_minter.pubkey()),
//     //     true
//     // );
// }
// #[tokio::test]
// async fn test_setup_program_test() {
//     let (
//         program_id,
//         _system_program_id,
//         _owner_account_key,
//         bridge_account_key,
//         _claimed_account_key,
//         _dtc_account_key,
//         _token_added_account_key,
//         _token_list_account_key,
//         program_test,
//     ) = setup_program_test();
//     let (mut banks_client, _payer, _recent_blockhash) = program_test.start().await;

//     let bridge_account: Account = banks_client
//         .get_account(bridge_account_key.pubkey())
//         .await
//         .expect("get_account")
//         .expect("bridge_account not found.");

//     assert_eq!(bridge_account.owner, program_id);
// }
