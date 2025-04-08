use thiserror::Error;
use solana_sdk::program_error::ProgramError;

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
    // #[error("Token error: {0}")]
    // TokenError(String),
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Pubkey parse error: {0}")]
    PubkeyParseError(#[from] solana_sdk::pubkey::ParsePubkeyError),
    #[error("Program error: {0}")]
    ProgramError(#[from] ProgramError),
    // #[error("Unknown error: {0}")]
    // Unknown(String),
}