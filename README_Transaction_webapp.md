# Smart contract integration

This is an example to integrate smart contract interaction with ElizaOS

```typescript
// solana-dao.ts
import { 
  Connection, 
  Keypair, 
  PublicKey, 
  SystemProgram, 
  Transaction, 
  TransactionInstruction 
} from '@solana/web3.js';
import BN from 'bn.js';

// Program ID (replace with your deployed program ID)
const PROGRAM_ID = new PublicKey('7Mof2gtQh9478eWLAFTbzk9hpwsSjsaEsGiBGD42dMQj');

// Fee recipient address
const FEE_ADDRESS = new PublicKey('BAGek78CDYQ8phuDqNk7sQzD7LdJeKkb7jD4y2AyR3tJ');

// Helper function for string serialization
function serializeString(str: string): Buffer {
  const buf = Buffer.alloc(4 + str.length);
  buf.writeUInt32LE(str.length, 0);
  buf.write(str, 4);
  return buf;
}

// Serialize DAO creation instruction data
export function serializeCreateDaoInstruction(
  name: string,
  description: string,
  discordServer: string,
  twitter: string,
  telegram: string,
  instagram: string,
  tiktok: string,
  website: string,
  treasury: string,
  profile: string,
  tokenAddress: string,
  solPriceUsd: number
): Buffer {
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
  const tokenAddressBuf = serializeString(tokenAddress);
  
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
    tokenAddressBuf,
    solPriceBuf
  ]);
}

// Serialize proposal creation instruction data
export function serializeCreateProposalInstruction(
  name: string,
  description: string,
  daoId: string,
  podId: string,
  startTime: number,
  endTime: number
): Buffer {
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

// Serialize vote instruction data
export function serializeVoteInstruction(
  voteValue: string,
  proposalId: string
): Buffer {
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

// Serialize featured instruction data
export function serializeFeaturedInstruction(
  daoId: string,
  solPriceUsd: number
): Buffer {
  // Instruction index (3 for Featured)
  const instructionBuf = Buffer.alloc(1);
  instructionBuf.writeUInt8(3, 0);
  
  // Serialize string
  const daoIdBuf = serializeString(daoId);
  
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
    daoIdBuf,
    solPriceBuf
  ]);
}

// Serialize modules instruction data
export function serializeModulesInstruction(
  daoId: string,
  moduleType: string,
  solPriceUsd: number
): Buffer {
  // Instruction index (4 for Modules)
  const instructionBuf = Buffer.alloc(1);
  instructionBuf.writeUInt8(4, 0);
  
  // Serialize strings
  const daoIdBuf = serializeString(daoId);
  const moduleTypeBuf = serializeString(moduleType);
  
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
    daoIdBuf,
    moduleTypeBuf,
    solPriceBuf
  ]);
}

// Fetch current SOL price from an API
export async function getSolPrice(): Promise<number> {
  try {
    const response = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd');
    const data = await response.json();
    const solPriceUsd = data.solana.usd;
    
    // Convert to cents and return as integer (e.g., $100.50 => 10050)
    return Math.round(solPriceUsd * 100);
  } catch (error) {
    console.error('Error fetching SOL price:', error);
    throw new Error('Failed to fetch SOL price. Please try again.');
  }
}

// Create a DAO transaction
export async function createDaoTransaction(
  connection: Connection,
  wallet: { publicKey: PublicKey },
  name: string,
  description: string,
  discordServer: string,
  twitter: string,
  telegram: string,
  instagram: string,
  tiktok: string,
  website: string,
  treasury: string,
  profile: string,
  tokenAddress: string,
  solPriceUsd?: number // Optional - will fetch current price if not provided
): Promise<{ transaction: Transaction, daoAccount: Keypair }> {
  if (!wallet.publicKey) throw new Error("Wallet not connected");
  
  // Get current SOL price if not provided
  if (!solPriceUsd) {
    solPriceUsd = await getSolPrice();
  }
  
  // Generate a new keypair for the DAO
  const daoAccount = Keypair.generate();
  
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
    tokenAddress,
    solPriceUsd
  );
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
      { pubkey: daoAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: FEE_ADDRESS, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data,
  });
  
  // Create transaction
  const transaction = new Transaction().add(instruction);
  
  // Set recent blockhash
  transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  transaction.feePayer = wallet.publicKey;
  
  // Partially sign with the DAO account
  transaction.partialSign(daoAccount);
  
  return { transaction, daoAccount };
}

// Create a new proposal transaction
export async function createProposalTransaction(
  connection: Connection,
  wallet: { publicKey: PublicKey },
  name: string,
  description: string,
  daoId: string,
  podId: string,
  startTime: number,
  endTime: number
): Promise<{ transaction: Transaction, proposalAccount: Keypair }> {
  if (!wallet.publicKey) throw new Error("Wallet not connected");
  
  // Generate a new keypair for the proposal
  const proposalAccount = Keypair.generate();
  
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
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
      { pubkey: proposalAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(daoId), isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: FEE_ADDRESS, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data,
  });
  
  // Create transaction
  const transaction = new Transaction().add(instruction);
  
  // Set recent blockhash
  transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  transaction.feePayer = wallet.publicKey;
  
  // Partially sign with the proposal account
  transaction.partialSign(proposalAccount);
  
  return { transaction, proposalAccount };
}

// Create a vote transaction
export async function createVoteTransaction(
  connection: Connection,
  wallet: { publicKey: PublicKey },
  voteValue: string, // "for" or "against"
  proposalId: string
): Promise<{ transaction: Transaction, voteAccount: Keypair }> {
  if (!wallet.publicKey) throw new Error("Wallet not connected");
  
  // Generate a new keypair for the vote
  const voteAccount = Keypair.generate();
  
  // Serialize instruction data
  const data = serializeVoteInstruction(
    voteValue,
    proposalId
  );
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
      { pubkey: voteAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(proposalId), isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: FEE_ADDRESS, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data,
  });
  
  // Create transaction
  const transaction = new Transaction().add(instruction);
  
  // Set recent blockhash
  transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  transaction.feePayer = wallet.publicKey;
  
  // Partially sign with the vote account
  transaction.partialSign(voteAccount);
  
  return { transaction, voteAccount };
}

// Create a featured transaction
export async function createFeaturedTransaction(
  connection: Connection,
  wallet: { publicKey: PublicKey },
  daoId: string,
  solPriceUsd?: number // Optional - will fetch current price if not provided
): Promise<{ transaction: Transaction, featuredAccount: Keypair }> {
  if (!wallet.publicKey) throw new Error("Wallet not connected");
  
  // Get current SOL price if not provided
  if (!solPriceUsd) {
    solPriceUsd = await getSolPrice();
  }
  
  // Generate a new keypair for the featured entry
  const featuredAccount = Keypair.generate();
  
  // Serialize instruction data
  const data = serializeFeaturedInstruction(
    daoId,
    solPriceUsd
  );
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
      { pubkey: featuredAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(daoId), isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: FEE_ADDRESS, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data,
  });
  
  // Create transaction
  const transaction = new Transaction().add(instruction);
  
  // Set recent blockhash
  transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  transaction.feePayer = wallet.publicKey;
  
  // Partially sign with the featured account
  transaction.partialSign(featuredAccount);
  
  return { transaction, featuredAccount };
}

// Create a module activation transaction
export async function createModuleTransaction(
  connection: Connection,
  wallet: { publicKey: PublicKey },
  daoId: string,
  moduleType: string, // "POD" or "POL"
  solPriceUsd?: number // Optional - will fetch current price if not provided
): Promise<{ transaction: Transaction, moduleAccount: Keypair }> {
  if (!wallet.publicKey) throw new Error("Wallet not connected");
  
  // Validate module type
  if (moduleType !== "POD" && moduleType !== "POL") {
    throw new Error('Invalid module type. Must be either "POD" or "POL".');
  }
  
  // Get current SOL price if not provided
  if (!solPriceUsd) {
    solPriceUsd = await getSolPrice();
  }
  
  // Generate a new keypair for the module
  const moduleAccount = Keypair.generate();
  
  // Serialize instruction data
  const data = serializeModulesInstruction(
    daoId,
    moduleType,
    solPriceUsd
  );
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
      { pubkey: moduleAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(daoId), isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: FEE_ADDRESS, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data,
  });
  
  // Create transaction
  const transaction = new Transaction().add(instruction);
  
  // Set recent blockhash
  transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  transaction.feePayer = wallet.publicKey;
  
  // Partially sign with the module account
  transaction.partialSign(moduleAccount);
  
  return { transaction, moduleAccount };
}
```

## Usage with Wallet Adapter

Here's how to use these transaction functions with `@solana/wallet-adapter-react`:

```typescript
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { createDaoTransaction } from './solana-dao';

// Inside your React component:
const { connection } = useConnection();
const wallet = useWallet();

const handleCreateDao = async () => {
  try {
    // Create the transaction
    const { transaction, daoAccount } = await createDaoTransaction(
      connection,
      wallet,
      "My DAO",
      "Description of my DAO",
      "https://discord.gg/mydao",
      "https://twitter.com/mydao",
      "https://t.me/mydao",
      "https://instagram.com/mydao",
      "https://tiktok.com/@mydao",
      "https://mydao.org",
      "treasury_account_pubkey",
      "profile_url",
      "token_address_pubkey"
    );
    
    // Send the transaction to the wallet for signing
    const signature = await wallet.sendTransaction(transaction, connection);
    
    // Confirm the transaction
    await connection.confirmTransaction(signature, 'confirmed');
    
    console.log('DAO created successfully!');
    console.log('DAO ID:', daoAccount.publicKey.toString());
    
    return daoAccount.publicKey.toString();
  } catch (error) {
    console.error('Error creating DAO:', error);
    throw error;
  }
};
```
The same pattern applies for creating proposals and voting.

After creating the transaction, we can retrive the transaction and the voteAccount

```typescript
const { transaction, daoAccount } = result;
```

And pass it to the API to create the DAO :

```typescript
const connection = new Connection(SOLANA_RPC_ENDPOINT);
const txTransaction = await walletState.sendTransaction(transaction, connection);

// Wait for confirmation
await connection.confirmTransaction(signature, 'confirmed');

const pubkey = daoAccount.publicKey.toString(),
```

Then we pass pubkey and txTransaction into the form :

```json
{
  name: string
  description: string
  discord_server?: string
  twitter?: string
  telegram?: string
  instagram?: string
  tiktok?: string
  website?: string
  treasury?: string
  pubkey: string
  transaction: string (txTransaction)
  profile?: string
  token_address: string
}
```

## Featured and Modules Integration

Here's how to use the new featured and module activation functions with `@solana/wallet-adapter-react`:

### Featured Integration

```typescript
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { createFeaturedTransaction } from './solana-dao';

// Inside your React component:
const { connection } = useConnection();
const wallet = useWallet();

const handleCreateFeatured = async (daoId: string) => {
  try {
    // Create the transaction
    const { transaction, featuredAccount } = await createFeaturedTransaction(
      connection,
      wallet,
      daoId
    );
    
    // Send the transaction to the wallet for signing
    const signature = await wallet.sendTransaction(transaction, connection);
    
    // Confirm the transaction
    await connection.confirmTransaction(signature, 'confirmed');
    
    console.log('DAO featured successfully!');
    console.log('Featured ID:', featuredAccount.publicKey.toString());
    
    // Now you can update your UI to show the DAO as featured
    // and store the featuredAccount.publicKey for reference
    
    return featuredAccount.publicKey.toString();
  } catch (error) {
    console.error('Error featuring DAO:', error);
    throw error;
  }
};
```

### Module Activation Integration

```typescript
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { createModuleTransaction } from './solana-dao';

// Inside your React component:
const { connection } = useConnection();
const wallet = useWallet();

const handleActivateModule = async (daoId: string, moduleType: "POD" | "POL") => {
  try {
    // Create the transaction
    const { transaction, moduleAccount } = await createModuleTransaction(
      connection,
      wallet,
      daoId,
      moduleType
    );
    
    // Send the transaction to the wallet for signing
    const signature = await wallet.sendTransaction(transaction, connection);
    
    // Confirm the transaction
    await connection.confirmTransaction(signature, 'confirmed');
    
    console.log(`Module ${moduleType} activated successfully!`);
    console.log('Module ID:', moduleAccount.publicKey.toString());
    
    // Now you can update your UI to show the module as activated
    // and store the moduleAccount.publicKey for reference
    
    return moduleAccount.publicKey.toString();
  } catch (error) {
    console.error(`Error activating ${moduleType} module:`, error);
    throw error;
  }
};
```

### API Integration for Featured and Modules

After creating the transactions, you can integrate with your API using similar patterns:

```typescript
// For featured DAO
const { transaction, featuredAccount } = await createFeaturedTransaction(
  connection, 
  wallet, 
  daoId
);

const signature = await wallet.sendTransaction(transaction, connection);
await connection.confirmTransaction(signature, 'confirmed');

// Send to your API
const featuredData = {
  dao_id: daoId,
  pubkey: featuredAccount.publicKey.toString(),
  transaction: signature
};

// For module activation
const { transaction, moduleAccount } = await createModuleTransaction(
  connection, 
  wallet, 
  daoId,
  "POD" // or "POL"
);

const signature = await wallet.sendTransaction(transaction, connection);
await connection.confirmTransaction(signature, 'confirmed');

// Send to your API
const moduleData = {
  dao_id: daoId,
  module_type: "POD", // or "POL"
  pubkey: moduleAccount.publicKey.toString(),
  transaction: signature
};
```
