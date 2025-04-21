const {
  Connection,
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} = require('@solana/web3.js');
const {
  struct,
  u8,
  seq,
  blob
} = require('@solana/buffer-layout');
const { Buffer } = require('buffer');
const BN = require('bn.js');
const fetch = require('node-fetch');

// Connection to the Solana cluster
const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

// Program ID (deployed program ID)
const programId = new PublicKey('7Mof2gtQh9478eWLAFTbzk9hpwsSjsaEsGiBGD42dMQj');

// Fee recipient address
const FEE_ADDRESS = new PublicKey('BAGek78CDYQ8phuDqNk7sQzD7LdJeKkb7jD4y2AyR3tJ');

// Function to fetch current SOL price
async function getSolPrice() {
  try {
    // Using CoinGecko API to get SOL price
    const response = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd');
    const data = await response.json();
    const solPriceUsd = data.solana.usd;
    
    console.log(`Current SOL price: $${solPriceUsd}`);
    
    // Convert to cents and return as integer (e.g., $100.50 => 10050)
    return Math.round(solPriceUsd * 100);
  } catch (error) {
    console.error('Error fetching SOL price:', error);
    throw new Error('Failed to fetch SOL price. Please try again.');
  }
}

// Utility function to create a new account
async function createAccount(connection, payer, space) {
  const newAccount = Keypair.generate();
  const transaction = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: newAccount.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(space),
      space,
      programId,
    })
  );

  await sendAndConfirmTransaction(connection, transaction, [payer, newAccount]);
  return newAccount;
}

// Simpler approach - manual serialization for instruction data
function serializeString(str) {
  const buf = Buffer.alloc(4 + str.length);
  buf.writeUInt32LE(str.length, 0);
  buf.write(str, 4);
  return buf;
}

function serializeCreateDaoInstruction(
  name, 
  description, 
  discordServer, 
  twitter, 
  telegram, 
  instagram, 
  tiktok, 
  website, 
  treasury, 
  profile, 
  solPriceUsd
) {
  // Instruction index (0 for CreateDao)
  const instructionBuf = Buffer.alloc(1);
  instructionBuf.writeUInt8(0, 0);
  
  // Serialize all strings
  const nameBuf = serializeString(name);
  const descriptionBuf = serializeString(description);
  const discordServerBuf = serializeString(discordServer);
  const twitterBuf = serializeString(twitter);
  const telegramBuf = serializeString(telegram);
  const instagramBuf = serializeString(instagram);
  const tiktokBuf = serializeString(tiktok);
  const websiteBuf = serializeString(website);
  const treasuryBuf = serializeString(treasury);
  const profileBuf = serializeString(profile);
  
  // Serialize u64 sol price (8 bytes, little-endian)
  const solPriceBuf = Buffer.alloc(8);
  
  // Convert to u64 (BN)
  const solPriceBN = new BN(solPriceUsd.toString());
  solPriceBN.toArray('le', 8).forEach((byte, index) => {
    solPriceBuf[index] = byte;
  });
  
  // Concat all buffers
  return Buffer.concat([
    instructionBuf,
    nameBuf,
    descriptionBuf,
    discordServerBuf,
    twitterBuf,
    telegramBuf,
    instagramBuf,
    tiktokBuf,
    websiteBuf,
    treasuryBuf,
    profileBuf,
    solPriceBuf
  ]);
}

function serializeCreateProposalInstruction(
  name,
  description,
  daoId,
  podId,
  startTime,
  endTime
) {
  // Instruction index (1 for CreateProposal)
  const instructionBuf = Buffer.alloc(1);
  instructionBuf.writeUInt8(1, 0);
  
  // Serialize all strings
  const nameBuf = serializeString(name);
  const descriptionBuf = serializeString(description);
  const daoIdBuf = serializeString(daoId);
  const podIdBuf = serializeString(podId);
  
  // Serialize i64 timestamps (8 bytes each, little-endian)
  const startTimeBuf = Buffer.alloc(8);
  const endTimeBuf = Buffer.alloc(8);
  
  const startTimeBN = new BN(startTime.toString());
  const endTimeBN = new BN(endTime.toString());
  
  startTimeBN.toArray('le', 8).forEach((byte, index) => {
    startTimeBuf[index] = byte;
  });
  
  endTimeBN.toArray('le', 8).forEach((byte, index) => {
    endTimeBuf[index] = byte;
  });
  
  // Concat all buffers
  return Buffer.concat([
    instructionBuf,
    nameBuf,
    descriptionBuf,
    daoIdBuf,
    podIdBuf,
    startTimeBuf,
    endTimeBuf
  ]);
}

function serializeVoteInstruction(
  voteValue,
  proposalId
) {
  // Instruction index (2 for Vote)
  const instructionBuf = Buffer.alloc(1);
  instructionBuf.writeUInt8(2, 0);
  
  // Serialize strings
  const voteBuf = serializeString(voteValue);
  const proposalIdBuf = serializeString(proposalId);
  
  // Concat all buffers
  return Buffer.concat([
    instructionBuf,
    voteBuf,
    proposalIdBuf
  ]);
}

// Create a new DAO
async function createDao(
  payer,
  name,
  description,
  discordServer,
  twitter,
  telegram,
  instagram,
  tiktok,
  website,
  treasury,
  profile,
  sol_price_usd = null // Optional parameter for direct price input
) {
  // Get current SOL price if not provided
  if (sol_price_usd === null) {
    sol_price_usd = await getSolPrice();
  }
  console.log(`Creating DAO with SOL price: $${sol_price_usd/100} (${sol_price_usd} cents)`);
  
  // Calculate expected fee based on SOL price
  const expectedFeeInSol = 20 / (sol_price_usd / 100);
  const expectedFeeInLamports = Math.round(expectedFeeInSol * 1_000_000_000);
  console.log(`Expected fee: ${expectedFeeInSol} SOL (${expectedFeeInLamports} lamports)`);
  
  // Generate a new keypair for the DAO
  const daoAccount = Keypair.generate();
  console.log('Generated DAO account:', daoAccount.publicKey.toString());
  
  // Serialize instruction data
  const data = serializeCreateDaoInstruction(
    name,
    description,
    discordServer,
    twitter,
    telegram,
    instagram,
    tiktok,
    website,
    treasury,
    profile,
    sol_price_usd
  );
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: daoAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: FEE_ADDRESS, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });
  
  // Create transaction
  const transaction = new Transaction().add(instruction);
  
  console.log('Sending transaction...');
  // Sign transaction with both payer and the new DAO account keypair
  const signature = await sendAndConfirmTransaction(
    connection, 
    transaction, 
    [payer, daoAccount],
    {
      skipPreflight: true, // Skip the preflight check to debug issues
    }
  );
  
  console.log('Transaction signature:', signature);
  console.log('DAO created successfully!');
  console.log('DAO ID:', daoAccount.publicKey.toString());
  
  return daoAccount.publicKey;
}

// Create a new proposal
async function createProposal(
  payer,
  name,
  description,
  daoId,
  podId,
  startTime,
  endTime
) {
  // Generate a new keypair for the proposal
  const proposalAccount = Keypair.generate();
  console.log('Generated proposal account:', proposalAccount.publicKey.toString());
  
  // Serialize instruction data
  const data = serializeCreateProposalInstruction(
    name,
    description,
    daoId,
    podId,
    startTime,
    endTime
  );
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: proposalAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(daoId), isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: FEE_ADDRESS, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });
  
  // Create transaction
  const transaction = new Transaction().add(instruction);
  
  console.log('Sending transaction...');
  // Sign transaction with both payer and the new proposal account keypair
  const signature = await sendAndConfirmTransaction(
    connection, 
    transaction, 
    [payer, proposalAccount],
    {
      skipPreflight: true, // Skip the preflight check to debug issues
    }
  );
  
  console.log('Transaction signature:', signature);
  console.log('Proposal created successfully!');
  console.log('Proposal ID:', proposalAccount.publicKey.toString());
  
  return proposalAccount.publicKey;
}

// Vote on a proposal
async function vote(
  payer,
  voteValue,
  proposalId
) {
  // Generate a new keypair for the vote
  const voteAccount = Keypair.generate();
  console.log('Generated vote account:', voteAccount.publicKey.toString());
  
  // Serialize instruction data
  const data = serializeVoteInstruction(
    voteValue,
    proposalId
  );
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: voteAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(proposalId), isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: FEE_ADDRESS, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });
  
  // Create transaction
  const transaction = new Transaction().add(instruction);
  
  console.log('Sending transaction...');
  // Sign transaction with both payer and the new vote account keypair
  const signature = await sendAndConfirmTransaction(
    connection, 
    transaction, 
    [payer, voteAccount],
    {
      skipPreflight: true, // Skip the preflight check to debug issues
    }
  );
  
  console.log('Transaction signature:', signature);
  console.log('Vote recorded successfully!');
  console.log('Vote ID:', voteAccount.publicKey.toString());
  
  return voteAccount.publicKey;
}

// Example usage
async function main() {
  try {
    // Load your wallet (This is just for testing. In production, use proper wallet management)
    const secretKey = Uint8Array.from([
      // Your test wallet secret key
    ]);
    const payer = Keypair.fromSecretKey(secretKey);
    
    // Check wallet balance
    const balance = await connection.getBalance(payer.publicKey);
    console.log('Wallet balance:', balance / 1_000_000_000, 'SOL');
    
    // 1. Create a DAO
    const daoId = await createDao(
      payer,
      'My DAO',
      'A description of my DAO',
      'https://discord.gg/mydao',
      'https://twitter.com/mydao',
      'https://t.me/mydao',
      'https://instagram.com/mydao',
      'https://tiktok.com/@mydao',
      'https://mydao.org',
      'treasury_account_pubkey', // This should be a valid PublicKey string
      'profile_url'
    );
    
    // 2. Create a proposal
    const now = Math.floor(Date.now() / 1000);
    const oneWeekFromNow = now + (7 * 24 * 60 * 60);
    
    const proposalId = await createProposal(
      payer,
      'My Proposal',
      'A description of my proposal',
      daoId.toString(),
      '', // Empty pod_id
      now, // Start time (now)
      oneWeekFromNow // End time (one week from now)
    );
    
    // 3. Vote on the proposal
    await vote(
      payer,
      'yes', // Can be 'yes', 'no', or 'abstain'
      proposalId.toString()
    );
    
  } catch (error) {
    console.error('Error:', error);
  }
}

// Run the example
// Uncomment to run the example
// main();

module.exports = {
  createDao,
  createProposal,
  vote,
  getSolPrice,
  connection,
}; 