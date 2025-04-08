#[derive(Debug)]
pub struct Config {
    pub private_key: String,  // Base58 encoded private key
    pub rpc_endpoint: String, // Solana RPC endpoint URL
    pub mint: String,         // Token mint address
    pub is_buy: bool,         // Buy (true) or sell (false) operation
    pub amount: u64,          // Amount in lamports (1 SOL = 1_000_000_000 lamports)
    pub slippage: f64,        // Slippage tolerance (0.1 = 10%)
    pub gas_fee: f64,         // Priority fee in SOL
}

// SwapResult structure moved here from main
#[derive(Debug)]
pub struct SwapResult {
    pub success: bool,           // Operation status
    pub tx_hash: Option<String>, // Transaction signature
    pub quote_in: u64,           // Input amount (lamports)
    pub quote_out: u64,          // Output amount (tokens or lamports)
    pub total_fee_in_sol: f64,   // Total fees paid in SOL
    pub error: Option<String>,   // Error message if any
}