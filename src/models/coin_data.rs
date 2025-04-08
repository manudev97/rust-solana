use serde::{Deserialize, Serialize};

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