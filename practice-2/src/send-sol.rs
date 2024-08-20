use std::str::FromStr;

use solana_client::rpc_client::RpcClient;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::transfer;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;

use practice_2::{Error, load_keypair};

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
