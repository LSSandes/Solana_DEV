The Dapp is a claiming system for the Solana Coin and a SPL token. It must fetch a dataset from a database that will populate two merkletrees within the program. The first merkletree will be for users claiming Solana and the second will be for claiming a SPL token. A simple UI to test the functionality where it will populate the program merkletrees from the dataset given from the database, and have a button that will claim the available balance for that user.  The dataset will be handed to you for testing so no need to make a data base. for the claiming period after said period for the claiming we would like to be able to send the unclaimed tokens back to a wallet. Example, if the claiming period is 90 days then after this 90 days we would like to send the unclaimed tokens and SOL back to a designated wallet.

### Flow Summary

1. **Data Retrieval**: Fetch the dataset from the database.
2. **Merkletree Setup**: Populate two merkletrees for Solana Coin and SPL token.
3. **UI Testing**:
   - Display claimable balances.
   - Button to claim balances.
4. **Claim Periods**:
   - Manage weekly (or defined period) claim periods.
   - Ensure unclaimed balances carry over to future periods (up to 90 days).
5. **User Interaction**:
   - Users interact with the UI to claim their available balances.

### Data Fields needed for the dataset 

- **Users**
  - `user_id`: Unique identifier for each user.
  - `wallet_address`: Solana wallet address to which the tokens will be sent.
  - `claims`: List of claims associated with the user.

- **Claims**
  - `claim_id`: Unique identifier for each claim record.
  - `period_id`: Identifier for the claim period (e.g., which week the claim belongs to).
  - `solana_balance`: The amount of Solana Coin the user can claim.
  - `spl_balance`: The amount of SPL token the user can claim.
  - `is_claimed`: A boolean flag to check if the user has already claimed the balance.

- **Periods**
  - `period_id`: Unique identifier for each period.
  - `start_date`: The start date of the claim period.
  - `end_date`: The end date of the claim period.
  - `merkle_root`: The root hash of the merkletree for that period.
  - `merkle_proof`: List of proofs needed for users to validate their claims within the merkletree.
