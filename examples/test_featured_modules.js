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
    
    // Calculate expected fee for standard operations
    const expectedFee = 20 / (solPriceUsd / 100); // $20 / SOL price
    console.log(`Expected fee: ~${expectedFee.toFixed(4)} SOL per operation`);
    
    // Calculate expected fees for featured listings
    const oneDayFee = (20 * 1) / (solPriceUsd / 100); // $20 * 1 day / SOL price
    const threeDayFee = (20 * 3) / (solPriceUsd / 100); // $20 * 3 days / SOL price
    console.log(`Expected featured fee for 1 day: ~${oneDayFee.toFixed(4)} SOL`);
    console.log(`Expected featured fee for 3 days: ~${threeDayFee.toFixed(4)} SOL`);
    
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
    
    // 4. Enable featured status for the DAO (for 1 day)
    console.log('------------------------------------');
    console.log('Step 3a: Enabling featured status for 1 day...');
    const featuredId1Day = await createFeatured(
      payer,
      daoId.toString(),
      1, // Feature for 1 day
      solPriceUsd // Pass the mock SOL price directly
    );
    
    console.log('Featured status (1 day) enabled successfully with ID:', featuredId1Day.toString());
    
    // Check wallet balance after enabling 1-day featured status
    const balanceAfter1DayFeatured = await connection.getBalance(payer.publicKey);
    console.log('Wallet balance after 1-day featured:', balanceAfter1DayFeatured / 1_000_000_000, 'SOL');
    console.log('1-day featured fee paid:', (balanceAfterModule - balanceAfter1DayFeatured) / 1_000_000_000, 'SOL');
    
    // 5. Enable featured status for the DAO (for 3 days)
    console.log('------------------------------------');
    console.log('Step 3b: Enabling featured status for 3 days...');
    const featuredId3Days = await createFeatured(
      payer,
      daoId.toString(),
      3, // Feature for 3 days
      solPriceUsd // Pass the mock SOL price directly
    );
    
    console.log('Featured status (3 days) enabled successfully with ID:', featuredId3Days.toString());
    
    // Check wallet balance after enabling 3-day featured status
    const balanceAfter3DayFeatured = await connection.getBalance(payer.publicKey);
    console.log('Wallet balance after 3-day featured:', balanceAfter3DayFeatured / 1_000_000_000, 'SOL');
    console.log('3-day featured fee paid:', (balanceAfter1DayFeatured - balanceAfter3DayFeatured) / 1_000_000_000, 'SOL');
    
    console.log('------------------------------------');
    console.log('Test completed successfully!');
    console.log('Summary:');
    console.log('- DAO ID:', daoId.toString());
    console.log('- Module ID:', moduleId.toString());
    console.log('- Featured ID (1 day):', featuredId1Day.toString());
    console.log('- Featured ID (3 days):', featuredId3Days.toString());
    console.log('- Total SOL spent:', (balance - balanceAfter3DayFeatured) / 1_000_000_000, 'SOL');
    
    // Verify fee calculations
    const actual1DayFee = (balanceAfterModule - balanceAfter1DayFeatured) / 1_000_000_000;
    const actual3DayFee = (balanceAfter1DayFeatured - balanceAfter3DayFeatured) / 1_000_000_000;
    
    console.log('------------------------------------');
    console.log('FEE VERIFICATION:');
    console.log(`1-day featured - Expected: ${oneDayFee.toFixed(4)} SOL, Actual: ${actual1DayFee.toFixed(4)} SOL`);
    console.log(`3-day featured - Expected: ${threeDayFee.toFixed(4)} SOL, Actual: ${actual3DayFee.toFixed(4)} SOL`);
    
    // Check if fees are approximately correct (within 0.01 SOL tolerance for rent)
    const tolerance = 0.01;
    const fee1DayMatch = Math.abs(actual1DayFee - oneDayFee) < tolerance;
    const fee3DayMatch = Math.abs(actual3DayFee - threeDayFee) < tolerance;
    
    console.log(`1-day fee match: ${fee1DayMatch ? '✅ PASS' : '❌ FAIL'}`);
    console.log(`3-day fee match: ${fee3DayMatch ? '✅ PASS' : '❌ FAIL'}`);
    
    if (fee1DayMatch && fee3DayMatch) {
      console.log('🎉 Featured days tests PASSED!');
    } else {
      console.log('⚠️  Some featured tests failed - check fee calculations');
    }
    
  } catch (error) {
    console.error('Error during test:', error);
  }
}

// Run the test
main();
