use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Burn, Mint, MintTo, Token, Transfer};

declare_id!("5G2vmwuHzznDrRQYHsK4FfXJscPMHUSqZvouCHa9SnQ7");

pub const CORE_STATE_SEED: &str = "core-state";
pub const VAULT_AUTH_SEED: &str = "vault-auth";

#[program]
pub mod coin_flip {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        ctx.accounts.core_state.admin = ctx.accounts.admin.key();
        ctx.accounts.core_state.core_state_nonce = args.core_state_nonce;
        ctx.accounts.core_state.flip_counter = 0;
        Ok(())
    }

    pub fn deposit_sol(ctx: Context<DepositSol>, args: DepositSolArgs) -> Result<()> {
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.admin.key(),
            &ctx.accounts.vault_authority.key(),
            args.amount,
        );
        let _result = anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.vault_authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        );

        Ok(())
    }

    pub fn withdraw_sol(ctx: Context<WithdrawSol>, args: WithdrawSolArgs) -> Result<()> {
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.vault_authority.key(),
            &ctx.accounts.admin.key(),
            args.amount,
        );
        let _result = anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.vault_authority.to_account_info(),
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        );

        Ok(())
    }

    pub fn register_spl(ctx: Context<RegisterSpl>, args: RegisterSplArgs) -> Result<()> {
        Ok(())
    }

    pub fn deposit_spl(ctx: Context<DepositSpl>, args: DepositSplArgs) -> Result<()> {
        Ok(())
    }

    pub fn withdraw_spl(ctx: Context<WithdrawSpl>, args: WithdrawSplArgs) -> Result<()> {
        Ok(())
    }

    pub fn bet_sol(ctx: Context<BetSol>, args: BetSolArgs) -> Result<()> {
        Ok(())
    }

    pub fn bet_spl(ctx: Context<BetSpl>, args: BetSplArgs) -> Result<()> {
        Ok(())
    }
}

// -------------------------------------------------------------------------------- //
// ----------------------------------- Contexts ----------------------------------- //
// -------------------------------------------------------------------------------- //

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        space = 1 + 8 + 1 + std::mem::size_of::<Pubkey>(),
        seeds = [CORE_STATE_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump,
        payer = admin,
    )]
    pub core_state: Account<'info, CoreState>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>, 
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: DepositSolArgs)]
pub struct DepositSol<'info> {
    #[account(
        mut,
        constraint = admin.lamports() >= args.amount @ ErrorCode::InsufficientFunds,
    )]
    pub admin: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: WithdrawSolArgs)]
pub struct WithdrawSol<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: RegisterSplArgs)]
pub struct RegisterSpl<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(args: DepositSplArgs)]
pub struct DepositSpl<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(args: WithdrawSplArgs)]
pub struct WithdrawSpl<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(args: BetSolArgs)]
pub struct BetSol<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub admin: AccountInfo<'info>,
    #[account(
        seeds = [CORE_STATE_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.core_state_nonce,
    )]
    pub core_state: Account<'info, CoreState>,
    #[account(mut)]
    pub user_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(args: BetSplArgs)]
pub struct BetSpl<'info> {
    #[account(mut)]
    pub user_authority: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
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

// -------------------------------------------------------------------------------- //
// ------------------------------------- Args ------------------------------------- //
// -------------------------------------------------------------------------------- //

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositSolArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawSolArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterSplArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositSplArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawSplArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BetSolArgs {
    pub core_state_nonce: u8,
    pub amount: u64,
    pub bet_side: bool, // true = Head, false = Tail
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BetSplArgs {
    pub core_state_nonce: u8,
    pub amount: u64,
    pub bet_side: bool, // true = Head, false = Tail
    pub token_mint: Pubkey,
}

// -------------------------------------------------------------------------------- //
// ------------------------------------ Others ------------------------------------ //
// -------------------------------------------------------------------------------- //

#[account]
#[derive(Default)]
pub struct CoreState {
    pub core_state_nonce: u8,
    pub admin: Pubkey, // admin public key
    pub flip_counter: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Wrong Admin Address")]
    WrongAdmin,
    #[msg("Token Onwer Mismatch")]
    TokenOnwerMismatch,
    #[msg("Token Mint Mismatch")]
    TokenMintMismatch,
    #[msg("Insufficient Funds")]
    InsufficientFunds,
}
