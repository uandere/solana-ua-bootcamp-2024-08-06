use std::env::VarError;
use std::str::FromStr;
use dotenv::dotenv;
use solana_client::client_error::ClientError;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcRequestAirdropConfig;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::{ParsePubkeyError, Pubkey};
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::{Keypair, Signer};
use thiserror::Error;


#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    ParsePubkey (#[from] ParsePubkeyError),
    #[error(transparent)]
    Client (#[from] ClientError),
    #[error(transparent)]
    Var (#[from] VarError),
    #[error(transparent)]
    BadKeypair (#[from] ed25519_dalek::SignatureError),
    #[error(transparent)]
    Other (#[from] serde_json::Error)
}

fn check_balance() -> Result<(), Error> {
    const CLUSTER_URI: &str = "https://api.devnet.solana.com";
    let client = RpcClient::new(CLUSTER_URI);

    println!("⚡️ Connected to devnet");

    let local_wallet_pubkey =
        Pubkey::from_str("Dw1qLN2zozkt9NEwDjHa6eVxLJFA6jgE3ENPrV8YEbei")?;

    let _signature = client.request_airdrop_with_config(
        &local_wallet_pubkey,
        LAMPORTS_PER_SOL,
        RpcRequestAirdropConfig {
            recent_blockhash: None,
            commitment: Some(CommitmentConfig { commitment: CommitmentLevel::Confirmed }),
        })?;

    let balance_lamports = client.get_balance(&local_wallet_pubkey)?;

    let balance_in_sol = balance_lamports / LAMPORTS_PER_SOL;

    println!("💰 The balance for the wallet at address ${} is: ${}", local_wallet_pubkey, balance_in_sol);

    Ok(())
}

fn generate_keypair() {
    let keypair = Keypair::new();
    println!("The public key is: {}", keypair.pubkey());
    println!("The secret key is: {:?}", keypair.secret());
}

fn load_keypair() -> Result<(), Error> {
    dotenv().ok();
    let secret_key = std::env::var("SECRET_KEY")?;

    let secret_key_bytes: Vec<u8> = serde_json::from_str(&secret_key)?;

    let secret_key_array: [u8; 64] = secret_key_bytes.try_into().expect("Expected 64 bytes");

    let keypair = Keypair::from_bytes(&secret_key_array)?;
    
    println!("Public key: {}", keypair.pubkey());
    println!("Secret key: {:?}", keypair.secret());
    
    Ok(())
}

fn main() {
    // Demonstrating how generating keypair can be done.
    generate_keypair();

    // Loading Keypair from a file
    load_keypair().unwrap();

    // Checking balance on a local machine wallet (devnet)
    check_balance().unwrap();
}
