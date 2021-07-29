pub mod error;
pub mod processor;
pub mod state;
pub mod instruction;

pub use solana_program;

use crate::{error::TokenError, processor::Processor};
use solana_program::{
    entrypoint::ProgramResult, pubkey::Pubkey,
    account_info::AccountInfo, entrypoint, program_error::PrintProgramError,
};

solana_program::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        error.print::<TokenError>();
        return Err(error);
    }
    Ok(())
}
