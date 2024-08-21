use std::env::VarError;
use dotenv::dotenv;
use solana_client::client_error::ClientError;
use solana_program::pubkey::ParsePubkeyError;
use solana_sdk::signature::Keypair;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ParsePubkey(#[from] ParsePubkeyError),
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error(transparent)]
    Var(#[from] VarError),
    #[error(transparent)]
    BadKeypair(#[from] ed25519_dalek::SignatureError),
    #[error(transparent)]
    ProgramError(#[from] solana_sdk::program_error::ProgramError),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

/// Loads keypair from `.env` file.
/// `.env` file is expected to be inside the current working directory.
pub fn load_keypair() -> Result<Keypair, Error> {
    dotenv().ok();
    let secret_key = std::env::var("SECRET_KEY")?;

    let secret_key_bytes: Vec<u8> = serde_json::from_str(&secret_key)?;

    let secret_key_array: [u8; 64] = secret_key_bytes.try_into().expect("Expected 64 bytes");

    let keypair = Keypair::from_bytes(&secret_key_array)?;

    Ok(keypair)
}