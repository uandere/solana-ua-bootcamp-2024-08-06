mod r#create

use std::env::VarError;
use std::str::FromStr;

use dotenv::dotenv;
use solana_client::client_error::ClientError;
use solana_client::rpc_client::RpcClient;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::{ParsePubkeyError, Pubkey};
use solana_program::system_instruction::transfer;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
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
    Other(#[from] serde_json::Error),
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

pub fn send_sol_with_memo(
    client: &RpcClient,
    sender: &Keypair,
    recipient: &Pubkey,
    amount: u64,
    message: &str,
) -> Result<Signature, Error> {
    
    let transfer_ix = transfer(&sender.pubkey(), recipient, amount);
    
    let memo_ix = spl_memo::build_memo(message.as_bytes(), &[&sender.pubkey()]);

    let blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, memo_ix],
        Some(&sender.pubkey()),
        &[&sender],
        blockhash);

    let signature = client.send_and_confirm_transaction(&tx)?;

    Ok(signature)
}


fn main() -> Result<(), Error> {
    
    let client = RpcClient::new("https://api.devnet.solana.com");
    
    let our_keypair = load_keypair()?;
    let recipient = Pubkey::from_str("EQaSfMikgUoiKBBZLzvsTX3aArBBC38WsiZ6tfcSazgp")?;
    let memo = "Hello, Nazar";
    
    println!("Memo is: {}", memo);

    println!("💸 Attempting to send 0.001 SOL to {}...", recipient);

    let signature = send_sol_with_memo(&client, &our_keypair, &recipient,
                                       LAMPORTS_PER_SOL / 1000, memo)?;
    
    println!("✅ Transaction confirmed, signature: {signature}");

    Ok(())
}
