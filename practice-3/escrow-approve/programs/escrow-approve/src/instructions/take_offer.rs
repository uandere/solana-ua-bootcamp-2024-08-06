use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        Mint, TokenAccount, TokenInterface, transfer_checked,
        TransferChecked, approve, Approve
    },
};

use crate::Offer;

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(mut)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        has_one = token_mint_b,
    )]
    pub offer: Account<'info, Offer>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn delegate_needed_tokens_to_program(context: &Context<TakeOffer>) -> Result<()> {
    let binding = context.accounts.maker.key();

    let seeds: &[&[u8]] = &[
        b"offer",
        binding.as_ref(),
        &context.accounts.offer.id.to_le_bytes(),
        &[context.accounts.offer.bump],
    ];

    let signer_seeds = &[&seeds[..]];

    let ctx = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        Approve {
            to: context.accounts.taker_token_account_b.to_account_info(),
            delegate: context.accounts.offer.to_account_info(),
            authority: context.accounts.taker.to_account_info(),
        },
        signer_seeds,
    );

    approve(
        ctx,
        context.accounts.offer.token_b_wanted_amount,
    )
}


pub fn resolve_offer(context: &Context<TakeOffer>) -> Result<()> {

    let binding = context.accounts.maker.key();

    let seeds: &[&[u8]] = &[
        b"offer",
        binding.as_ref(),
        &context.accounts.offer.id.to_le_bytes(),
        &[context.accounts.offer.bump],
    ];

    let signer_seeds = &[&seeds[..]];

    // Sending token B to maker
    let transfer_accounts_a = TransferChecked {
        from: context.accounts.taker_token_account_b.to_account_info(),
        mint: context.accounts.token_mint_b.to_account_info(),
        to: context.accounts.maker_token_account_b.to_account_info(),
        authority: context.accounts.offer.to_account_info(),
    };

    let cpi_ctx_a = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        transfer_accounts_a,
        signer_seeds,
    );

    transfer_checked(
        cpi_ctx_a,
        context.accounts.offer.token_b_wanted_amount,
        context.accounts.token_mint_b.decimals,
    )?;

    // Sending token A to taker
    let transfer_accounts_b = TransferChecked {
        from: context.accounts.maker_token_account_a.to_account_info(),
        mint: context.accounts.token_mint_a.to_account_info(),
        to: context.accounts.taker_token_account_a.to_account_info(),
        authority: context.accounts.offer.to_account_info(),
    };

    let cpi_ctx_b = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        transfer_accounts_b,
        signer_seeds,
    );

    transfer_checked(
        cpi_ctx_b,
        context.accounts.offer.token_a_offered_amount,
        context.accounts.token_mint_a.decimals
    )
}
