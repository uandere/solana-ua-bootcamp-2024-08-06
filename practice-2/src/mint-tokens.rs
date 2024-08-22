use std::str::FromStr;

use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signature;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_token::instruction::mint_to;

use practice_2::{Error, load_keypair};

pub fn mint_tokens(
    client: &RpcClient,
    mint: &Pubkey,
    payer: &Keypair,
    ata: &Pubkey,
    amount: u64,
) -> Result<Signature, Error> {
    let token_program_id = spl_token::id();

    let mint_to_ix = mint_to(
        &token_program_id,
        mint,
        ata,
        &payer.pubkey(),
        &[&payer.pubkey()],
        amount,
    )?;

    let blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&payer.pubkey()),
        &[payer],
        blockhash
    );

    let signature = client.send_and_confirm_transaction(
        &tx
    )?;

    Ok(signature)
}

pub fn main() -> Result<(), Error> {
    let client = RpcClient::new("https://api.devnet.solana.com");
    let our_keypair = load_keypair()?;
    let mint = Pubkey::from_str("Cyi1orjuKBFQHeLcLpzEQFFdeQd7PVLuRUnZscaUV7kX")?;
    let ata = Pubkey::from_str("8zuf3F6YGKx31nC8mVceJRvXSMKd62hPQibzZp1Yreoq")?;

    let signature = mint_tokens(
        &client,
        &mint,
        &our_keypair,
        &ata,
        10 * 10_u64.pow(9) - 10,
    )?;

    println!("Success! Mint Token Transaction: {}", signature);

    Ok(())
}