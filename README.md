# Solana DAO Program

A Solana smart contract for creating and managing DAOs, proposals, and voting.

## Features

- Create a DAO (with a $20 USD fee that adjusts based on SOL price)
- Create a proposal for a DAO
- Vote on proposals
- Each function sends fees to a specified wallet address

## Dynamic Fee System

The DAO creation fee is fixed at $20 USD but dynamically adjusts based on the current SOL price:

- The client fetches the current SOL price from an API (CoinGecko)
- This price is passed to the smart contract as a parameter
- The contract calculates how much SOL equals $20 USD at the current exchange rate
- This ensures the fee remains at a consistent USD value regardless of SOL price fluctuations

## DAO Structure

```rust
struct Dao {
    authority: Pubkey,
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
}
```

## Proposal Structure

```rust
struct Proposal {
    authority: Pubkey,
    name: String,
    description: String,
    dao_id: String,
    pod_id: String, // Can be empty
    start_time: i64,
    end_time: i64,
}
```

## Vote Structure

```rust
struct Vote {
    voter: Pubkey,
    vote: String, // "yes", "no", or "abstain"
    proposal_id: String,
}
```

## Building the Program

```bash
cargo build-bpf
```

## Deploying the Program

```bash
solana program deploy target/deploy/dao_program.so
```

## Using the JavaScript Client

The `examples/client.js` file provides a JavaScript client for interacting with the DAO program. Before using it, make sure to:

1. Install the required dependencies:

```bash
npm install @solana/web3.js @solana/buffer-layout bn.js buffer node-fetch
```

2. Update the `programId` variable with your deployed program's ID:

```javascript
const programId = new PublicKey('YOUR_PROGRAM_ID_HERE');
```

3. Use the client functions:

```javascript
// Create a DAO - automatically fetches current SOL price and calculates fee
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
  'treasury_account_pubkey',
  'profile_url'
);

// Create a proposal
const proposalId = await createProposal(
  payer,
  'My Proposal',
  'A description of my proposal',
  daoId.toString(),
  '', // Empty pod_id
  startTime,
  endTime
);

// Vote on the proposal
await vote(
  payer,
  'yes', // Can be 'yes', 'no', or 'abstain'
  proposalId.toString()
);
```

## Security Considerations

- This program enforces that the fee recipient address is valid
- All functions verify that the transaction signer has the proper authority
- The SOL price is validated to be within reasonable bounds to prevent manipulation
- For proposals, the program validates that start time is after the current time and end time is after start time
- For votes, the program validates that the vote is one of the allowed values ("yes", "no", or "abstain")
- The program checks if the creator has sufficient funds for transaction fees

## License

MIT 