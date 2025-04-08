use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::ID as TOKEN_PROGRAM_ID;
use std::str::FromStr;

use crate::error::SwapError;
use crate::models::coin_data::CoinData;

/// Builds and sends a transaction with the given instructions
pub async fn build_and_send_transaction(
    client: &RpcClient,
    payer: &Keypair,
    instructions: Vec<Instruction>,
) -> Result<String, SwapError> {
    let owner = payer.pubkey();
    
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&owner));
    let recent_blockhash = client.get_latest_blockhash()?;
    transaction.sign(&[payer], recent_blockhash);
    
    println!("🚚 Enviando transacción...");
    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("✅ Transacción confirmada: {}", signature);
    
    Ok(signature.to_string())
}

/// Prepares the transaction instructions for a buy swap
pub fn prepare_buy_instructions(
    owner: &Pubkey,
    mint: &Pubkey,
    token_account_out: &Pubkey,
    coin_data: &CoinData,
    amount_after_fee: u64,
    gas_fee: f64,
    swap_instruction: Instruction,
    client: &RpcClient,
) -> Result<Vec<Instruction>, SwapError> {
    let mut instructions = Vec::new();
    
    // Add priority fee instruction
    instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
        (gas_fee * 1_000_000.0) as u64,
    ));
    
    // Check if token account exists and create if needed
    match client.get_account(token_account_out) {
        Ok(_) => println!("Token account already exists, skipping creation"),
        Err(_) => {
            println!("Creating token account for receiving tokens");
            instructions.push(create_associated_token_account(
                owner,
                owner,
                mint,
                &TOKEN_PROGRAM_ID,
            ));
        }
    }
    
    // Transfer SOL to the program
    instructions.push(system_instruction::transfer(
        owner,
        &Pubkey::from_str(&coin_data.bonding_curve).unwrap(),
        amount_after_fee,
    ));
    
    // Add swap instruction
    instructions.push(swap_instruction);
    
    Ok(instructions)
}

/// Prepares the transaction instructions for a sell swap
pub fn prepare_sell_instructions(
    gas_fee: f64,
    swap_instruction: Instruction,
) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    
    // Add priority fee instruction
    instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
        (gas_fee * 1_000_000.0) as u64,
    ));
    
    // Add swap instruction
    instructions.push(swap_instruction);
    
    instructions
}

/// Creates a Solana RPC client with the given endpoint
pub fn create_rpc_client(rpc_endpoint: &str) -> RpcClient {
    RpcClient::new_with_commitment(
        rpc_endpoint.to_string(),
        CommitmentConfig::confirmed(),
    )
}