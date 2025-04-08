use bs58;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

use crate::api::get_coin_data;
use crate::config::{Config, SwapResult};
use crate::error::SwapError;
use crate::solana::instructions::{create_buy_instruction, create_sell_instruction};
use crate::solana::txs::{
    build_and_send_transaction, create_rpc_client, prepare_buy_instructions, prepare_sell_instructions,
};
use crate::swap::utils::{
    apply_slippage, calculate_fee_on_sol, calculate_sol_output, calculate_token_output,
    calculate_token_price,
};

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
    let client = create_rpc_client(&config.rpc_endpoint);

    // Check wallet balance
    let balance = client.get_balance(&owner)?;
    if balance < config.amount {
        return Err(SwapError::InsufficientFunds);
    }

    // Calculate fees and amounts
    let (fee_amount, amount_after_fee) = calculate_fee_on_sol(config.amount);
    let token_price = calculate_token_price(
        coin_data.virtual_sol_reserves,
        coin_data.virtual_token_reserves,
    );

    // Prepare swap data based on operation type (buy/sell)
    let (quote_amount, instructions) = if config.is_buy {
        // Calculate token output and max SOL cost with slippage
        let token_out = calculate_token_output(amount_after_fee, token_price);
        let max_sol_cost = apply_slippage(amount_after_fee, config.slippage, true);

        // Ensure token account exists for receiving tokens
        let token_account_address = get_associated_token_address(&owner, &mint);
        
        // Create buy instruction
        let swap_instruction = create_buy_instruction(
            &owner,
            &mint,
            &token_account_address,
            &coin_data,
            token_out,
            max_sol_cost,
        );

        // Prepare all instructions for the transaction
        let instructions = prepare_buy_instructions(
            &owner,
            &mint,
            &token_account_address,
            &coin_data,
            amount_after_fee,
            config.gas_fee,
            swap_instruction,
            &client,
        )?;

        (token_out, instructions)
    } else {
        // For selling: calculate SOL output with slippage
        let token_amount = config.amount;
        let sol_out = calculate_sol_output(token_amount, token_price);
        let min_sol_output = apply_slippage(sol_out, config.slippage, false);

        // Prepare for selling tokens
        let token_account_address = get_associated_token_address(&owner, &mint);
        
        // Create sell instruction
        let swap_instruction = create_sell_instruction(
            &owner,
            &mint,
            &token_account_address,
            &coin_data,
            token_amount,
            min_sol_output,
        );

        // Prepare all instructions for the transaction
        let instructions = prepare_sell_instructions(config.gas_fee, swap_instruction);

        (min_sol_output, instructions)
    };

    // Build and send transaction
    let signature = build_and_send_transaction(&client, &payer, instructions).await?;

    Ok(SwapResult {
        success: true,
        tx_hash: Some(signature),
        quote_in: if config.is_buy {
            amount_after_fee
        } else {
            config.amount
        },
        quote_out: quote_amount,
        total_fee_in_sol: fee_amount,
        error: None,
    })
}