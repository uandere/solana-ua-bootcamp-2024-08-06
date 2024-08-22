use std::str::FromStr;

use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

use practice_2::{Error, load_keypair};

pub fn create_associated_token_account(
    client: &RpcClient,
    mint: &Pubkey,
    payer: &Keypair,
) -> Result<Signature, Error> {
    let ix = create_associated_token_account_idempotent(
        &payer.pubkey(),
        &payer.pubkey(),
        mint,
        &spl_token::id(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        client.get_latest_blockhash()?,
    );

    let signature = client.send_and_confirm_transaction(&tx)?;

    Ok(signature)
}

pub fn main() -> Result<(), Error> {
    let client = RpcClient::new("https://api.devnet.solana.com");

    let our_keypair = load_keypair()?;

    let mint = Pubkey::from_str("Cyi1orjuKBFQHeLcLpzEQFFdeQd7PVLuRUnZscaUV7kX")?;

    let signature = create_associated_token_account(
        &client,
        &mint,
        &our_keypair,
    )?;

    println!("Token account: {}", get_associated_token_address(
        &our_keypair.pubkey(), &mint
    ));

    println!("Signature: {}", signature);

    Ok(())
}