use crate::instruction::WPoktInstruction;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

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
            WPoktInstruction::Construct => {}
            WPoktInstruction::SetBridgeOnlyOwner => {}
            WPoktInstruction::MintOnlyBridge => {}
            WPoktInstruction::Burn => {}
        }
        Ok(())
    }
}
