use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    clock::UnixTimestamp,
    pubkey::Pubkey,
};

/// Status of a proposal
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub enum ProposalStatus {
    /// Proposal is active and accepting votes
    Active,
    /// Proposal has passed (more yes than no votes)
    Passed,
    /// Proposal has failed (more no than yes votes or tied)
    Failed,
}

/// Data structure for a proposal
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Proposal {
    /// The proposal creator's public key
    pub creator: Pubkey,
    /// Title of the proposal
    pub title: String,
    /// Description of the proposal
    pub description: String,
    /// When the proposal was created
    pub created_at: UnixTimestamp,
    /// When voting ends
    pub ends_at: UnixTimestamp,
    /// Number of yes votes
    pub yes_votes: u64,
    /// Number of no votes
    pub no_votes: u64,
    /// Current status of the proposal
    pub status: ProposalStatus,
    /// Accounts that have already voted (we would normally use a separate account for this)
    pub voters: Vec<Pubkey>,
} 