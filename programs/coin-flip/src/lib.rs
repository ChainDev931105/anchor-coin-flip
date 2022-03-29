use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Burn, Mint, MintTo, Token, Transfer};

declare_id!("D62yCVSQs6cerJJrfhnpSRVRUVk6HQU72YLmfXjWTzGC");

pub const CORE_STATE_SEED: &str = "core-state";
pub const HOUSE_SEED: &str = "house";

#[program]
pub mod coin_flip {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> ProgramResult {
        ctx.accounts.core_state.admin = ctx.accounts.admin.key();
        ctx.accounts.core_state.core_state_nonce = args.core_state_nonce;
        ctx.accounts.core_state.flip_counter = 0;
        Ok(())
    }

    // pub fn register_spl(ctx: Context<RegisterSpl>, args: RegisterSplArgs) -> ProgramResult {
    //     Ok(())
    // }

    pub fn bet_spl(ctx: Context<BetSpl>, args: BetSplArgs) -> ProgramResult {
        Ok(())
    }

    // pub fn bet_sol(ctx: Context<BetSol>, args: BetSolArgs) -> ProgramResult {
    //     Ok(())
    // }
}

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        seeds = [CORE_STATE_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.core_state_nonce,
        payer = admin,
    )]
    pub core_state: Account<'info, CoreState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: BetSplArgs)]
pub struct BetSpl<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    pub admin: AccountInfo<'info>,
    #[account(
        seeds = [CORE_STATE_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.core_state_nonce,
    )]
    pub core_state: Account<'info, CoreState>,
    #[account(
        constraint = token_mint.key() == args.token_mint @ ErrorCode::TokenMintMismatch,
    )]
    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = user_token_account.owner == user_authority.key() @ ErrorCode::TokenOnwerMismatch,
        constraint = user_token_account.mint == token_mint.key() @ ErrorCode::TokenOnwerMismatch,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    pub core_state_nonce: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BetSplArgs {
    pub amount: u64,
    pub core_state_nonce: u8,
    pub bet_side: bool, // true = Head, false = Tail
    pub token_mint: Pubkey,
}

#[account]
#[derive(Default)]
pub struct CoreState {
    pub core_state_nonce: u8,
    pub admin: Pubkey, // admin public key
    pub flip_counter: u8,
}

#[error]
pub enum ErrorCode {
    #[msg("Wrong Admin Address")]
    WrongAdmin,
    #[msg("Token Onwer Mismatch")]
    TokenOnwerMismatch,
    #[msg("Token Mint Mismatch")]
    TokenMintMismatch,
}
