use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

/// Instructions supported by the DAO program
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum DaoInstruction {
    /// Creates a new proposal
    /// 
    /// Accounts expected:
    /// 0. `[signer]` Proposal creator
    /// 1. `[writable]` Proposal account (PDA)
    /// 2. `[writable]` System program
    CreateProposal {
        /// Title of the proposal
        title: String,
        /// Description of the proposal 
        description: String,
        /// Duration in seconds for the voting period
        voting_period: u64,
    },

    /// Vote on an existing proposal
    /// 
    /// Accounts expected:
    /// 0. `[signer]` Voter account
    /// 1. `[writable]` Proposal account (PDA)
    Vote {
        /// true for yes, false for no
        approve: bool,
    },

    /// Finalize a proposal after voting period ends
    /// 
    /// Accounts expected:
    /// 0. `[signer]` Any account (could be restricted to admin)
    /// 1. `[writable]` Proposal account (PDA)
    FinalizeProposal {},

    /// Withdraw SOL from the program
    /// 
    /// Accounts expected:
    /// 0. `[signer]` Administrator or authorized account
    /// 1. `[writable]` Recipient account
    /// 2. `[writable]` Treasury PDA account
    /// 3. `[]` System program
    WithdrawSol {
        /// Amount of lamports to withdraw
        amount: u64,
    },
    
    /// Initialize the treasury account for holding funds
    /// 
    /// Accounts expected:
    /// 0. `[signer]` Account paying for initialization (admin)
    /// 1. `[writable]` Treasury PDA to be created
    /// 2. `[]` System program
    InitializeTreasury {},
}

impl DaoInstruction {
    /// Unpacks a byte buffer into a DaoInstruction
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        
        // The first byte is the instruction tag
        let tag = input[0];
        // The rest is the serialized data
        let data = &input[1..];
        
        match tag {
            0 => {
                // CreateProposal instruction
                let args = CreateProposalArgs::try_from_slice(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                
                Ok(DaoInstruction::CreateProposal {
                    title: args.title,
                    description: args.description,
                    voting_period: args.voting_period,
                })
            },
            1 => {
                // Vote instruction
                let args = VoteArgs::try_from_slice(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                
                Ok(DaoInstruction::Vote {
                    approve: args.approve,
                })
            },
            2 => {
                // FinalizeProposal instruction (no arguments)
                Ok(DaoInstruction::FinalizeProposal {})
            },
            3 => {
                // WithdrawSol instruction
                let args = WithdrawSolArgs::try_from_slice(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                
                Ok(DaoInstruction::WithdrawSol {
                    amount: args.amount,
                })
            },
            4 => {
                // InitializeTreasury instruction (no arguments)
                Ok(DaoInstruction::InitializeTreasury {})
            },
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Struct for CreateProposal arguments serialization
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct CreateProposalArgs {
    pub title: String,
    pub description: String,
    pub voting_period: u64,
}

/// Struct for Vote arguments serialization
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct VoteArgs {
    pub approve: bool,
} 

/// Struct for WithdrawSol arguments serialization
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct WithdrawSolArgs {
    pub amount: u64,
} 