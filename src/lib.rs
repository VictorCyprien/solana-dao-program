use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

// This is the entry point of your program that Solana will call
entrypoint!(process_instruction);

pub mod instruction;
pub mod processor;
pub mod state;
pub mod error;

// Main program entry point function
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("DAO program entrypoint");
    
    // Call the processor to handle the instruction
    processor::Processor::process(program_id, accounts, instruction_data)
}

#[cfg(test)]
mod tests {
    // Tests will be added here
}
