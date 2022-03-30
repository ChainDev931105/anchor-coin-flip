use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, TokenAccount, Burn, Mint, MintTo, Token, Transfer},
};

declare_id!("5G2vmwuHzznDrRQYHsK4FfXJscPMHUSqZvouCHa9SnQ7");

pub const CORE_STATE_SEED: &str = "core-state";
pub const VAULT_AUTH_SEED: &str = "vault-auth";

pub mod utils;

#[program]
pub mod coin_flip {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        ctx.accounts.core_state.admin = ctx.accounts.admin.key();
        ctx.accounts.core_state.core_state_nonce = args.core_state_nonce;
        ctx.accounts.core_state.flip_counter = 0;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, args: DepositArgs) -> Result<()> {
        let is_native = ctx.accounts.token_mint.key() == spl_token::native_mint::id();

        let vault_auth_seeds: &[&[u8]] = &[
            VAULT_AUTH_SEED.as_bytes().as_ref(),
            &[args.vault_auth_nonce],
        ];

        utils::create_program_token_account_if_not_present(
            &ctx.accounts.vault_token_account,
            &ctx.accounts.system_program,
            &ctx.accounts.admin,
            &ctx.accounts.token_program,
            &ctx.accounts.token_mint,
            &ctx.accounts.vault_authority,
            &ctx.accounts.rent,
            vault_auth_seeds,// admin seeds
            &[&[]],// admin seeds
            is_native,
        )?;

        if !is_native {
            anchor_lang::solana_program::program::invoke(
                &spl_token::instruction::transfer(
                    &ctx.accounts.token_program.key(),
                    &ctx.accounts.admin_token_account.key(),
                    &ctx.accounts.vault_token_account.key(),
                    &ctx.accounts.admin.key(),
                    &[],
                    args.amount,
                )?,
                &[
                    ctx.accounts.vault_token_account.to_account_info(),
                    ctx.accounts.admin_token_account.to_account_info(),
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.admin.to_account_info(),
                ],
            )?;
        } else {
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    &ctx.accounts.admin_token_account.key(),
                    &ctx.accounts.vault_token_account.key(),
                    args.amount,
                ),
                &[
                    ctx.accounts.vault_token_account.to_account_info(),
                    ctx.accounts.admin_token_account.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, args: WithdrawArgs) -> Result<()> {
        Ok(())
    }

    pub fn bet(ctx: Context<Bet>, args: BetArgs) -> Result<()> {
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
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: DepositArgs)]
pub struct Deposit<'info> {
    #[account(
        seeds = [CORE_STATE_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.core_state_nonce,
    )]
    pub core_state: Account<'info, CoreState>,
    #[account(
        mut,
        constraint = admin.key() == core_state.admin @ ErrorCode::WrongAdmin,
    )]
    pub admin: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = admin_token_account.owner == admin.key() @ ErrorCode::TokenOnwerMismatch,
        constraint = admin_token_account.mint == token_mint.key() @ ErrorCode::TokenMintMismatch,
        constraint = admin_token_account.amount >= args.amount @ ErrorCode::InsufficientFunds,
    )]
    pub admin_token_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        // constraint = vault_token_account.owner == vault_authority.key() @ ErrorCode::TokenOnwerMismatch,
        // constraint = vault_token_account.mint == token_mint.key() @ ErrorCode::TokenMintMismatch,
        // constraint = vault_token_account.amount >= args.amount @ ErrorCode::InsufficientFunds,
    )]
    pub vault_token_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(args: WithdrawArgs)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [CORE_STATE_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.core_state_nonce,
    )]
    pub core_state: Account<'info, CoreState>,
    #[account(
        mut,
        constraint = admin.key() == core_state.admin @ ErrorCode::WrongAdmin,
    )]
    pub admin: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = admin_token_account.owner == admin.key() @ ErrorCode::TokenOnwerMismatch,
        constraint = admin_token_account.mint == token_mint.key() @ ErrorCode::TokenOnwerMismatch,
        constraint = admin_token_account.amount >= args.amount @ ErrorCode::InsufficientFunds,
    )]
    pub admin_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: BetArgs)]
pub struct Bet<'info> {
    #[account(
        seeds = [CORE_STATE_SEED.as_bytes().as_ref(), admin.key().as_ref()],
        bump = args.core_state_nonce,
    )]
    pub core_state: Account<'info, CoreState>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = admin.key() == core_state.admin @ ErrorCode::WrongAdmin,
    )]
    pub admin: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    #[account(mut)]
    pub user_authority: Signer<'info>,
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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
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
pub struct DepositArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BetArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
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
