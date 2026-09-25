use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct Vaultinit<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub staked_mint: InterfaceAccount<'info, Mint>,

    pub reward_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [b"vault", admin.key().as_ref()],
        bump,
        space = 8 + Vault::INIT_SPACE
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = staked_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub staked_mint_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = reward_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub reward_mint_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(address = vault.staked_mint)]
    pub staked_mint: InterfaceAccount<'info, Mint>,

    ///CHECK: unchecked account used only to derive pda
    pub admin: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = staked_mint,
    )]
    pub user_stake_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds= [b"vault", admin.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        associated_token::authority = vault,
        associated_token::mint = staked_mint
    )]
    pub vault_stake_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"vault",user.key().as_ref(),vault.key().as_ref()],
        bump,
        space = 8 + UserStakeProfile::INIT_SPACE,
    )]
    pub user_stake_profile: Account<'info, UserStakeProfile>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(constraint = staked_mint.key()== vault.staked_mint)]
    pub staked_mint: InterfaceAccount<'info, Mint>,

    #[account(constraint = reward_mint.key() == vault.reward_mint)]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    ///CHECK only used for deriving pda
    pub admin: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault", admin.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = vault
    )]
    pub vault_reward_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref(),vault.key().as_ref()],
        bump
    )]
    pub user_stake_profile: Account<'info, UserStakeProfile>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user,
        associated_token::mint = reward_mint,
    )]
    pub user_reward_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    ///CHECK: inly used for pdas
    #[account(constraint = admin.key()== vault.admin)]
    pub admin: UncheckedAccount<'info>,

    #[account(
        constraint = staked_mint.key() == vault.staked_mint
    )]
    pub staked_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", admin.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        associated_token::authority = vault,
        associated_token::mint = staked_mint
    )]
    pub vault_staked_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault",user.key().as_ref(),  vault.key().as_ref()],
        bump,
        constraint = user_profile.authority == user.key()
    )]
    pub user_profile: Account<'info, UserStakeProfile>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = staked_mint
    )]
    pub user_staked_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub admin: Pubkey,
    pub staked_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub total_staked: u64,
    pub last_updated: i64,
    pub reward_rate: u64,
    pub reward_per_token_stored: u128,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserStakeProfile {
    pub authority: Pubkey,
    pub staked_amount: u64,
    pub last_updated: i64,
    pub reward_per_token_paid: u128,
    pub rewards_earned: u64,
}



