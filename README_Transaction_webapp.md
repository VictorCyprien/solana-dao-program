Here's just the core transaction code you need to integrate into your existing React TypeScript webapp :

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

// Create a new DAO transaction
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
      "profile_url"
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
