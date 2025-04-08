use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    sysvar::rent,
};
use std::str::FromStr;
use spl_token::ID as TOKEN_PROGRAM_ID;

use crate::models::coin_data::CoinData;
use crate::solana::constants::*;

/// Generates the required AccountMetas for swap instruction
pub fn get_swap_keys(
    owner: &Pubkey,
    mint: &Pubkey,
    token_account_address: &Pubkey,
    coin_data: &CoinData,
    is_buy: bool,
) -> Vec<AccountMeta> {
    let bonding_curve = Pubkey::from_str(&coin_data.bonding_curve).unwrap();
    let associated_bonding_curve = Pubkey::from_str(&coin_data.associated_bonding_curve).unwrap();
    println!("Bonding Curve: {}", bonding_curve);
    println!("Associated Bonding Curve: {}", associated_bonding_curve);

    vec![
        AccountMeta::new_readonly(*GLOBAL, false),
        AccountMeta::new(*FEE_RECIPIENT, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(bonding_curve, false),
        AccountMeta::new(associated_bonding_curve, false),
        AccountMeta::new(*token_account_address, false),
        AccountMeta::new(*owner, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(
            if is_buy {
                TOKEN_PROGRAM_ID
            } else {
                *ASSOC_TOKEN_ACC_PROG
            },
            false,
        ),
        AccountMeta::new_readonly(
            if is_buy {
                rent::id()
            } else {
                TOKEN_PROGRAM_ID
            },
            false,
        ),
        AccountMeta::new_readonly(*PUMP_FUN_ACCOUNT, false),
        AccountMeta::new_readonly(*PUMP_FUN_PROGRAM, false),
    ]
}

/// Creates a buy swap instruction
pub fn create_buy_instruction(
    owner: &Pubkey,
    mint: &Pubkey,
    token_account_address: &Pubkey,
    coin_data: &CoinData,
    token_out: u64,
    max_sol_cost: u64,
) -> Instruction {
    let keys = get_swap_keys(
        owner,
        mint,
        token_account_address,
        coin_data,
        true,
    );

    Instruction::new_with_bincode(
        *PUMP_FUN_PROGRAM,
        &(PUMP_BUY_METHOD, token_out, max_sol_cost),
        keys,
    )
}

/// Creates a sell swap instruction
pub fn create_sell_instruction(
    owner: &Pubkey,
    mint: &Pubkey,
    token_account_address: &Pubkey,
    coin_data: &CoinData,
    token_amount: u64,
    min_sol_output: u64,
) -> Instruction {
    let keys = get_swap_keys(
        owner,
        mint,
        token_account_address,
        coin_data,
        false,
    );

    Instruction::new_with_bincode(
        *PUMP_FUN_PROGRAM,
        &(PUMP_SELL_METHOD, token_amount, min_sol_output),
        keys,
    )
}