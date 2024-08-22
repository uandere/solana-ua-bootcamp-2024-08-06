use std::str::FromStr;

use mpl_token_metadata::instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs};
use mpl_token_metadata::types::DataV2;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::signature::Signature;
use solana_program::pubkey::Pubkey;

use std::env::VarError;
use dotenv::dotenv;
use solana_client::client_error::ClientError;
use solana_program::pubkey::ParsePubkeyError;
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


pub fn create_token_metadata(
    client: &RpcClient,
    mint: &Pubkey,
    payer: &Keypair,
) -> Result<Signature, Error> {
    let token_metadata_program_id = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")?;
    
    let metadata_data = DataV2 {
        name: "NAZAR".to_string(),
        symbol: "DEMCHUK".to_string(),
        uri: "https://arweave.net/1234".to_string(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let args = CreateMetadataAccountV3InstructionArgs {
        data: metadata_data,
        is_mutable: false,
        collection_details: None,
    };

    let (metadata_pda, _metadata_bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            token_metadata_program_id.as_ref(),
            mint.as_ref(),
        ],
        &token_metadata_program_id,
    );

    let create_metadata_account_instruction = CreateMetadataAccountV3 {
        metadata: metadata_pda,
        mint: *mint,
        mint_authority: payer.pubkey(),
        payer: payer.pubkey(),
        update_authority: (payer.pubkey(), false),
        system_program: solana_program::system_program::id(),
        rent: None,
    };
    
    let ix = create_metadata_account_instruction.instruction(args);
    
    let blockhash = client.get_latest_blockhash()?;
    
    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        blockhash
    );
    
    let signature = client.send_and_confirm_transaction(&transaction)?;

    Ok(signature)
}

pub fn main() -> Result<(), Error> {
    let client = RpcClient::new("https://api.devnet.solana.com");

    let our_keypair = load_keypair()?;

    let mint = Pubkey::from_str("Cyi1orjuKBFQHeLcLpzEQFFdeQd7PVLuRUnZscaUV7kX")?;
    
    let signature = create_token_metadata(
        &client,
        &mint,
        &our_keypair,
    )?;
    
    println!("Success, signature is: {}", signature);
    println!("New token mint: {}", mint);
    
    Ok(())
}