#![allow(unused_imports)]
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount, set_authority, SetAuthority},
};


use std::mem::size_of;

declare_id!("4BsMpWZvHvJKRuTzkRX9ocNj3BxkcGZZnYw8miiWyUoz");

const DISCRIMINATOR_SIZE: usize = 8;


#[program]
pub mod solana_nft_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, total_supply: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.owner = ctx.accounts.signer.key();
        state.total_supply = total_supply;
        state.nfts_minted = 0;
        Ok(())
    }

    pub fn init_nft(ctx: Context<InitNFT>, num_nfts: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        
        // Check that only the program's owner can mint
        require!(ctx.accounts.signer.key() == state.owner, CustomError::Unauthorized);

        // Ensure the total supply isn't exceeded
        require!(state.nfts_minted + num_nfts <= state.total_supply, CustomError::SupplyExceeded);

        // Mint the NFTs
        for _ in 0..num_nfts {
            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.associated_token_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            );

            mint_to(cpi_context, 1)?;
            state.nfts_minted += 1;
        }

        // TODO: close the mint account
        // Disable future minting by setting the mint authority to None
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: ctx.accounts.mint.to_account_info(),
                current_authority: ctx.accounts.signer.to_account_info(),
            },
        );

        set_authority(
            cpi_context,
            spl_token::instruction::AuthorityType::MintTokens,
            None
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = NFTState::SIZE
    )]
    pub state: Account<'info, NFTState>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitNFT<'info> {
    #[account(mut)]
    pub state: Account<'info, NFTState>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = signer.key(),
        mint::freeze_authority = signer.key(),
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct NFTState {
    pub owner: Pubkey,       // Owner of the program
    pub total_supply: u64,   // Fixed supply of NFTs
    pub nfts_minted: u64,    // Number of NFTs minted so far
}

impl NFTState {
    const SIZE: usize = DISCRIMINATOR_SIZE + size_of::<Pubkey>() + size_of::<u64>() * 2;
}

#[error_code]
pub enum CustomError {
    #[msg("You are not authorized to mint NFTs")]
    Unauthorized,
    #[msg("Minting this many NFTs would exceed the total supply")]
    SupplyExceeded,
}