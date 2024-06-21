use anchor_lang::prelude::*;

declare_id!("9PhcDUK9usxXFnKonQbRCHhVdZY9YAAKxoAMh7wBjeeW");

#[program]
pub mod claiming {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
