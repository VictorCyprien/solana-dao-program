const fs = require('fs');
const path = require('path');
const { Keypair } = require('@solana/web3.js');
const { createDao, createProposal, vote, connection } = require('./client.js');

// Mock SOL price function since node-fetch is having issues
const mockGetSolPrice = async () => {
  // Let's assume SOL is $100 USD
  return 10000; // $100.00 in cents
};

// Helper function to load keypair from file
function loadKeypairFromFile(filePath) {
  const keypairData = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  return Keypair.fromSecretKey(Uint8Array.from(keypairData));
}

// Example usage for testing
async function main() {
  try {
    // Load the existing keypair (use the keypair that has SOL)
    const homedir = require('os').homedir();
    const keypairPath = path.join(homedir, '.config', 'solana', 'id.json');
    const payer = loadKeypairFromFile(keypairPath);
    console.log('Using wallet:', payer.publicKey.toString());
    
    // Check wallet balance
    const balance = await connection.getBalance(payer.publicKey);
    console.log('Wallet balance:', balance / 1_000_000_000, 'SOL');
    
    if (balance < 1_000_000_000) { // Less than 1 SOL
      console.error('Insufficient balance. Please fund your wallet with at least 1 SOL');
      return;
    }
    
    // 1. Use mock SOL price ($100 USD)
    const solPriceUsd = 10000; // $100.00 in cents
    console.log(`Using mock SOL price: $${solPriceUsd/100} (${solPriceUsd} cents)`);
    
    // Calculate expected fee
    const expectedFee = 20 / (solPriceUsd / 100); // $20 / SOL price
    console.log(`Expected fee: ~${expectedFee.toFixed(4)} SOL`);
    
    // 2. Create a DAO with the mock SOL price
    console.log('Creating DAO...');
    const timestamp = Date.now();
    const daoId = await createDao(
      payer,
      `Test DAO ${timestamp}`, // Add timestamp to make unique
      'A test DAO created via our SDK',
      'https://discord.gg/testdao',
      'https://twitter.com/testdao',
      'https://t.me/testdao',
      'https://instagram.com/testdao',
      'https://tiktok.com/@testdao',
      'https://testdao.org',
      'treasury_account_pubkey',
      'profile_url',
      'token_address',
      solPriceUsd // Pass the mock SOL price directly
    );
    
    console.log('DAO created successfully with ID:', daoId.toString());
    
    // Check wallet balance after DAO creation to verify fee
    const balanceAfterDao = await connection.getBalance(payer.publicKey);
    console.log('Wallet balance after DAO creation:', balanceAfterDao / 1_000_000_000, 'SOL');
    console.log('Fee paid:', (balance - balanceAfterDao) / 1_000_000_000, 'SOL');
    
    // 3. Create a proposal
    console.log('Creating proposal...');
    // Use current timestamp + 60 seconds for start time to ensure it's in the future
    const now = Math.floor(Date.now() / 1000);
    const startTime = now + 60; // Start 1 minute in the future
    const endTime = now + (7 * 24 * 60 * 60); // End one week later
    
    const proposalId = await createProposal(
      payer,
      'Test Proposal',
      'A test proposal for our DAO',
      daoId.toString(),
      '', // Empty pod_id
      startTime, // Start time (now + 60 seconds)
      endTime // End time (one week from now)
    );
    
    console.log('Proposal created successfully with ID:', proposalId.toString());
    
    // 4. Vote on the proposal
    console.log('Voting on proposal...');
    const voteId = await vote(
      payer,
      'for', // Can be "for" or "against"
      proposalId.toString()
    );
    
    console.log('Vote recorded successfully with ID:', voteId.toString());
    console.log('Test completed successfully!');
    
  } catch (error) {
    console.error('Error during test:', error);
  }
}

// Run the test
main(); 