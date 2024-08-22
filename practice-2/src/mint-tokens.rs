use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signature;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::mint_to;

use practice_2::{Error, load_keypair};

pub fn mint_tokens(
    client: RpcClient,
    mint_pubkey: &Pubkey,
    payer: &Keypair,
    amount: u64,
) -> Result<Signature, Error> {
    let token_program_id = spl_token::id();

    // TODO: Change this to constant value
    let associated_acc_pubkey = get_associated_token_address(
        &payer.pubkey(),
        mint_pubkey,
    );

    let mint_to_ix = mint_to(
        &token_program_id,
        mint_pubkey,
        &associated_acc_pubkey,
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

    Ok(())
}