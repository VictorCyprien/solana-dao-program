use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
    program::{invoke_signed},
    system_instruction,
    system_program,
    rent::Rent,
};

use crate::{
    error::DaoError,
    instruction::DaoInstruction,
    state::{Proposal, ProposalStatus},
};

/// Processor is the business logic handler of the DAO program
pub struct Processor;

impl Processor {
    /// Process a DAO program instruction
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        // Unpack the instruction data
        let instruction = DaoInstruction::unpack(instruction_data)?;

        // Match the instruction type and call the appropriate handler
        match instruction {
            DaoInstruction::CreateProposal {
                title,
                description,
                voting_period,
            } => {
                msg!("Instruction: Create Proposal");
                Self::process_create_proposal(program_id, accounts, title, description, voting_period)
            }
            DaoInstruction::Vote { approve } => {
                msg!("Instruction: Vote");
                Self::process_vote(program_id, accounts, approve)
            }
            DaoInstruction::FinalizeProposal {} => {
                msg!("Instruction: Finalize Proposal");
                Self::process_finalize_proposal(program_id, accounts)
            }
            DaoInstruction::WithdrawSol { amount } => {
                msg!("Instruction: Withdraw SOL");
                Self::process_withdraw_sol(program_id, accounts, amount)
            }
            DaoInstruction::InitializeTreasury {} => {
                msg!("Instruction: Initialize Treasury");
                Self::process_initialize_treasury(program_id, accounts)
            }
        }
    }

    /// Handles creating a new proposal
    fn process_create_proposal(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        title: String,
        description: String,
        voting_period: u64,
    ) -> ProgramResult {
        // Get account iterator
        let account_info_iter = &mut accounts.iter();
        
        // Get accounts
        let creator_info = next_account_info(account_info_iter)?;
        let proposal_info = next_account_info(account_info_iter)?;
        
        // Check that creator signed the transaction
        if !creator_info.is_signer {
            msg!("Creator must sign transaction");
            return Err(DaoError::ExpectedSigner.into());
        }

        // Get the current timestamp
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        
        // Create the proposal
        let proposal = Proposal {
            creator: *creator_info.key,
            title,
            description,
            created_at: current_timestamp,
            ends_at: current_timestamp + voting_period as i64,
            yes_votes: 0,
            no_votes: 0,
            status: ProposalStatus::Active,
            voters: Vec::new(),
        };

        // Serialize and store the proposal data
        proposal.serialize(&mut *proposal_info.data.borrow_mut())?;
        
        msg!("Proposal created successfully");
        Ok(())
    }

    /// Handles voting on a proposal
    fn process_vote(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        approve: bool,
    ) -> ProgramResult {
        // Get account iterator
        let account_info_iter = &mut accounts.iter();
        
        // Get accounts
        let voter_info = next_account_info(account_info_iter)?;
        let vote_account_info = next_account_info(account_info_iter)?;
        
        // Check that voter signed the transaction
        if !voter_info.is_signer {
            msg!("Voter must sign transaction");
            return Err(DaoError::ExpectedSigner.into());
        }
        
        // Check that vote account signed the transaction (because it's being created)
        if !vote_account_info.is_signer {
            msg!("Vote account must be a signer for account creation");
            return Err(DaoError::ExpectedSigner.into());
        }

        // Verify the vote account is owned by our program
        if vote_account_info.owner != program_id {
            msg!("Vote account must be owned by the DAO program");
            return Err(ProgramError::IncorrectProgramId);
        }
        
        // Get mutable reference to the vote account data
        let mut data = vote_account_info.data.borrow_mut();
        
        // Write voter pubkey (32 bytes)
        data[0..32].copy_from_slice(&voter_info.key.to_bytes());
        
        // Write vote value (1 byte)
        data[32] = if approve { 1 } else { 0 };
        
        if approve {
            msg!("Voted Yes");
        } else {
            msg!("Voted No");
        }
        
        msg!("Vote recorded successfully");
        Ok(())
    }

    /// Handles finalizing a proposal after voting period
    fn process_finalize_proposal(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        // Get account iterator
        let account_info_iter = &mut accounts.iter();
        
        // Get accounts
        let caller_info = next_account_info(account_info_iter)?;
        let proposal_info = next_account_info(account_info_iter)?;
        
        // Check that caller signed the transaction
        if !caller_info.is_signer {
            msg!("Caller must sign transaction");
            return Err(DaoError::ExpectedSigner.into());
        }

        // Deserialize the proposal
        let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())?;
        
        // Check if proposal is still active
        if proposal.status != ProposalStatus::Active {
            msg!("Proposal is already finalized");
            return Err(DaoError::ProposalFinalized.into());
        }
        
        // Check if voting period has ended
        let clock = Clock::get()?;
        if clock.unix_timestamp < proposal.ends_at {
            msg!("Voting period has not ended yet");
            return Err(DaoError::VotingStillActive.into());
        }
        
        // Determine if proposal passed or failed
        if proposal.yes_votes > proposal.no_votes {
            proposal.status = ProposalStatus::Passed;
            msg!("Proposal passed with {} Yes votes and {} No votes", proposal.yes_votes, proposal.no_votes);
        } else {
            proposal.status = ProposalStatus::Failed;
            msg!("Proposal failed with {} Yes votes and {} No votes", proposal.yes_votes, proposal.no_votes);
        }
        
        // Serialize and store the updated proposal data
        proposal.serialize(&mut *proposal_info.data.borrow_mut())?;
        
        msg!("Proposal finalized successfully");
        Ok(())
    }

    /// Handles initializing treasury account
    fn process_initialize_treasury(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        // Get account iterator
        let account_info_iter = &mut accounts.iter();
        
        // Get accounts
        let admin_info = next_account_info(account_info_iter)?;
        let treasury_pda_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        
        // Verify the accounts
        if !admin_info.is_signer {
            msg!("Admin must sign transaction");
            return Err(DaoError::ExpectedSigner.into());
        }

        // Check that system program is the correct one
        if system_program_info.key != &system_program::ID {
            return Err(ProgramError::InvalidAccountData);
        }
        
        // Dériver l'adresse PDA du trésor (treasury)
        let (treasury_pda, bump_seed) = Pubkey::find_program_address(&[b"treasury"], program_id);
        
        // Vérifier que le compte PDA fourni est bien notre PDA du trésor
        if treasury_pda != *treasury_pda_info.key {
            msg!("Invalid treasury PDA account");
            return Err(ProgramError::InvalidAccountData);
        }
        
        // Si le compte existe déjà, ne rien faire d'autre
        if treasury_pda_info.data_len() > 0 {
            msg!("Treasury already initialized");
            return Ok(());
        }
        
        msg!("Admin: {}", admin_info.key);
        msg!("Treasury PDA: {}", treasury_pda);
        msg!("Initializing treasury...");
        
        // Espace pour les données (peut être minimal)
        let space = 0;
        
        // Obtenir le montant minimum de lamports pour l'exemption de loyer
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(space);
        
        // Créer les seeds pour l'instruction signée
        let seeds = &[b"treasury".as_ref(), &[bump_seed]];
        
        // Créer le compte avec l'instruction système
        invoke_signed(
            &system_instruction::create_account(
                admin_info.key,
                &treasury_pda,
                lamports,
                space as u64,
                program_id,
            ),
            &[
                admin_info.clone(),
                treasury_pda_info.clone(),
                system_program_info.clone(),
            ],
            &[seeds],
        )?;
        
        msg!("Treasury initialized successfully");
        Ok(())
    }

    /// Handles withdrawing SOL from the program
    fn process_withdraw_sol(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        // Get account iterator
        let account_info_iter = &mut accounts.iter();
        
        // Get accounts
        let authority_info = next_account_info(account_info_iter)?;
        let recipient_info = next_account_info(account_info_iter)?;
        let treasury_pda_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        
        // Verify the accounts
        if !authority_info.is_signer {
            msg!("Authority must sign transaction");
            return Err(DaoError::ExpectedSigner.into());
        }

        // Check that system program is the correct one
        if system_program_info.key != &system_program::ID {
            return Err(ProgramError::InvalidAccountData);
        }
        
        // Dériver l'adresse PDA du trésor (treasury)
        let (treasury_pda, bump_seed) = Pubkey::find_program_address(&[b"treasury"], program_id);
        
        // Vérifier que le compte PDA fourni est bien notre PDA du trésor
        if treasury_pda != *treasury_pda_info.key {
            msg!("Invalid treasury PDA account");
            return Err(ProgramError::InvalidAccountData);
        }
        
        // Vérifiez si le compte est possédé par notre programme
        let is_owned_by_program = treasury_pda_info.owner == program_id;
        
        // Si le compte n'est pas possédé par notre programme, c'est probablement un compte système standard
        if !is_owned_by_program {
            msg!("Treasury PDA is not owned by the program, treating it as a system account");
            
            // Pour un compte système standard, nous ne pouvons pas utiliser invoke_signed, on doit initialiser le compte
            // Initialiser le compte si nécessaire avant de continuer
            if treasury_pda_info.data_len() == 0 {
                msg!("Initializing treasury account first...");
                
                // Espace pour les données (peut être minimal)
                let space = 0;
                
                // Obtenir le montant minimum de lamports pour l'exemption de loyer
                let rent = Rent::get()?;
                let min_lamports = rent.minimum_balance(space);
                
                // Créer les seeds pour l'instruction signée
                let seeds = &[b"treasury".as_ref(), &[bump_seed]];
                
                // Créer le compte avec l'instruction système
                invoke_signed(
                    &system_instruction::create_account(
                        authority_info.key,
                        &treasury_pda,
                        min_lamports,
                        space as u64,
                        program_id,
                    ),
                    &[
                        authority_info.clone(),
                        treasury_pda_info.clone(),
                        system_program_info.clone(),
                    ],
                    &[seeds],
                )?;
                
                msg!("Treasury account initialized");
            }
            
            // Si le compte n'est toujours pas possédé par le programme après l'initialisation, erreur
            if treasury_pda_info.owner != program_id {
                // Nous pouvons toujours aider l'utilisateur en expliquant comment transférer les fonds
                msg!("Treasury account is not owned by the program. Please initialize the treasury with:");
                msg!("node dao-treasury-cli.js init");
                msg!("Then transfer funds to the treasury with:");
                msg!("node dao-treasury-cli.js deposit <amount>");
                msg!("Finally withdraw with:");
                msg!("node dao-treasury-cli.js withdraw <amount>");
                return Err(ProgramError::IllegalOwner);
            }
        }
        
        // For security, you should check if this is an authorized admin
        // This could be implemented by checking a list of admin pubkeys
        // stored in a PDA or another mechanism
        
        // TODO: Add admin authorization check, for now we're using a simple ownership check
        
        msg!("Authority: {}", authority_info.key);
        msg!("Recipient: {}", recipient_info.key);
        msg!("Treasury PDA: {}", treasury_pda_info.key);
        msg!("Amount to withdraw: {}", amount);
        
        // Check if there are enough lamports in the treasury account
        let treasury_lamports = treasury_pda_info.lamports();
        if treasury_lamports < amount {
            msg!("Insufficient funds: treasury has {} lamports, trying to withdraw {}", 
                treasury_lamports, amount);
            return Err(ProgramError::InsufficientFunds);
        }
        
        // Méthode plus simple et plus fiable de transfert direct de lamports
        // Calculer le montant minimum nécessaire pour que le compte reste exempt de loyer
        let rent = Rent::get()?;
        let min_rent_exempt = rent.minimum_balance(treasury_pda_info.data_len());
        
        // S'assurer qu'après le retrait, il reste suffisamment de lamports pour l'exemption de loyer
        if treasury_lamports - amount < min_rent_exempt {
            msg!("Cannot withdraw: would leave treasury with insufficient rent-exempt balance");
            return Err(ProgramError::InsufficientFunds);
        }
        
        // Créer la fonction de transfert
        let transfer_instruction = system_instruction::transfer(
            &treasury_pda,
            recipient_info.key,
            amount
        );
        
        // Seeds pour signer au nom de la PDA
        let seeds = &[b"treasury".as_ref(), &[bump_seed]];
        
        // Exécuter l'instruction de transfert avec la signature PDA
        invoke_signed(
            &transfer_instruction,
            &[
                treasury_pda_info.clone(),
                recipient_info.clone(),
                system_program_info.clone(),
            ],
            &[seeds],
        )?;
        
        msg!("Withdrawal of {} lamports successful", amount);
        Ok(())
    }
} 