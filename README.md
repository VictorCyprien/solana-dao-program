# Solana DAO Program

A Solana smart contract for creating and managing Decentralized Autonomous Organizations (DAOs), proposals, and voting on-chain.

## Overview

The Solana DAO Program provides a complete solution for creating and managing DAOs on the Solana blockchain. It allows users to:

- Create DAOs with customizable metadata
- Create proposals within DAOs
- Vote on proposals with "for", "against" options
- Manage fees automatically with a dynamic fee system

## Features

- **Create a DAO** - Establish a new DAO with comprehensive metadata including social links and web presence
- **Create a Proposal** - Submit proposals to a DAO with customizable start and end times
- **Vote on Proposals** - Community members can vote on active proposals
- **Dynamic Fee System** - The DAO creation fee is fixed at $20 USD but dynamically adjusts based on the current SOL price
- **Fee Distribution** - Each function sends fees to a specified wallet address

## Dynamic Fee System

The DAO creation fee is fixed at $20 USD but dynamically adjusts based on the current SOL price:

- The client fetches the current SOL price from an API (CoinGecko)
- This price is passed to the smart contract as a parameter
- The contract calculates how much SOL equals $20 USD at the current exchange rate
- This ensures the fee remains at a consistent USD value regardless of SOL price fluctuations

## Account Structures

### DAO Structure

```rust
struct Dao {
    authority: Pubkey,      // Creator of the DAO
    name: String,           // Name of the DAO
    description: String,    // Description of the DAO
    discord_server: String, // Discord server link
    twitter: String,        // Twitter handle
    telegram: String,       // Telegram group link
    instagram: String,      // Instagram handle
    tiktok: String,         // TikTok handle
    website: String,        // Official website URL
    treasury: String,       // Treasury account address
    profile: String,        // Profile image URL
    token_address: String,  // Token address associated with the DAO
}
```

### Proposal Structure

```rust
struct Proposal {
    authority: Pubkey,      // Creator of the proposal
    name: String,           // Name of the proposal
    description: String,    // Description of the proposal
    dao_id: String,         // DAO public key this proposal belongs to
    pod_id: String,         // Optional sub-group ID (can be empty)
    start_time: i64,        // Start time for voting (unix timestamp)
    end_time: i64,          // End time for voting (unix timestamp)
}
```

### Vote Structure

```rust
struct Vote {
    voter: Pubkey,          // Public key of the voter
    vote: String,           // "yes", "no", or "abstain"
    proposal_id: String,    // Proposal public key this vote is for
}
```

## Building and Deploying

### Building the Program

```bash
cargo build-sbf
```

### Deploying the Program

```bash
solana program deploy target/deploy/dao_program.so
```

## Using the JavaScript Client

The project includes a JavaScript client for interacting with the DAO program. It handles:

- Serializing instruction data
- Creating transactions
- Managing accounts
- Fetching SOL price for dynamic fee calculation

### Installation

```bash
npm install @solana/web3.js @solana/buffer-layout bn.js buffer node-fetch
```

### Usage Example

```javascript
import { Connection, PublicKey } from '@solana/web3.js';
import { 
  createDaoTransaction, 
  createProposalTransaction, 
  createVoteTransaction 
} from './solana-dao';

// Create a connection to the Solana network
const connection = new Connection('https://api.devnet.solana.com');

// Example: Create a DAO
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

## Integration with Web Applications

The program is designed to be easily integrated with web applications using Solana wallet adapters:

```typescript
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { createDaoTransaction } from './solana-dao';

// Inside your React component:
const { connection } = useConnection();
const wallet = useWallet();

// Use the transaction functions as shown in the example above
```

## Security Considerations

- The program enforces that the fee recipient address is valid
- All functions verify that the transaction signer has the proper authority
- The SOL price is validated to be within reasonable bounds to prevent manipulation
- For proposals, the program validates that start time is after the current time and end time is after start time
- For votes, the program validates that the vote is one of the allowed values ("yes", "no", or "abstain")
- The program checks if the creator has sufficient funds for transaction fees

## Error Handling

The program defines specific error types to handle various error cases:

- `InvalidInstruction`: Instruction data couldn't be deserialized
- `NotRentExempt`: Account doesn't have enough lamports to be rent exempt
- `ExpectedAmountMismatch`: Fee amount doesn't match the expected amount
- `InvalidFeeAccount`: Fee recipient address is invalid
- `ProposalTimeInvalid`: Proposal times are invalid
- `InvalidVote`: Vote value is not one of the allowed values
- `InsufficientFunds`: User doesn't have enough funds
- `InvalidSolPrice`: SOL price is not within reasonable bounds

## License

MIT 