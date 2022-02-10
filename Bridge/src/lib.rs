pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(test)]
mod test {
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};
    use entrypoint::process_instruction;
    use instruction::BridgeInstruction;
    use solana_program::{account_info::AccountInfo, clock::Epoch, pubkey::Pubkey, msg};
    use state::{Bridge, ClaimedDictionary};
    use std::{mem, collections::BTreeMap};
    use solana_program::program_pack::Pack;
    
    #[test]
    fn test_construction() {
        let owner_account_key = Pubkey::new_unique();
        let bridge_account_key = Pubkey::new_unique();
        let claimed_account_key = Pubkey::new_unique();
        let program_id = Pubkey::default();
        // let mut bridge_account_data = vec![0; mem::size_of::<Bridge>()];
        // constructor instruction parameters
        let w_pokt_address = Pubkey::new_unique();
        let verify_address = Pubkey::new_unique();
        let stable_fee = 5;
        let chain_id = 5;

        let mut owner_account_lamports = 0;
        let mut owner_account_data = vec![0; mem::size_of::<u32>()];
        let owner_account = AccountInfo::new(
            &owner_account_key,
            true,
            false,
            &mut owner_account_lamports,
            &mut owner_account_data,
            &program_id,
            false,
            Epoch::default(),
        );

        let mut bridge_account_lamports = 0;
        msg!("Bridge Size: {}", mem::size_of::<Bridge>());
        let mut bridge_account_data = vec![0_u8; mem::size_of::<Bridge>()];
        let bridge_account = AccountInfo::new(
            &bridge_account_key,
            false,
            true,
            &mut bridge_account_lamports,
            &mut bridge_account_data,
            &program_id,
            false,
            Epoch::default(),
        );

        let mut claimed_account_lamports = 0;
        let mut claimed_account_data = vec![0; 10240];
        let claimed_account = AccountInfo::new(
            &claimed_account_key,
            false,
            true,
            &mut claimed_account_lamports,
            &mut claimed_account_data,
            &program_id,
            false,
            Epoch::default(),
        );

        let arguments = BridgeInstruction::Construct {
            w_pokt_address,
            verify_address,
            chain_id,
            stable_fee,
        };
        let instruction_data: Vec<u8> = arguments.try_to_vec().unwrap();
        let accounts = vec![owner_account, bridge_account, claimed_account];

        assert_eq!(
            Bridge::unpack_from_slice(&accounts[1].data.borrow())
                .unwrap()
                .is_initialized,
            false,
        );

        process_instruction(&program_id, &accounts, &instruction_data).unwrap();

        assert_eq!(
            Bridge::unpack_from_slice(&accounts[1].data.borrow())
                .unwrap()
                .is_initialized,
            true,
        );
        assert_eq!(
            Bridge::unpack_from_slice(&accounts[1].data.borrow())
                .unwrap()
                .owner,
            owner_account_key,
        );
        assert_eq!(
            Bridge::unpack_from_slice(&accounts[1].data.borrow())
                .unwrap()
                .chain_id,
            chain_id
        );

        // assert_eq!(
        //     ClaimedDictionary::try_from_slice(&accounts[2].data.borrow())
        //         .unwrap()
        //         .is_initialized,
        //     true
        // );

        // let ret =  *ClaimedDictionary::try_from_slice(&accounts[2].data.borrow())
        //         .unwrap()
        //         .claimed_dictionary
        //         .get(&1).unwrap();
        // assert_eq!(ret, true);
        // assert_eq!(
        //     ClaimedDictionary::try_from_slice(&accounts[2].data.borrow())
        //         .unwrap()
        //         .claimed_dictionary
        //         .get(&1)
        //         .unwrap()
        //         .get(&1)
        //         .unwrap(),
        //     true
        // )
    }
}
