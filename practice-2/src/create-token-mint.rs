use solana_client::rpc_client::RpcClient;
use solana_program::message::Message;
use solana_program::program_pack::Pack;
use solana_program::system_instruction;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signature;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_token::instruction::initialize_mint;
use spl_token::state::Mint;

use practice_2::{Error, load_keypair};

pub fn create_token_mint(
    client: &RpcClient,
    payer: &Keypair,
    decimals: u8,
) -> Result<Signature, Error> {
    let token_program_id = spl_token::id();
    let mint = Keypair::new();

    let mint_rent = client.get_minimum_balance_for_rent_exemption(Mint::LEN)?;

    // Create the mint account
    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint.pubkey(),
        mint_rent,
        Mint::LEN as u64,
        &token_program_id,
    );
    
    let initialize_mint_ix = initialize_mint(
        &token_program_id,
        &mint.pubkey(),
        &payer.pubkey(),
        Some(&payer.pubkey()),
        decimals,
    )?;
    
    let message = Message::new(
        &[create_account_ix, initialize_mint_ix],
        Some(&payer.pubkey()),
    );

    let blockhash = client.get_latest_blockhash()?;
    
    let tx = Transaction::new(
        &[&payer, &mint],
        message,
        blockhash,
    );
    
    let signature = client.send_and_confirm_transaction(&tx)?;


    println!("✅ Token Mint Pubkey: {}", mint.pubkey());

    Ok(signature)
}

pub fn main() -> Result<(), Error> {
    let client = RpcClient::new("https://api.devnet.solana.com");

    let our_keypair = load_keypair()?;

    let signature = create_token_mint(&client, &our_keypair, 9)?;

    println!("✅ Transaction confirmed, signature: {signature}");

    Ok(())
}
