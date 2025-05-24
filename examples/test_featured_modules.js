const fs = require('fs');
const path = require('path');
const { Keypair } = require('@solana/web3.js');
const { createDao, activateModule, createFeatured, connection } = require('./client.js');

// Helper function to load keypair from file
function loadKeypairFromFile(filePath) {
  const keypairData = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  return Keypair.fromSecretKey(Uint8Array.from(keypairData));
}

// Test the featured and modules functionality
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
    
    if (balance < 3_000_000_000) { // Less than 3 SOL
      console.error('Insufficient balance. Please fund your wallet with at least 3 SOL for this test');
      return;
    }
    
    // 1. Use mock SOL price ($100 USD)
    const solPriceUsd = 10000; // $100.00 in cents
    console.log(`Using mock SOL price: $${solPriceUsd/100} (${solPriceUsd} cents)`);
    
    // Calculate expected fee
    const expectedFee = 20 / (solPriceUsd / 100); // $20 / SOL price
    console.log(`Expected fee: ~${expectedFee.toFixed(4)} SOL per operation`);
    
    // 2. Create a DAO with the mock SOL price
    console.log('------------------------------------');
    console.log('Step 1: Creating DAO...');
    const timestamp = Date.now();
    const daoId = await createDao(
      payer,
      `Featured Test DAO ${timestamp}`, // Add timestamp to make unique
      'A test DAO for featured and modules testing',
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
    
    // Check wallet balance after DAO creation
    const balanceAfterDao = await connection.getBalance(payer.publicKey);
    console.log('Wallet balance after DAO creation:', balanceAfterDao / 1_000_000_000, 'SOL');
    console.log('Fee paid:', (balance - balanceAfterDao) / 1_000_000_000, 'SOL');
    
    // 3. Enable the POD module for the DAO
    console.log('------------------------------------');
    console.log('Step 2: Activating POD module...');
    const moduleId = await activateModule(
      payer,
      daoId.toString(),
      "POD", // Activate POD module
      solPriceUsd // Pass the mock SOL price directly
    );
    
    console.log('Module activated successfully with ID:', moduleId.toString());
    
    // Check wallet balance after module activation
    const balanceAfterModule = await connection.getBalance(payer.publicKey);
    console.log('Wallet balance after module activation:', balanceAfterModule / 1_000_000_000, 'SOL');
    console.log('Fee paid:', (balanceAfterDao - balanceAfterModule) / 1_000_000_000, 'SOL');
    
    // 4. Enable featured status for the DAO
    console.log('------------------------------------');
    console.log('Step 3: Enabling featured status...');
    const featuredId = await createFeatured(
      payer,
      daoId.toString(),
      solPriceUsd // Pass the mock SOL price directly
    );
    
    console.log('Featured status enabled successfully with ID:', featuredId.toString());
    
    // Check wallet balance after enabling featured status
    const balanceAfterFeatured = await connection.getBalance(payer.publicKey);
    console.log('Wallet balance after enabling featured status:', balanceAfterFeatured / 1_000_000_000, 'SOL');
    console.log('Fee paid:', (balanceAfterModule - balanceAfterFeatured) / 1_000_000_000, 'SOL');
    
    console.log('------------------------------------');
    console.log('Test completed successfully!');
    console.log('Summary:');
    console.log('- DAO ID:', daoId.toString());
    console.log('- Module ID:', moduleId.toString());
    console.log('- Featured ID:', featuredId.toString());
    console.log('- Total SOL spent:', (balance - balanceAfterFeatured) / 1_000_000_000, 'SOL');
    
  } catch (error) {
    console.error('Error during test:', error);
  }
}

// Run the test
main();
