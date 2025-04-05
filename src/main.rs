use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::{Instruction, AccountMeta},
    system_program,
    system_instruction,
    sysvar::rent,
    commitment_config::CommitmentConfig,
    program_error::ProgramError,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use spl_token_2022::{
    instruction::{close_account, sync_native},
    ID as TOKEN_PROGRAM_2022_ID,
};
use reqwest::header;
use serde::{Deserialize, Serialize};
use log::error;
use thiserror::Error;
use bs58;
use std::str::FromStr;
use dotenvy::dotenv;

// Global constants for Pump.fun program addresses
lazy_static::lazy_static! {
    static ref GLOBAL: Pubkey = Pubkey::from_str("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf").unwrap();
    static ref FEE_RECIPIENT: Pubkey = Pubkey::from_str("CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM").unwrap();
    static ref PUMP_FUN_PROGRAM: Pubkey = Pubkey::from_str("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P").unwrap();
    static ref PUMP_FUN_ACCOUNT: Pubkey = Pubkey::from_str("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1").unwrap();
    static ref ASSOC_TOKEN_ACC_PROG: Pubkey = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();
}

/// Configuration structure for swap operations
#[derive(Debug)]
pub struct Config {
    pub private_key: String,     // Base58 encoded private key
    pub rpc_endpoint: String,    // Solana RPC endpoint URL
    pub mint: String,            // Token mint address
    pub is_buy: bool,            // Buy (true) or sell (false) operation
    pub amount: u64,             // Amount in lamports (1 SOL = 1_000_000_000 lamports)
    pub slippage: f64,           // Slippage tolerance (0.1 = 10%)
    pub gas_fee: f64,            // Priority fee in SOL
}

/// Custom error enum for swap operations
#[derive(Error, Debug)]
pub enum SwapError {
    #[error("RPC error: {0}")]
    RpcError(#[from] solana_client::client_error::ClientError),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Base58 decoding error: {0}")]
    Base58Error(#[from] bs58::decode::Error),
    #[error("Invalid keypair")]
    InvalidKeypair,
    #[error("Swap API error: {0}")]
    ApiError(String),
    #[error("Token error: {0}")]
    TokenError(String),
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Pubkey parse error: {0}")]
    PubkeyParseError(#[from] solana_sdk::pubkey::ParsePubkeyError),
    #[error("Program error: {0}")]
    ProgramError(#[from] ProgramError),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Structure representing token data from Pump.fun API
#[derive(Debug, Serialize, Deserialize)]
pub struct CoinData {
    pub mint: String,                     // Token mint address
    pub name: String,                     // Token name
    pub symbol: String,                   // Token symbol
    pub bonding_curve: String,            // Bonding curve address
    pub associated_bonding_curve: String, // Associated bonding curve address
    pub virtual_sol_reserves: u64,        // SOL reserves in bonding curve
    pub virtual_token_reserves: u64,      // Token reserves in bonding curve
    #[serde(flatten)]
    pub extra: serde_json::Value,         // Additional API data
}

/// Result structure for swap operations
#[derive(Debug)]
pub struct SwapResult {
    pub success: bool,           // Operation status
    pub tx_hash: Option<String>, // Transaction signature
    pub quote_in: u64,           // Input amount (lamports)
    pub quote_out: u64,          // Output amount (tokens or lamports)
    pub total_fee_in_sol: f64,   // Total fees paid in SOL
    pub error: Option<String>,   // Error message if any
}

/// Creates an HTTP client with Pump.fun specific headers
async fn create_http_client() -> Result<reqwest::Client, SwapError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:125.0) Gecko/20100101 Firefox/125.0")
        .default_headers({
            let mut headers = header::HeaderMap::new();
            headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
            headers.insert(header::ACCEPT_LANGUAGE, header::HeaderValue::from_static("en-US,en;q=0.5"));
            headers.insert(header::REFERER, header::HeaderValue::from_static("https://www.pump.fun/"));
            headers.insert(header::ORIGIN, header::HeaderValue::from_static("https://www.pump.fun"));
            headers
        })
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    Ok(client)
}

/// Fetches token data from Pump.fun API
pub async fn get_coin_data(mint_str: &str) -> Result<CoinData, SwapError> {
    let client = create_http_client().await?;
    let url = format!("https://frontend-api-v3.pump.fun/coins/{}", mint_str);
    
    let response = client.get(&url).send().await?;
    
    if response.status().is_success() {
        let coin_data = response.json::<CoinData>().await?;
        Ok(coin_data)
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        Err(SwapError::ApiError(format!("API returned status {}: {}", status, error_text)))
    }
}

/// Generates the required AccountMetas for swap instruction
fn get_swap_keys(
    owner: &Pubkey,
    mint: &Pubkey,
    token_account_address: &Pubkey,
    coin_data: &CoinData,
    is_buy: bool
) -> Vec<AccountMeta> {
    let bonding_curve = Pubkey::from_str(&coin_data.bonding_curve).unwrap();
    let associated_bonding_curve = Pubkey::from_str(&coin_data.associated_bonding_curve).unwrap();
    
    vec![
        AccountMeta::new_readonly(*GLOBAL, false),
        AccountMeta::new(*FEE_RECIPIENT, true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(bonding_curve, true),
        AccountMeta::new(associated_bonding_curve, true),
        AccountMeta::new(*token_account_address, true),
        AccountMeta::new(*owner, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(
            if is_buy { TOKEN_PROGRAM_2022_ID } else { *ASSOC_TOKEN_ACC_PROG }, 
            false
        ),
        AccountMeta::new_readonly(
            if is_buy { rent::id() } else { TOKEN_PROGRAM_2022_ID }, 
            false
        ),
        AccountMeta::new_readonly(*PUMP_FUN_ACCOUNT, false),
        AccountMeta::new_readonly(*PUMP_FUN_PROGRAM, false),
    ]
}

/// Calculates fee amount and remaining amount after fee
fn calculate_fee_on_sol(amount: u64) -> (f64, u64) {
    let fee_percent = 0.001;  // 0.1% fee
    let fee_amount = (amount as f64) * fee_percent;
    let amount_after_fee = amount - fee_amount as u64;
    (fee_amount, amount_after_fee)
}

/// Converts u64 to little-endian bytes
fn u64_to_le_bytes(value: u64) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&value.to_le_bytes());
    bytes
}

/// Main swap function handling both buy and sell operations
pub async fn pump_fun_swap(config: Config) -> Result<SwapResult, SwapError> {
    // Decode private key and create keypair
    let private_key_bytes = bs58::decode(&config.private_key).into_vec()?;
    let payer = Keypair::from_bytes(&private_key_bytes).map_err(|_| SwapError::InvalidKeypair)?;
    let owner = payer.pubkey();
    
    // Fetch token data from Pump.fun API
    let coin_data = get_coin_data(&config.mint).await?;
    let mint = Pubkey::from_str(&config.mint)?;
    
    // Initialize RPC client
    let client = RpcClient::new_with_commitment(
        config.rpc_endpoint.clone(), 
        CommitmentConfig::confirmed()
    );
    
    // Check wallet balance
    let balance = client.get_balance(&owner)?;
    if balance < config.amount {
        return Err(SwapError::InsufficientFunds);
    }
    
    // Calculate fees and amounts
    let (fee_amount, amount_after_fee) = calculate_fee_on_sol(config.amount);
    let token_price = coin_data.virtual_sol_reserves as f64 / coin_data.virtual_token_reserves as f64;
    
    // Prepare swap data based on operation type (buy/sell)
    let (quote_amount, data) = if config.is_buy {
        let token_out = ((amount_after_fee as f64) / token_price) as u64;
        let sol_in_with_slippage = (amount_after_fee as f64) * (1.0 + config.slippage);
        let max_sol_cost = sol_in_with_slippage as u64;
        
        let data = [
            u64_to_le_bytes(16927863322537952870), // Buy instruction discriminator
            u64_to_le_bytes(token_out),
            u64_to_le_bytes(max_sol_cost),
        ].concat();
        
        (token_out, data)
    } else {
        let min_sol_output = ((config.amount as f64 * token_price) * (1.0 - config.slippage)) as u64;
        
        let data = [
            u64_to_le_bytes(12502976635542562355), // Sell instruction discriminator
            u64_to_le_bytes(config.amount),
            u64_to_le_bytes(min_sol_output),
        ].concat();
        
        (min_sol_output, data)
    };
    
    // Prepare swap instruction
    let token_account_address = get_associated_token_address(&owner, &mint);
    let keys = get_swap_keys(&owner, &mint, &token_account_address, &coin_data, config.is_buy);
    
    let swap_instruction = Instruction {
        program_id: *PUMP_FUN_PROGRAM,
        accounts: keys,
        data,
    };
    
    // Prepare token accounts
    let token_account_in = get_associated_token_address(&owner, if config.is_buy { &spl_token_2022::native_mint::ID } else { &mint });
    let token_account_out = get_associated_token_address(&owner, if config.is_buy { &mint } else { &spl_token_2022::native_mint::ID });
    
    // Build transaction instructions
    let mut instructions = vec![];
    
    // Add priority fee instruction
    instructions.push(ComputeBudgetInstruction::set_compute_unit_price((config.gas_fee * 1_000_000.0) as u64));
    
    // Add operation-specific instructions
    if config.is_buy {
        // Buy instructions
        instructions.push(create_associated_token_account(
            &owner, &owner, &spl_token_2022::native_mint::ID, &TOKEN_PROGRAM_2022_ID
        ));
        
        instructions.push(system_instruction::transfer(
            &owner, &token_account_in, amount_after_fee
        ));
        
        instructions.push(sync_native(
            &token_account_in, &TOKEN_PROGRAM_2022_ID
        )?);
        
        instructions.push(create_associated_token_account(
            &owner, &owner, &mint, &TOKEN_PROGRAM_2022_ID
        ));
    } else {
        // Sell instructions
        instructions.push(create_associated_token_account(
            &owner, &owner, &spl_token_2022::native_mint::ID, &TOKEN_PROGRAM_2022_ID
        ));
    }
    
    // Add swap instruction
    instructions.push(swap_instruction);
    
    // Clean up temporary accounts
    if config.is_buy {
        instructions.push(close_account(
            &token_account_in,
            &owner,
            &owner,
            &spl_token_2022::native_mint::ID,
            &[]
        )?);
    } else {
        instructions.push(close_account(
            &token_account_out,
            &owner,
            &owner,
            &spl_token_2022::native_mint::ID,
            &[]
        )?);
    }
    
    // Build, sign and send transaction
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&owner));
    let recent_blockhash = client.get_latest_blockhash()?;
    transaction.sign(&[&payer], recent_blockhash);
    let signature = client.send_and_confirm_transaction(&transaction)?;
    
    Ok(SwapResult {
        success: true,
        tx_hash: Some(signature.to_string()),
        quote_in: if config.is_buy { amount_after_fee } else { config.amount },
        quote_out: quote_amount,
        total_fee_in_sol: fee_amount,
        error: None,
    })
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv().expect("Failed to load .env file");
    
    let config = Config {
        private_key: std::env::var("PRIVATE_KEY")
            .expect("PRIVATE_KEY must be set in .env file"),
        rpc_endpoint: std::env::var("RPC_ENDPOINT")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        mint: std::env::var("DEFAULT_MINT")
            .unwrap_or_else(|_| "4KrEDuTk2jUWymCzUuc1Y6rosUEBCtd3tdvkecwSpump".to_string()),
        is_buy: true,
        amount: 1000000,  // 0.001 SOL
        slippage: 0.1,    // 10%
        gas_fee: 0.0001,  // 0.0001 SOL
    };
    
    match pump_fun_swap(config).await {
        Ok(result) => {
            println!("Swap successful!");
            println!("TX Hash: {}", result.tx_hash.unwrap());
            println!("Quote In: {}", result.quote_in);
            println!("Quote Out: {}", result.quote_out);
            println!("Fees: {} SOL", result.total_fee_in_sol);
        },
        Err(e) => {
            println!("Swap error: {}", e);
        }
    }
    
    Ok(())
}