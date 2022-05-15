pub mod utils;
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Transfer, Mint, Token},
};

use utils::{arbitary_amounts, normal_amount_fn};

declare_id!("DQsAkHhyHJhQ5tHjLFqyBh5XHUEWQbD9fgQPkvV9JGZc");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<InitializeAmm>) -> Result<()> {

        if ctx.accounts.amm_account.is_initialized {
            return Err(AmmError::AlreadyInUse.into());
        }

        let (token_authority, _bump_seed) = Pubkey::find_program_address(
            &[&ctx.accounts.amm_account.to_account_info().key.to_bytes()],
            ctx.program_id,
        );

        if *ctx.accounts.authority.key != token_authority {
            return Err(AmmError::InvalidProgramAddress.into());
        }

        if *ctx.accounts.authority.key != ctx.accounts.token_a.owner {
            return Err(AmmError::InvalidOwner.into());
        }

        if *ctx.accounts.authority.key != ctx.accounts.token_b.owner {
            return Err(AmmError::InvalidOwner.into());
        }

        if ctx.accounts.token_a.mint == ctx.accounts.token_b.mint {
            return Err(AmmError::RepeatedMint.into());
        }

        if ctx.accounts.mint_a.supply == 0 {
            return Err(AmmError::InvalidSupply.into());
        }

        if ctx.accounts.mint_b.supply == 0 {
            return Err(AmmError::InvalidSupply.into());
        }

        let amm_account = &mut ctx.accounts.amm_account;
        amm_account.auth_key = *ctx.accounts.signer.key;
        amm_account.token_a = ctx.accounts.token_a.key();
        amm_account.token_b = ctx.accounts.token_b.key();
        let normal_a_amount = normal_amount_fn(ctx.accounts.token_a.amount as f64, ctx.accounts.mint_a.decimals);
        let normal_b_amount = normal_amount_fn(ctx.accounts.token_b.amount as f64, ctx.accounts.mint_b.decimals);
        amm_account.constant = (normal_a_amount * normal_b_amount) as f64;
        amm_account.is_initialized = true;
        amm_account.token_a_decimal = ctx.accounts.mint_a.decimals;
        amm_account.token_b_decimal = ctx.accounts.mint_b.decimals;
        amm_account.mint_a = ctx.accounts.mint_a.key();
        amm_account.mint_b = ctx.accounts.mint_b.key();
        Ok(())
    }

    pub fn swap_transfer_token(ctx: Context<ProxySwapTransferToken>, amount: u64) -> Result<()> {

        if amount <= 0 {
            return Err(AmmError::InvalidSupply.into());
        }

        let (token_authority, bump_seed) = Pubkey::find_program_address(
            &[&ctx.accounts.amm_account.to_account_info().key.to_bytes()],
            ctx.program_id,
        );
        let seeds = &[
            &ctx.accounts.amm_account.to_account_info().key.to_bytes(),
            &[bump_seed][..],
        ];

        if *ctx.accounts.authority.key != token_authority {
            return Err(AmmError::InvalidProgramAddress.into());
        }


        let from = &ctx.accounts.from;
        let token_acc_a = &ctx.accounts.token_acc_a;
        let token_acc_b = &ctx.accounts.token_acc_b;
        let _to = &ctx.accounts.to;

        let normal_a_amount = normal_amount_fn(token_acc_a.amount as f64, ctx.accounts.amm_account.token_a_decimal);
        let normal_b_amount = normal_amount_fn(token_acc_b.amount as f64, ctx.accounts.amm_account.token_b_decimal);

        if from.mint.key() == ctx.accounts.amm_account.mint_a.key() {
            if token_acc_a.mint.key() != ctx.accounts.amm_account.mint_a.key() {
                return Err(AmmError::InvalidMint.into());
            }
            let new_pool_a_acc = (amount as f64) + normal_a_amount;
            let new_pool_b_acc = ctx.accounts.amm_account.constant / new_pool_a_acc as f64;

            let token_a_decimal = ctx.accounts.amm_account.token_a_decimal;
            let token_a_amount = arbitary_amounts(amount as f64, token_a_decimal);
            if (from.amount as f64) < token_a_amount {
                return Err(AmmError::InsufficientBalance.into());
            }
            token::transfer(ctx.accounts.from_singer_token_a_ctx(), token_a_amount as u64)?;
            let token_b_decimal = ctx.accounts.amm_account.token_b_decimal;
            let new_pool_b_decimal = arbitary_amounts(new_pool_b_acc, token_b_decimal);
            let transfer_to_b = (ctx.accounts.token_acc_b.amount as f64) - new_pool_b_decimal;
            token::transfer(
                ctx.accounts.from_authority_token_b_ctx().with_signer(&[&seeds[..]]), 
                transfer_to_b as u64
            )?;
        } else {

            if from.mint.key() != ctx.accounts.amm_account.mint_b.key() {
                return Err(AmmError::InvalidMint.into());
            }

            if token_acc_b.mint.key() != ctx.accounts.amm_account.mint_b.key() {
                return Err(AmmError::InvalidMint.into());
            }

            let new_pool_b_acc = (amount as f64) + normal_b_amount;
            let new_pool_a_acc = ctx.accounts.amm_account.constant / new_pool_b_acc as f64;

            let token_b_decimal = ctx.accounts.amm_account.token_b_decimal;
            let token_b_amount = arbitary_amounts(amount as f64, token_b_decimal);
            if (from.amount as f64) < token_b_amount {
                return Err(AmmError::InsufficientBalance.into());
            }
            token::transfer(ctx.accounts.from_singer_token_b_ctx(), token_b_amount as u64)?;
            let token_a_decimal = ctx.accounts.amm_account.token_a_decimal;
            let new_pool_a_decimal = arbitary_amounts(new_pool_a_acc, token_a_decimal);
            let transfer_to_a = (ctx.accounts.token_acc_a.amount as f64) - new_pool_a_decimal;
            token::transfer(
                ctx.accounts.from_authority_token_a_ctx().with_signer(&[&seeds[..]]), 
                transfer_to_a as u64
            )?;
        }
        Ok(())
    }

    pub fn add_token(ctx: Context<AddToken>, amount: u64) -> Result<()> {
        if amount <= 0 {
            return Err(AmmError::InvalidSupply.into());
        }
        let token_a_amount = ctx.accounts.token_acc_a.amount;
        let token_b_amount = ctx.accounts.token_acc_b.amount;
        let mut normal_a_amount = normal_amount_fn(token_a_amount as f64, ctx.accounts.amm_account.token_a_decimal);
        let mut normal_b_amount = normal_amount_fn(token_b_amount as f64, ctx.accounts.amm_account.token_b_decimal);

        let decimal: u8;

        if ctx.accounts.amm_account.mint_a.key() == ctx.accounts.from.mint.key() {
            if ctx.accounts.to.mint.key() != ctx.accounts.from.mint.key() {
                return Err(AmmError::InvalidMint.into());
            }
            if ctx.accounts.to.key() != ctx.accounts.amm_account.token_a.key() {
                return Err(AmmError::InvalidTokenAccount.into());
            }
            if ctx.accounts.token_acc_a.key() != ctx.accounts.amm_account.token_a.key() {
                return Err(AmmError::InvalidTokenAccount.into());
            }
            normal_a_amount += amount as f64;
            decimal = ctx.accounts.amm_account.token_a_decimal;
            msg!("Trasnfer to token account A");
        } else {
            if ctx.accounts.amm_account.mint_b.key() != ctx.accounts.from.mint.key() {
                return Err(AmmError::InvalidMint.into());
            }
            if ctx.accounts.to.mint.key() != ctx.accounts.from.mint.key() {
                return Err(AmmError::InvalidMint.into());
            }
            if ctx.accounts.to.key() != ctx.accounts.amm_account.token_b.key() {
                return Err(AmmError::InvalidTokenAccount.into());
            }
            if ctx.accounts.token_acc_b.key() != ctx.accounts.amm_account.token_b.key() {
                return Err(AmmError::InvalidTokenAccount.into());
            }
            normal_b_amount += amount as f64;
            decimal = ctx.accounts.amm_account.token_b_decimal;
            msg!("Trasnfer to token accoun B");
        }
        let amm_acc = &mut ctx.accounts.amm_account;
        amm_acc.constant = normal_a_amount * normal_b_amount;
        let new_amount = arbitary_amounts(amount  as f64, decimal);
        token::transfer(ctx.accounts.add_token_ctx(), new_amount as u64)
    }
}

#[derive(Accounts)]
pub struct InitializeAmm<'info> {
    // space =32 for auth key + 32 token A + 32 token B + 8 ratio
    //  + 1 is initialized + 1 token A decimal + 1 token B decimal +32 mint a+ 32 mint b
    #[account(init, payer = signer, space = 32 + 32 + 32 + 32 + 32 + 32 + 8 + 1 + 1 + 1)]
    amm_account: Account<'info, Amm>,
    #[account(mut)]
    signer: Signer<'info>,
    mint_a: Account<'info, Mint>,
    mint_b: Account<'info, Mint>,
    #[account(mut)]
    pub token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b: Account<'info, TokenAccount>,
    system_program: Program<'info, System>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    pub authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ProxySwapTransferToken<'info> {
    amm_account: Account<'info, Amm>,
    #[account(mut)]
    signer: Signer<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    pub authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_acc_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_acc_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ProxySwapTransferToken<'info> {
    fn from_authority_token_a_ctx(&self) -> CpiContext<'_, '_, '_, 'info,  Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.token_acc_a.to_account_info(),
            to: self.to.to_account_info(),
            authority: self.authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    fn from_singer_token_a_ctx(&self) -> CpiContext<'_, '_, '_, 'info,  Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.from.to_account_info(),
            to: self.token_acc_a.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }


    fn from_authority_token_b_ctx(&self) -> CpiContext<'_, '_, '_, 'info,  Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.token_acc_b.to_account_info(),
            to: self.to.to_account_info(),
            authority: self.authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    fn from_singer_token_b_ctx(&self) -> CpiContext<'_, '_, '_, 'info,  Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.from.to_account_info(),
            to: self.token_acc_b.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct AddToken<'info> {
    #[account(mut, has_one = auth_key @ AmmError::InvalidSigner)]
    amm_account: Account<'info, Amm>,
    pub auth_key: Signer<'info>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub token_acc_a: Account<'info, TokenAccount>,
    pub token_acc_b: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> AddToken<'info> {
    fn add_token_ctx(&self) -> CpiContext<'_, '_, '_, 'info,  Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.from.to_account_info(),
            to: self.to.to_account_info(),
            authority: self.auth_key.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[account] 
pub struct Amm {
    pub auth_key: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub token_a_decimal: u8,
    pub token_b_decimal: u8,
    pub constant: f64,
    pub is_initialized: bool,
}

#[error_code]
pub enum AmmError {
    //0
    #[msg("AMM account already in use")]
    AlreadyInUse,
    #[msg("Invalid program address generated from bump seed and key")]
    InvalidProgramAddress,
    #[msg("Input account owner is not the program address")]
    InvalidOwner,
    #[msg("Input token account empty")]
    EmptySupply,
    #[msg("InvalidInput")]
    InvalidInput,
    //5
    #[msg("Address of the provided swap token account is incorrect")]
    IncorrectSwapAccount,
    #[msg("Swap input token accounts have the same mint")]
    RepeatedMint,
    #[msg("Mint has a zero supply")]
    InvalidSupply,
    #[msg("Expected amount is more than actual amount")]
    InvalidAmount,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Amout should be greater than 0")]
    InvalidAmmount,
    #[msg("Invalid Mint adderess")]
    InvalidMint,
    #[msg("You are not a contract owner")]
    InvalidSigner,
    #[msg("Token Account is invalid")]
    InvalidTokenAccount
}