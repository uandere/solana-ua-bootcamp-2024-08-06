use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

declare_id!("C1cUvDnDKvN64HwAJp7Awfrb2LMiLQZywqfShFF73XcN");

#[program]
pub mod escrow_approve {
    use super::*;

    pub fn make_offer(
        ctx: Context<
            MakeOffer>,
        id: u64,
        token_a_offered_amount: u64,
        token_b_wanted_amount: u64,
    ) -> Result<()> {
        delegate_offered_tokens_to_program(
            ctx,
            id,
            token_a_offered_amount,
            token_b_wanted_amount,
        )
    }

    pub fn take_offer(
        ctx: Context<TakeOffer>,
    ) -> Result<()> {
        delegate_needed_tokens_to_program(&ctx)?;
        resolve_offer(&ctx)
    }
}
