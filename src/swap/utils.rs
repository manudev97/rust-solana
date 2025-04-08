/// Calculates fee amount and remaining amount after fee
pub fn calculate_fee_on_sol(amount: u64) -> (f64, u64) {
    let fee_percent = 0.001; // 0.1% fee
    let fee_amount = (amount as f64) * fee_percent;
    let amount_after_fee = amount - fee_amount as u64;
    (fee_amount, amount_after_fee)
}

/// Calculates token price based on reserves
pub fn calculate_token_price(virtual_sol_reserves: u64, virtual_token_reserves: u64) -> f64 {
    virtual_sol_reserves as f64 / virtual_token_reserves as f64
}

/// Calculates token output for a given SOL input and token price
pub fn calculate_token_output(sol_input: u64, token_price: f64) -> u64 {
    ((sol_input as f64) / token_price) as u64
}

/// Calculates SOL output for a given token input and token price
pub fn calculate_sol_output(token_input: u64, token_price: f64) -> u64 {
    ((token_input as f64) * token_price) as u64
}

/// Applies slippage to a value (increases for buy, decreases for sell)
pub fn apply_slippage(value: u64, slippage: f64, is_buy: bool) -> u64 {
    if is_buy {
        ((value as f64) * (1.0 + slippage)) as u64
    } else {
        ((value as f64) * (1.0 - slippage)) as u64
    }
}