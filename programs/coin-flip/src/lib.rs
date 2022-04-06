use anchor_lang::{
    prelude::*,
    solana_program::sysvar::{
        recent_blockhashes,
        slot_hashes,
    },
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, TokenAccount, Burn, Mint, MintTo, Token, Transfer},
};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

declare_id!("AS68LEJddL4UnwJJG5qEkGJihLEbdqxKj6YUdpLhBaED");

pub const CORE_STATE_SEED: &str = "core-state";
pub const VAULT_AUTH_SEED: &str = "vault-auth";
pub const VAULT_TOKEN_ACCOUNT_SEED: &str = "vault-token-account";
pub const BET_STATE_SEED: &str = "bet-state";

pub mod utils;

#[program]
pub mod coin_flip {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        ctx.accounts.core_state.admin = ctx.accounts.admin.key();
        ctx.accounts.core_state.core_state_nonce = args.core_state_nonce;
        ctx.accounts.core_state.vault_auth_nonce = args.vault_auth_nonce;
        ctx.accounts.core_state.flip_counter = 0;
        ctx.accounts.core_state.fee_percent = args.fee_percent;
        Ok(())
    }

    pub fn register(_ctx: Context<Register>, _args: RegisterArgs) -> Result<()> {
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

        let vault_auth_seeds = [
            VAULT_AUTH_SEED.as_bytes(),
            core_state.admin.as_ref(),
            &[core_state.vault_auth_nonce],
        ];

        if !is_native {
            utils::assert_is_ata(&admin_token_account, &admin.key(), &token_mint.key())?;
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
        ctx.accounts.core_state.flip_counter += 1;

        let core_state = &ctx.accounts.core_state;
        let user = &ctx.accounts.user;
        let vault_authority = &ctx.accounts.vault_authority;
        let token_mint = &ctx.accounts.token_mint;
        let user_token_account = &ctx.accounts.user_token_account;
        let vault_token_account = &ctx.accounts.vault_token_account;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;

        let is_native = token_mint.key() == spl_token::native_mint::id();

        if !is_native {
            utils::assert_is_ata(&user_token_account, &user.key(), &token_mint.key())?;
            anchor_lang::solana_program::program::invoke(
                &spl_token::instruction::transfer(
                    &token_program.key(),
                    &user_token_account.key(),
                    &vault_token_account.key(),
                    &user.key(),
                    &[],
                    args.amount,
                )?,
                &[
                    vault_token_account.to_account_info(),
                    user_token_account.to_account_info(),
                    token_program.to_account_info(),
                    user.to_account_info(),
                ],
            )?;
        } else {
            utils::assert_keys_equal(user_token_account.key(), user.key())?;
            utils::assert_keys_equal(vault_token_account.key(), vault_authority.key())?;
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    &user_token_account.key(),
                    &vault_token_account.key(),
                    args.amount,
                ),
                &[
                    vault_token_account.to_account_info(),
                    user_token_account.to_account_info(),
                    system_program.to_account_info(),
                ],
            )?;
        }

        ctx.accounts.bet_state.core_state = core_state.key();
        ctx.accounts.bet_state.bet_state_nonce = args.bet_state_nonce;
        ctx.accounts.bet_state.user = user.key();
        ctx.accounts.bet_state.token_mint = token_mint.key();
        ctx.accounts.bet_state.amount = args.amount;
        ctx.accounts.bet_state.bet_side = args.bet_side;
        ctx.accounts.bet_state.flip_counter = args.flip_counter;
        ctx.accounts.bet_state.approved = true;

        Ok(())
    }

    pub fn bet_return(ctx: Context<BetReturn>) -> Result<()> {
        ctx.accounts.bet_state.approved = false;

        let core_state = &ctx.accounts.core_state;
        let bet_state = &ctx.accounts.bet_state;
        let fee = bet_state.amount * (core_state.fee_percent as u64) / 100;
        let user = &ctx.accounts.user;
        let vault_authority = &ctx.accounts.vault_authority;
        let token_mint = &ctx.accounts.token_mint;
        let user_token_account = &ctx.accounts.user_token_account;
        let vault_token_account = &ctx.accounts.vault_token_account;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;

        let clock = (Clock::get()?).unix_timestamp as u64;

        let is_native = token_mint.key() == spl_token::native_mint::id();

        let mut hasher = DefaultHasher::new();

        let block_hash = recent_blockhashes::id();
        let slot_hash = slot_hashes::id();
        [block_hash, slot_hash].hash(&mut hasher);
        let hash0 = hasher.finish();
        let mut hasher = DefaultHasher::new();
        
        [hash0, clock, core_state.flip_counter as u64].hash(&mut hasher);
        let hash = hasher.finish();

        if (hash % 2 == 0) ^ bet_state.bet_side {
            let vault_auth_seeds = [
                VAULT_AUTH_SEED.as_bytes(),
                core_state.admin.as_ref(),
                &[core_state.vault_auth_nonce],
            ];
    
            if !is_native {
                utils::assert_is_ata(&user_token_account, &user.key(), &token_mint.key())?;
                anchor_lang::solana_program::program::invoke_signed(
                    &spl_token::instruction::transfer(
                        &token_program.key(),
                        &vault_token_account.key(),
                        &user_token_account.key(),
                        &vault_authority.key(),
                        &[],
                        2 * bet_state.amount - fee,
                    )?,
                    &[
                        vault_token_account.to_account_info(),
                        user_token_account.to_account_info(),
                        token_program.to_account_info(),
                        vault_authority.to_account_info(),
                    ],
                    &[&vault_auth_seeds],
                )?;
            } else {
                utils::assert_keys_equal(user_token_account.key(), user.key())?;
                utils::assert_keys_equal(vault_token_account.key(), vault_authority.key())?;
                anchor_lang::solana_program::program::invoke_signed(
                    &anchor_lang::solana_program::system_instruction::transfer(
                        &vault_token_account.key(),
                        &user_token_account.key(),
                        2 * bet_state.amount - fee,
                    ),
                    &[
                        vault_token_account.to_account_info(),
                        user_token_account.to_account_info(),
                        system_program.to_account_info(),
                        user.to_account_info(),
                    ],
                    &[&vault_auth_seeds],
                )?;
            }
            msg!("Congratulations, You won!");
        }
        else {
            msg!("Sorry, You lost!");
        }
        
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
        space = 8 + 1 + 1 + 8 + 1 + std::mem::size_of::<Pubkey>(),
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
        mut,
        seeds = [CORE_STATE_SEED.as_bytes(), core_state.admin.as_ref()],
        bump = core_state.core_state_nonce,
    )]
    pub core_state: Box<Account<'info, CoreState>>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes(), core_state.admin.as_ref()],
        bump = core_state.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub token_mint: Account<'info, Mint>,
    /// CHECK:
    #[account(mut)]
    pub user_token_account: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,
    #[account(
        init,
        space = 8 + 1 + 8 + 1 + 8 + 1 + 3 * std::mem::size_of::<Pubkey>(),
        seeds = [BET_STATE_SEED.as_bytes(), core_state.admin.as_ref(), &args.flip_counter.to_le_bytes()],
        bump,
        payer = user,
    )]
    pub bet_state: Box<Account<'info, BetState>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BetReturn<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [CORE_STATE_SEED.as_bytes(), core_state.admin.as_ref()],
        bump = core_state.core_state_nonce,
    )]
    pub core_state: Box<Account<'info, CoreState>>,
    /// CHECK:
    #[account(mut)]
    pub user: AccountInfo<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [VAULT_AUTH_SEED.as_bytes(), core_state.admin.as_ref()],
        bump = core_state.vault_auth_nonce,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub token_mint: Account<'info, Mint>,
    /// CHECK:
    #[account(mut)]
    pub user_token_account: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = bet_state.approved @ ErrorCode::UnapprovedBet,
        constraint = bet_state.core_state == core_state.key() @ ErrorCode::InvalidCoreState,
        constraint = bet_state.token_mint == token_mint.key() @ ErrorCode::InvalidTokenMint,
        seeds = [BET_STATE_SEED.as_bytes(), core_state.admin.as_ref(), &bet_state.flip_counter.to_le_bytes()],
        bump = bet_state.bet_state_nonce,
    )]
    pub bet_state: Box<Account<'info, BetState>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// -------------------------------------------------------------------------------- //
// ------------------------------------- Args ------------------------------------- //
// -------------------------------------------------------------------------------- //

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    pub core_state_nonce: u8,
    pub vault_auth_nonce: u8,
    pub fee_percent: u8,
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
    pub amount: u64,
    pub bet_side: bool, // true = Head, false = Tail
    pub flip_counter: u64,
    pub bet_state_nonce: u8,
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
    pub fee_percent: u8,
}

#[account]
#[derive(Default)]
pub struct BetState {
    pub bet_state_nonce: u8,
    pub core_state: Pubkey,
    pub user: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub bet_side: bool, // true = Head, false = Tail
    pub flip_counter: u64,
    pub approved: bool, // originally false. set true after transfer
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
    #[msg("Unapproved Bet")]
    UnapprovedBet,
    #[msg("Invalid CoreState")]
    InvalidCoreState,
    #[msg("Invalid TokenMint")]
    InvalidTokenMint,
}
