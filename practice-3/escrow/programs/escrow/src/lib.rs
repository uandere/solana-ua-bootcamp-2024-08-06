pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8mtEyFoPPp47tAccHJaa272CEFGczzDQJNqvNCcUUWjo");

#[program]
pub mod escrow {
    use super::*;

    pub fn make_offer(
        context: Context<MakeOffer>,
        id: u64,
        token_a_offered_amount: u64,
        token_b_wanted_amount: u64,
    ) -> Result<()> {
        send_offered_tokens_to_vault(&context, token_a_offered_amount)?;
        save_offer(context, id, token_b_wanted_amount)
    }

    pub fn take_offer(context: Context<TakeOffer>) -> Result<()> {
        send_wanted_tokens_to_maker(&context)?;
        withdraw_and_close_vault(context)
    }

}

