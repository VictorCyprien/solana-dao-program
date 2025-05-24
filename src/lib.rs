use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
    clock::Clock,
};
use std::str::FromStr;

// Program entrypoint
entrypoint!(process_instruction);

// Fee address where funds will be sent
const FEE_ADDRESS: &str = "BAGek78CDYQ8phuDqNk7sQzD7LdJeKkb7jD4y2AyR3tJ";
// Fee in USD for creating a DAO
const CREATE_DAO_FEE_USD: u64 = 20; // $20 USD

// Error codes specific to this program
#[derive(Debug, thiserror::Error)]
pub enum DaoError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Not Rent Exempt")]
    NotRentExempt,
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    #[error("Invalid Fee Account")]
    InvalidFeeAccount,
    #[error("Proposal Time Invalid")]
    ProposalTimeInvalid,
    #[error("Invalid Vote")]
    InvalidVote,
    #[error("Insufficient Funds")]
    InsufficientFunds,
    #[error("Invalid SOL Price")]
    InvalidSolPrice,
}

impl From<DaoError> for ProgramError {
    fn from(e: DaoError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// Program instructions
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum DaoInstruction {
    /// Create a new DAO
    /// 
    /// Accounts:
    /// 0. `[signer]` Creator account
    /// 1. `[writable]` New DAO account
    /// 2. `[]` System program
    /// 3. `[writable]` Fee recipient account
    CreateDao {
        name: String,
        description: String,
        discord_server: String,
        twitter: String,
        telegram: String,
        instagram: String,
        tiktok: String,
        website: String,
        treasury: String,
        profile: String,
        token_address: String,
        sol_price_usd: u64, // Current SOL price in USD cents (e.g., 10000 = $100.00)
    },
    
    /// Create a new proposal
    /// 
    /// Accounts:
    /// 0. `[signer]` Creator account
    /// 1. `[writable]` New proposal account
    /// 2. `[]` DAO account
    /// 3. `[]` System program
    /// 4. `[writable]` Fee recipient account (optional)
    CreateProposal {
        name: String,
        description: String,
        dao_id: String,
        pod_id: String, // Can be empty
        start_time: i64,
        end_time: i64,
    },
    
    /// Vote on a proposal
    /// 
    /// Accounts:
    /// 0. `[signer]` Voter account
    /// 1. `[writable]` New vote account
    /// 2. `[]` Proposal account
    /// 3. `[]` System program
    /// 4. `[writable]` Fee recipient account (optional)
    Vote {
        vote: String,
        proposal_id: String,
    },

    /// Feature a DAO (paid advertisement)
    /// 
    /// Accounts:
    /// 0. `[signer]` Creator account
    /// 1. `[writable]` New featured account
    /// 2. `[]` DAO account
    /// 3. `[]` System program
    /// 4. `[writable]` Fee recipient account
    Featured {
        dao_id: String,
        sol_price_usd: u64, // Current SOL price in USD cents
    },
    
    /// Enable DAO modules
    /// 
    /// Accounts:
    /// 0. `[signer]` Creator account
    /// 1. `[writable]` New module account
    /// 2. `[]` DAO account
    /// 3. `[]` System program
    /// 4. `[writable]` Fee recipient account
    Modules {
        dao_id: String,
        module_type: String, // "POD" or "POL"
        sol_price_usd: u64, // Current SOL price in USD cents
    },
}

// DAO account data structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Dao {
    pub authority: Pubkey,
    pub name: String,
    pub description: String,
    pub discord_server: String,
    pub twitter: String,
    pub telegram: String,
    pub instagram: String,
    pub tiktok: String,
    pub website: String,
    pub treasury: String,
    pub profile: String,
    pub token_address: String,
}

// Proposal account data structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Proposal {
    pub authority: Pubkey,
    pub name: String,
    pub description: String,
    pub dao_id: String,
    pub pod_id: String,
    pub start_time: i64,
    pub end_time: i64,
}

// Vote account data structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Vote {
    pub voter: Pubkey,
    pub vote: String,
    pub proposal_id: String,
}

// Featured account data structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Featured {
    pub authority: Pubkey,
    pub dao_id: String,
}

// Module account data structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Module {
    pub authority: Pubkey,
    pub dao_id: String,
    pub module_type: String,
}

// Program entrypoint processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = DaoInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        DaoInstruction::CreateDao {
            name,
            description,
            discord_server,
            twitter,
            telegram,
            instagram,
            tiktok,
            website,
            treasury,
            profile,
            token_address,
            sol_price_usd,
        } => {
            process_create_dao(
                program_id,
                accounts,
                name,
                description,
                discord_server,
                twitter,
                telegram,
                instagram,
                tiktok,
                website,
                treasury,
                profile,
                token_address,
                sol_price_usd,
            )
        }
        DaoInstruction::CreateProposal {
            name,
            description,
            dao_id,
            pod_id,
            start_time,
            end_time,
        } => {
            process_create_proposal(
                program_id,
                accounts,
                name,
                description,
                dao_id,
                pod_id,
                start_time,
                end_time,
            )
        }
        DaoInstruction::Vote { vote, proposal_id } => {
            process_vote(program_id, accounts, vote, proposal_id)
        }
        DaoInstruction::Featured { dao_id, sol_price_usd } => {
            process_featured(program_id, accounts, dao_id, sol_price_usd)
        }
        DaoInstruction::Modules { dao_id, module_type, sol_price_usd } => {
            process_modules(program_id, accounts, dao_id, module_type, sol_price_usd)
        }
    }
}

// Calculate fee in lamports based on SOL price
fn calculate_fee_in_lamports(sol_price_usd: u64) -> Result<u64, ProgramError> {
    // Validate price is within reasonable bounds (e.g., $1-$10,000)
    if sol_price_usd < 100 || sol_price_usd > 1_000_000 {
        return Err(DaoError::InvalidSolPrice.into());
    }
    
    // SOL has 9 decimal places (1 SOL = 1_000_000_000 lamports)
    // sol_price_usd is in cents, so we divide by 100 to get dollars
    // Calculate how many SOL equals $20 USD
    let sol_amount = (CREATE_DAO_FEE_USD as u128 * 100 * 1_000_000_000) / sol_price_usd as u128;
    
    // Check for overflow or other calculation errors
    if sol_amount > u64::MAX as u128 {
        return Err(DaoError::InvalidSolPrice.into());
    }
    
    Ok(sol_amount as u64)
}

// Process the create DAO instruction
pub fn process_create_dao(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    description: String,
    discord_server: String,
    twitter: String,
    telegram: String,
    instagram: String,
    tiktok: String,
    website: String,
    treasury: String,
    profile: String,
    token_address: String,
    sol_price_usd: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get the required accounts
    let creator_account = next_account_info(account_info_iter)?;
    let dao_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let fee_account = next_account_info(account_info_iter)?;
    
    // Verify the creator is signer
    if !creator_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify fee address is correct
    let expected_fee_pubkey = Pubkey::from_str(FEE_ADDRESS).unwrap();
    if *fee_account.key != expected_fee_pubkey {
        return Err(DaoError::InvalidFeeAccount.into());
    }
    
    // Calculate fee based on current SOL price
    let create_dao_fee = calculate_fee_in_lamports(sol_price_usd)?;
    
    msg!("DAO creation fee: {} lamports (${} at SOL price of ${}.{})", 
        create_dao_fee,
        CREATE_DAO_FEE_USD,
        sol_price_usd / 100,
        sol_price_usd % 100
    );
    
    // Create DAO data structure
    let dao = Dao {
        authority: *creator_account.key,
        name,
        description,
        discord_server,
        twitter,
        telegram,
        instagram,
        tiktok,
        website,
        treasury,
        profile,
        token_address,
    };
    
    // Calculate space required for the DAO account
    let dao_serialized = dao.try_to_vec()?;
    let space = dao_serialized.len() as u64;
    
    // Calculate the rent required
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(space as usize);
    
    // Check if creator has enough funds for rent + fee
    if creator_account.lamports() < rent_lamports + create_dao_fee {
        return Err(DaoError::InsufficientFunds.into());
    }
    
    // Create the DAO account
    invoke(
        &system_instruction::create_account(
            creator_account.key,
            dao_account.key,
            rent_lamports,
            space,
            program_id,
        ),
        &[creator_account.clone(), dao_account.clone(), system_program.clone()],
    )?;
    
    // Transfer fee to fee account
    invoke(
        &system_instruction::transfer(
            creator_account.key,
            fee_account.key,
            create_dao_fee,
        ),
        &[creator_account.clone(), fee_account.clone(), system_program.clone()],
    )?;
    
    // Serialize and store the DAO data
    dao.serialize(&mut &mut dao_account.data.borrow_mut()[..])?;
    
    msg!("DAO created successfully with ID: {}", dao_account.key);
    Ok(())
}

// Process the create proposal instruction
pub fn process_create_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    description: String,
    dao_id: String,
    pod_id: String,
    start_time: i64,
    end_time: i64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get the required accounts
    let creator_account = next_account_info(account_info_iter)?;
    let proposal_account = next_account_info(account_info_iter)?;
    let dao_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    
    // Get fee account if provided (fees for proposal creation are optional)
    let fee_account = if account_info_iter.len() > 0 {
        Some(next_account_info(account_info_iter)?)
    } else {
        None
    };
    
    // Verify the creator is signer
    if !creator_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify time constraints
    let clock = Clock::get()?;
    if start_time < clock.unix_timestamp || end_time <= start_time {
        return Err(DaoError::ProposalTimeInvalid.into());
    }
    
    // Create proposal data structure
    let proposal = Proposal {
        authority: *creator_account.key,
        name,
        description,
        dao_id,
        pod_id,
        start_time,
        end_time,
    };
    
    // Calculate space required for the proposal account
    let proposal_serialized = proposal.try_to_vec()?;
    let space = proposal_serialized.len() as u64;
    
    // Calculate the rent required
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(space as usize);
    
    // Create the proposal account
    invoke(
        &system_instruction::create_account(
            creator_account.key,
            proposal_account.key,
            rent_lamports,
            space,
            program_id,
        ),
        &[creator_account.clone(), proposal_account.clone(), system_program.clone()],
    )?;
    
    // If fee account is provided, send a small fee (optional)
    if let Some(fee_acc) = fee_account {
        // Verify fee address is correct
        let expected_fee_pubkey = Pubkey::from_str(FEE_ADDRESS).unwrap();
        if *fee_acc.key == expected_fee_pubkey {
            // Send a small fee (0.01 SOL)
            invoke(
                &system_instruction::transfer(
                    creator_account.key,
                    fee_acc.key,
                    10_000_000, // 0.01 SOL in lamports
                ),
                &[creator_account.clone(), fee_acc.clone(), system_program.clone()],
            )?;
        }
    }
    
    // Serialize and store the proposal data
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;
    
    msg!("Proposal created successfully with ID: {}", proposal_account.key);
    Ok(())
}

// Process the vote instruction
pub fn process_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vote: String,
    proposal_id: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get the required accounts
    let voter_account = next_account_info(account_info_iter)?;
    let vote_account = next_account_info(account_info_iter)?;
    let proposal_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    
    // Get fee account if provided (fees for voting are optional)
    let fee_account = if account_info_iter.len() > 0 {
        Some(next_account_info(account_info_iter)?)
    } else {
        None
    };
    
    // Verify the voter is signer
    if !voter_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate the vote
    if vote != "for" && vote != "against" {
        return Err(DaoError::InvalidVote.into());
    }
    
    // Create vote data structure
    let vote_data = Vote {
        voter: *voter_account.key,
        vote,
        proposal_id,
    };
    
    // Calculate space required for the vote account
    let vote_serialized = vote_data.try_to_vec()?;
    let space = vote_serialized.len() as u64;
    
    // Calculate the rent required
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(space as usize);
    
    // Create the vote account
    invoke(
        &system_instruction::create_account(
            voter_account.key,
            vote_account.key,
            rent_lamports,
            space,
            program_id,
        ),
        &[voter_account.clone(), vote_account.clone(), system_program.clone()],
    )?;
    
    // If fee account is provided, send a small fee (optional)
    if let Some(fee_acc) = fee_account {
        // Verify fee address is correct
        let expected_fee_pubkey = Pubkey::from_str(FEE_ADDRESS).unwrap();
        if *fee_acc.key == expected_fee_pubkey {
            // Send a small fee (0.005 SOL)
            invoke(
                &system_instruction::transfer(
                    voter_account.key,
                    fee_acc.key,
                    5_000_000, // 0.005 SOL in lamports
                ),
                &[voter_account.clone(), fee_acc.clone(), system_program.clone()],
            )?;
        }
    }
    
    // Serialize and store the vote data
    vote_data.serialize(&mut &mut vote_account.data.borrow_mut()[..])?;
    
    msg!("Vote recorded successfully");
    Ok(())
}

// Process the featured instruction
pub fn process_featured(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    dao_id: String,
    sol_price_usd: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get the required accounts
    let creator_account = next_account_info(account_info_iter)?;
    let featured_account = next_account_info(account_info_iter)?;
    let dao_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let fee_account = next_account_info(account_info_iter)?;
    
    // Verify the creator is signer
    if !creator_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify fee address is correct
    let expected_fee_pubkey = Pubkey::from_str(FEE_ADDRESS).unwrap();
    if *fee_account.key != expected_fee_pubkey {
        return Err(DaoError::InvalidFeeAccount.into());
    }
    
    // Calculate fee based on current SOL price
    let feature_fee = calculate_fee_in_lamports(sol_price_usd)?;
    
    msg!("Featured DAO fee: {} lamports (${} at SOL price of ${}.{})", 
        feature_fee,
        CREATE_DAO_FEE_USD,
        sol_price_usd / 100,
        sol_price_usd % 100
    );
    
    // Create featured data structure
    let featured_data = Featured {
        authority: *creator_account.key,
        dao_id,
    };
    
    // Calculate space required for the featured account
    let featured_serialized = featured_data.try_to_vec()?;
    let space = featured_serialized.len() as u64;
    
    // Calculate the rent required
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(space as usize);
    
    // Check if creator has enough funds for rent + fee
    if creator_account.lamports() < rent_lamports + feature_fee {
        return Err(DaoError::InsufficientFunds.into());
    }
    
    // Create the featured account
    invoke(
        &system_instruction::create_account(
            creator_account.key,
            featured_account.key,
            rent_lamports,
            space,
            program_id,
        ),
        &[creator_account.clone(), featured_account.clone(), system_program.clone()],
    )?;
    
    // Transfer fee to fee account
    invoke(
        &system_instruction::transfer(
            creator_account.key,
            fee_account.key,
            feature_fee,
        ),
        &[creator_account.clone(), fee_account.clone(), system_program.clone()],
    )?;
    
    // Serialize and store the featured data
    featured_data.serialize(&mut &mut featured_account.data.borrow_mut()[..])?;
    
    msg!("DAO featured successfully with ID: {}", featured_account.key);
    Ok(())
}

// Process the modules instruction
pub fn process_modules(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    dao_id: String,
    module_type: String,
    sol_price_usd: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get the required accounts
    let creator_account = next_account_info(account_info_iter)?;
    let module_account = next_account_info(account_info_iter)?;
    let dao_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let fee_account = next_account_info(account_info_iter)?;
    
    // Verify the creator is signer
    if !creator_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify fee address is correct
    let expected_fee_pubkey = Pubkey::from_str(FEE_ADDRESS).unwrap();
    if *fee_account.key != expected_fee_pubkey {
        return Err(DaoError::InvalidFeeAccount.into());
    }
    
    // Calculate fee based on current SOL price
    let module_fee = calculate_fee_in_lamports(sol_price_usd)?;
    
    msg!("Module activation fee: {} lamports (${} at SOL price of ${}.{})", 
        module_fee,
        CREATE_DAO_FEE_USD,
        sol_price_usd / 100,
        sol_price_usd % 100
    );
    
    // Create module data structure
    let module_data = Module {
        authority: *creator_account.key,
        dao_id,
        module_type,
    };
    
    // Calculate space required for the module account
    let module_serialized = module_data.try_to_vec()?;
    let space = module_serialized.len() as u64;
    
    // Calculate the rent required
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(space as usize);
    
    // Check if creator has enough funds for rent + fee
    if creator_account.lamports() < rent_lamports + module_fee {
        return Err(DaoError::InsufficientFunds.into());
    }
    
    // Create the module account
    invoke(
        &system_instruction::create_account(
            creator_account.key,
            module_account.key,
            rent_lamports,
            space,
            program_id,
        ),
        &[creator_account.clone(), module_account.clone(), system_program.clone()],
    )?;
    
    // Transfer fee to fee account
    invoke(
        &system_instruction::transfer(
            creator_account.key,
            fee_account.key,
            module_fee,
        ),
        &[creator_account.clone(), fee_account.clone(), system_program.clone()],
    )?;
    
    // Serialize and store the module data
    module_data.serialize(&mut &mut module_account.data.borrow_mut()[..])?;
    
    msg!("DAO module activated successfully with ID: {}", module_account.key);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_fee() {
        // SOL price = $100.00 (10000 cents)
        // $20 / $100 = 0.2 SOL = 200_000_000 lamports
        let fee = calculate_fee_in_lamports(10000).unwrap();
        assert_eq!(fee, 200_000_000);
        
        // SOL price = $50.00 (5000 cents)
        // $20 / $50 = 0.4 SOL = 400_000_000 lamports
        let fee = calculate_fee_in_lamports(5000).unwrap();
        assert_eq!(fee, 400_000_000);
        
        // SOL price = $200.00 (20000 cents)
        // $20 / $200 = 0.1 SOL = 100_000_000 lamports
        let fee = calculate_fee_in_lamports(20000).unwrap();
        assert_eq!(fee, 100_000_000);
    }
}
