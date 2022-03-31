use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, TokenAccount, Burn, Mint, MintTo, Token, Transfer},
};

declare_id!("5G2vmwuHzznDrRQYHsK4FfXJscPMHUSqZvouCHa9SnQ7");

pub const CORE_STATE_SEED: &str = "core-state";
pub const VAULT_AUTH_SEED: &str = "vault-auth";
pub const VAULT_TOKEN_ACCOUNT_SEED: &str = "vault-token-account";

pub mod utils;

#[program]
pub mod coin_flip {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        ctx.accounts.core_state.admin = ctx.accounts.admin.key();
        ctx.accounts.core_state.core_state_nonce = args.core_state_nonce;
        ctx.accounts.core_state.vault_auth_nonce = args.vault_auth_nonce;
        ctx.accounts.core_state.flip_counter = 0;
        Ok(())
    }

    pub fn register(ctx: Context<Register>, args: RegisterArgs) -> Result<()> {
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, args: DepositArgs) -> Result<()> {
        let admin = &ctx.accounts.admin;
        let vault_authority = &ctx.accounts.vault_authority;
        let token_mint = &ctx.accounts.token_mint;
        let admin_token_account = &ctx.accounts.admin_token_account;
        let vault_token_account = &ctx.accounts.vault_token_account;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;

        let is_native = token_mint.key() == spl_token::native_mint::id();

        if !is_native {
            utils::assert_is_ata(&admin_token_account, &admin.key(), &token_mint.key())?;
            // utils::assert_is_ata(&vault_token_account, &vault_authority.key(), &token_mint.key())?;
            anchor_lang::solana_program::program::invoke(
                &spl_token::instruction::transfer(
                    &token_program.key(),
                    &admin_token_account.key(),
                    &vault_token_account.key(),
                    &admin.key(),
                    &[],
                    args.amount,
                )?,
                &[
                    vault_token_account.to_account_info(),
                    admin_token_account.to_account_info(),
                    token_program.to_account_info(),
                    admin.to_account_info(),
                ],
            )?;
        } else {
            utils::assert_keys_equal(admin_token_account.key(), admin.key())?;
            utils::assert_keys_equal(vault_token_account.key(), vault_authority.key())?;
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    &admin_token_account.key(),
                    &vault_token_account.key(),
                    args.amount,
                ),
                &[
                    vault_token_account.to_account_info(),
                    admin_token_account.to_account_info(),
                    system_program.to_account_info(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, args: WithdrawArgs) -> Result<()> {
        let core_state = &ctx.accounts.core_state;
        let admin = &ctx.accounts.admin;
        let vault_authority = &ctx.accounts.vault_authority;
        let token_mint = &ctx.accounts.token_mint;
        let admin_token_account = &ctx.accounts.admin_token_account;
        let vault_token_account = &ctx.accounts.vault_token_account;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;

        let is_native = token_mint.key() == spl_token::native_mint::id();

        let _vault_auth_seeds = vault_authority! {
            bump = core_state.vault_auth_nonce
        };

        let admin_key = admin.key();

        let vault_auth_seeds = [
            VAULT_AUTH_SEED.as_bytes(),
            admin_key.as_ref(),
            &[core_state.vault_auth_nonce],
        ];

        if !is_native {
            utils::assert_is_ata(&admin_token_account, &admin.key(), &token_mint.key())?;
            // utils::assert_is_ata(&vault_token_account, &vault_authority.key(), &token_mint.key())?;
            anchor_lang::solana_program::program::invoke_signed(
                &spl_token::instruction::transfer(
                    &token_program.key(),
                    &vault_token_account.key(),
                    &admin_token_account.key(),
                    &vault_authority.key(),
                    &[],
                    args.amount,
                )?,
                &[
                    vault_token_account.to_account_info(),
                    admin_token_account.to_account_info(),
                    token_program.to_account_info(),
                    vault_authority.to_account_info(),
                ],
                &[&vault_auth_seeds],
            )?;
        } else {
            utils::assert_keys_equal(admin_token_account.key(), admin.key())?;
            utils::assert_keys_equal(vault_token_account.key(), vault_authority.key())?;
            anchor_lang::solana_program::program::invoke_signed(
                &anchor_lang::solana_program::system_instruction::transfer(
                    &vault_token_account.key(),
                    &admin_token_account.key(),
                    args.amount,
                ),
                &[
                    vault_token_account.to_account_info(),
                    admin_token_account.to_account_info(),
                    system_program.to_account_info(),
                    admin.to_account_info(),
                ],
                &[&vault_auth_seeds],
            )?;
        }

        Ok(())
    }

    pub fn bet(ctx: Context<Bet>, args: BetArgs) -> Result<()> {
        Ok(())
    }
}

#[macro_export]
macro_rules! vault_authority {
    (bump = $bump:expr) => {
        &[VAULT_AUTH_SEED.as_bytes().as_ref(), &[$bump]]
    };
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
        space = 8 + 1 + 1 + 8 + std::mem::size_of::<Pubkey>(),
        seeds = [CORE_STATE_SEED.as_bytes(), admin.key().as_ref()],
        bump,
        payer = admin,
    )]
    pub core_state: Account<'info, CoreState>,
    /// CHECK: 
    #[account(
        seeds = [VAULT_AUTH_SEED.as_bytes(), admin.key().as_ref()],
        bump = args.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: RegisterArgs)]
pub struct Register<'info> {
    #[account(
        seeds = [CORE_STATE_SEED.as_bytes(), admin.key().as_ref()],
        bump = core_state.core_state_nonce,
    )]
    pub core_state: Account<'info, CoreState>,
    #[account(
        mut,
        constraint = admin.key() == core_state.admin @ ErrorCode::WrongAdmin,
    )]
    pub admin: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    /// CHECK:
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes(), admin.key().as_ref()],
        bump = core_state.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    #[account(
        init,
        token::mint = token_mint,
        token::authority = vault_authority,
        seeds = [VAULT_TOKEN_ACCOUNT_SEED.as_bytes(), token_mint.key().as_ref(), admin.key().as_ref()],
        bump,
        payer = admin,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(args: DepositArgs)]
pub struct Deposit<'info> {
    #[account(
        seeds = [CORE_STATE_SEED.as_bytes(), admin.key().as_ref()],
        bump = core_state.core_state_nonce,
    )]
    pub core_state: Account<'info, CoreState>,
    #[account(
        mut,
        constraint = admin.key() == core_state.admin @ ErrorCode::WrongAdmin,
    )]
    pub admin: Signer<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes(), admin.key().as_ref()],
        bump = core_state.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub token_mint: Account<'info, Mint>,
    /// CHECK:
    #[account(mut)]
    pub admin_token_account: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: WithdrawArgs)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [CORE_STATE_SEED.as_bytes(), admin.key().as_ref()],
        bump = core_state.core_state_nonce,
    )]
    pub core_state: Account<'info, CoreState>,
    #[account(
        mut,
        constraint = admin.key() == core_state.admin @ ErrorCode::WrongAdmin,
    )]
    pub admin: Signer<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes(), admin.key().as_ref()],
        bump = core_state.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub token_mint: Account<'info, Mint>,
    /// CHECK:
    #[account(mut)]
    pub admin_token_account: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: BetArgs)]
pub struct Bet<'info> {
    #[account(
        seeds = [CORE_STATE_SEED.as_bytes(), admin.key().as_ref()],
        bump = args.core_state_nonce,
    )]
    pub core_state: Account<'info, CoreState>,
    /// CHECK:
    #[account(
        mut,
        constraint = admin.key() == core_state.admin @ ErrorCode::WrongAdmin,
    )]
    pub admin: AccountInfo<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes(), admin.key().as_ref()],
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
pub struct RegisterArgs {
    pub vault_token_account_nonce: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositArgs {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawArgs {
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
    pub vault_auth_nonce: u8,
    pub admin: Pubkey, // admin public key
    pub flip_counter: u64,
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
    #[msg("Incorrect Owner")]
    IncorrectOwner,
    #[msg("Uninitialized Account")]
    UninitializedAccount,
    #[msg("PublicKey Mismatch")]
    PublicKeyMismatch,
}
