use anchor_lang::prelude::*;

use solana_program::program_pack::{Pack, Sealed};
use anchor_spl::token::{self, TokenAccount, Transfer};
use std::convert::Into;

declare_id!("PROGRAM_ID");

#[program]
pub mod claiming_system  {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, _bump: u8) -> Result<()> {
        let claim_period = &mut ctx.accounts.claim_period;
        claim_period.start_date = Clock::get()?.unix_timestamp as u64;
        claim_period.end_date = claim_period.start_date + 90 * 24 * 60 * 60; // 90 days
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, user_id: String, claim_id: String, sol_amount: u64, spl_amount: u64) -> Result<()> {
        let user_claim = &mut ctx.accounts.user_claim;
        require!(!user_claim.is_claimed, ErrorCode::AlreadyClaimed);
        
        if sol_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.sol_account.to_account_info(),
                to: ctx.accounts.user_wallet.to_account_info(),
                authority: ctx.accounts.claim_authority.to_account_info()
            };
            let cpi_program = ctx.accounts.token_program.clone();
            token::transfer(
                CpiContext::new(cpi_program, cpi_accounts), 
                sol_amount
            )?;
        }
        
        if spl_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.spl_account.to_account_info(),
                to: ctx.accounts.user_wallet.to_account_info(),
                authority: ctx.accounts.claim_authority.to_account_info()
            };
            let cpi_program = ctx.accounts.token_program.clone();
            token::transfer(
                CpiContext::new(cpi_program, cpi_accounts), 
                spl_amount
            )?;
        }

        user_claim.is_claimed = true;
        Ok(())
    }

    

}

#[derive(Accounts)]
pub struct Initialize {}
