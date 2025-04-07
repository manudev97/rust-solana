# PUMP FUN SWAP

This script allows you to buy and sell (swap) tokens on Solana's Pump.fun platform. The script interacts with the Pump.fun program through the Solana blockchain, handling the creation of associated token accounts, price calculations, slippage, and transaction submissions.

## Main Components
1. Configuración (Config)
```rust
pub struct Config {
    pub private_key: String,    // Base58 private key
    pub rpc_endpoint: String,   // RPC endpoint URL
    pub mint: String,           // Mint token address
    pub is_buy: bool,           // true for buying, false for selling
    pub amount: u64,            // Amount in lamports (1 SOL = 1,000,000,000 lamports)
    pub slippage: f64,          // Slippage tolerance (e.g., 0.1 for 10%)
    pub gas_fee: f64,           // Fee priority in SOL
}
```

2. Error Handling (SwapError)
 Lists all possible errors that can occur during the swap.
 ```rust
 pub enum SwapError {
    ...
 }
 ```

**Troubleshooting Guide**

| Error                   | Solution                          |
|------------------------|-----------------------------------|
| RPC connection issues  | Check network, try alternate endpoint |
| Invalid private key    | Verify base58 encoding            |
| Insufficient funds     | Check wallet balance              |
| Token account exists   | Properly handle account creation  |
| Program ID mismatch    | Verify all program addresses      |

3. Token Data (CoinData)
Structure containing token information obtained from the Pump.fun API.

4. Swap Result (SwapResult)
Contains information about the result of the transaction.

## Main Functions
`get_coin_data` Obtains token information from the Pump.fun API.

`get_swap_keys` Generates the AccountMeta keys needed for the swap instruction.

`calculate_fee_on_sol` Calculates the fees applicable to the transaction.

`pump_fun_swap` Main function that:
- Sets up the RPC connection
- Obtains token data
- Builds the instructions
- Sends the transaction

## Execution
- Clone the repository
- Configure .env file with your private key

```rust
cargo build
cargo run --release
```