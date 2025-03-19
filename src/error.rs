use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the DAO program
#[derive(Error, Debug, Copy, Clone)]
pub enum DaoError {
    /// Invalid instruction data passed to program
    #[error("Invalid instruction data")]
    InvalidInstructionData,
    
    /// Not enough accounts provided to the instruction
    #[error("Not enough accounts")]
    NotEnoughAccounts,
    
    /// Account is not a signer when it should be
    #[error("Expected a signed account")]
    ExpectedSigner,
    
    /// The account has already voted on this proposal
    #[error("Account has already voted")]
    AlreadyVoted,
    
    /// The voting period for this proposal has ended
    #[error("Voting period has ended")]
    VotingEnded,
    
    /// The proposal is not ready to be finalized (voting still active)
    #[error("Voting still active")]
    VotingStillActive,
    
    /// The proposal has already been finalized
    #[error("Proposal already finalized")]
    ProposalFinalized,
}

impl From<DaoError> for ProgramError {
    fn from(e: DaoError) -> Self {
        ProgramError::Custom(e as u32)
    }
} 