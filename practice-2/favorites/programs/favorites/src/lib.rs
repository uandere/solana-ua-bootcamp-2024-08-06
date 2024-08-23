use anchor_lang::prelude::*;

declare_id!("8uvJrNtywsBSJWHzSQC349kAHujGaX7aN7xieio9fSy7");

#[program]
pub mod favorites {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
