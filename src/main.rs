mod api;
mod config;
mod error;
mod models;
mod solana;
mod swap;

use config::Config;
use dotenvy::dotenv;
use swap::pump_fun::pump_fun_swap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv().expect("Failed to load .env file");

    let config = Config {
        private_key: std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set in .env file"),
        rpc_endpoint: std::env::var("RPC_ENDPOINT")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        mint: std::env::var("DEFAULT_MINT")
            .unwrap_or_else(|_| "4KrEDuTk2jUWymCzUuc1Y6rosUEBCtd3tdvkecwSpump".to_string()),
        is_buy: true,
        amount: 100000, // 0.0001 SOL
        slippage: 0.1,   // 10%
        gas_fee: 0.00001, // 0.00001 SOL
    };

    match pump_fun_swap(config).await {
        Ok(result) => {
            if result.success {
                println!("Swap successful!");
                println!("TX Hash: {}", result.tx_hash.unwrap());
                println!("Quote In: {}", result.quote_in);
                println!("Quote Out: {}", result.quote_out);
                println!("Fees: {} SOL", result.total_fee_in_sol);
            } else if let Some(error_msg) = &result.error {
                println!("Swap completed but with errors: {}", error_msg);
            }
        }
        Err(e) => {
            println!("Swap error: {}", e);
        }
    }

    Ok(())
}