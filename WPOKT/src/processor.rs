use crate::instruction::WPOKTInstruction;
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
        let instruction = WPOKTInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        match instruction {
            WPOKTInstruction::Construct => {}
            WPOKTInstruction::GetChainId => {}
            WPOKTInstruction::GetDomainSeparator => {}
            WPOKTInstruction::MintOnlyMinter => {}
            WPOKTInstruction::ChangeMinterOnlyMinter => {}
            WPOKTInstruction::Burn => {}
            WPOKTInstruction::Approve => {}
            WPOKTInstruction::Transfer => {}
            WPOKTInstruction::TransferFrom => {}
            WPOKTInstruction::Permit => {}
            WPOKTInstruction::TransferWithAuthorization => {}
        }
        Ok(())
    }
}
