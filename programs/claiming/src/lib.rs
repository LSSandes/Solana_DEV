use anchor_lang::prelude::*;

use solana_program::{
    program_pack::{Pack, Sealed},
    system_instruction,
};
use anchor_spl::token::{self, TokenAccount, Transfer, Token};
use merkle_tree: MerkleTree;
use std::convert::Into;
use sha2::{Sha256, Digest};

declare_id!("PROGRAM_ID");

#[program]
pub mod claiming_system  {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, _bump: u8, sol_merkle_root: [u8; 32], spl_merkle_root: [u8, 32]) -> Result<()> {
        let claim_period = &mut ctx.accounts.claim_period;
        claim_period.start_date = Clock::get()?.unix_timestamp as u64;
        claim_period.end_date = claim_period.start_date + 90 * 24 * 60 * 60; // 90 days

        claim_period.sol_merkle_root = sol_merkle_root;
        claim_period.spl_merkle_root = spl_merkle_root;
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, 
                 user_id: String, 
                 claim_id: String, 
                 sol_amount: u64, 
                 spl_amount: u64,
                 sol_proof: Vec<[u8; 32]>,
                 spl_proof: Vec<[u8; 32]>
                ) -> Result<()> {
        let user_claim = &mut ctx.accounts.user_claim;
        
        // Check if current time is within claim period
        let current_time = Clock::get()?.unix_timestamp as u64;
        let claim_period = &ctx.accounts.claim_period;
        require!(current_time >= claim_period.start_date && current_time <= claim_period.end_date, ErrorCode:: ClaimPeriodNotActive);

        //Check if claim has already been made
        require!(!user_claim.is_claimed, ErrorCode::AlreadyClaimed);
        
        let sol_leaf = hash_claim_entry(&user_id, &claim_id, sol_amount);
        let spl_leaf = hash_claim_entry(&user_id, &claim_id, spl_amount);

        require!(
            merkle_tree::verify_proof(&claim_period.sol_merkle_root, &sol_leaf, &sol_proof),
            ErrorCode:: InvalidMerkleProof
        );

        require!(
            merkle_tree::verify_proof(&claim_period.spl_merkle_root, &spl_leaf, &spl_proof),
            ErrorCode: InvalidMerkleProof
        )
        //Solana coin claiming
        if sol_amount > 0 {
            // Used invoke function for transferring native SOL
            invoke(
                &solana_program::system_instruction::transfer(
                    &ctx.accounts.claim_authority.key,
                    &ctx.accounts.user_wallet.key,
                    sol_amount
                ),
                &[
                    ctx.accounts.claim_authority.to_account_info(),
                    ctx.accounts.user_wallet.to_account_info(),
                    ctx.accounts.system_program.to_account_info()
                ]
            )?;
            // let cpi_accounts = Transfer {
            //     from: ctx.accounts.sol_account.to_account_info(),
            //     to: ctx.accounts.user_wallet.to_account_info(),
            //     authority: ctx.accounts.claim_authority.to_account_info()
            // };
            // let cpi_program = ctx.accounts.token_program.clone();
            // token::transfer(
            //     CpiContext::new(cpi_program, cpi_accounts), 
            //     sol_amount
            // )?;
        }

        //SPL token claiming
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
    
    //Reclaims unclaimed SOL and SPL tokens back to a designated wallet after the claim period ends
    pub fn reclaim_unclaimed(ctx: Context<ReclaimUnclaimed>) -> Result<()> {
        let claim_period = &ctx.accounts.claim_period;
        require!(Clock::get()?.unix_timestamp as u64 > claim_period.end_date, ErrorCode::ClaimPeriodNotEnded);

        let cpi_accounts_sol = Transfer {
            from: ctx.accounts.sol_account.to_account_info(),
            to: ctx.accounts.designated_wallet.to_account_info(),
            authority: ctx.accounts.claim_authority.to_account_info(),
        };

        let cpi_accounts_spl = Transfer {
            from: ctx.accounts.spl_account.to_account_info(),
            to: ctx.accounts.designated_wallet.to_account_info(),
            authority: ctx.accounts.claim_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.clone();

        token::transfer(CpiContext::new(cpi_program.clone(), cpi_accounts_sol), ctx.accounts.sol_account.amount)?;
        token::transfer(CpiContext::new(cpi_program, cpi_accounts_spl), ctx.accounts.spl_account.amount)?;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    //8 bytes (for the account discriminator) + 16 bytes (8 bytes for start_date + 8 bytes for end_date)
    #[account(init, payer = user, space = 8 + 8*2)]
    pub claim_period: Account<'info, ClaimPeriod>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user_claim: Account<'info, UserClaim>,
    #[account(mut)]
    pub sol_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub spl_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_wallet: Account<'info, TokenAccount>,
    #[account(signer)]
    pub claim_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub claim_period: Account<'info, ClaimPeriod>
}

#[derive(Accounts)]
pub struct ReclaimUnclaimed<'info> {
    pub claim_period: Account<'info, ClaimPeriod>,
    #[account(mut)]
    pub sol_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub spl_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub designated_wallet: Account<'info, TokenAccount>,
    #[account(signer)]
    pub claim_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct ClaimPeriod {
    pub start_date: u64,
    pub end_date: u64,

    pub sol_merkle_root: [u8; 32];
    pub spl_merkle_root: [u8; 32];
}

#[account]
pub struct UserClaim {
    pub user_id: String,
    pub claim_id: String,
    pub is_claimed: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The claim has already been made.")]
    AlreadyClaimed,
    #[msg("The claiming period has not ended yet.")]
    ClaimPeriodNotEnded,
    #[msg("The claim period is not active.")]
    ClaimPeriodNotActive,
    #[msg("The provided Merkle proof is invalid.")]
    InvalidMerkleProof,
}

fn hash_claim_entry(user_id: &String, claim_id: &String, amount: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();

    hasher.update(user_id.as_bytes());
    hasher.update(claim_id.as_bytes());
    hasher.update(amount.to_le_bytes());
    hasher.finalize().into();
}