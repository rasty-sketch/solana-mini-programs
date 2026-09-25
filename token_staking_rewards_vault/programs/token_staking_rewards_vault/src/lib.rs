use anchor_lang::prelude::*;
use anchor_spl::token_interface::{transfer_checked, TransferChecked};
mod types;
use types::*;
#[cfg(test)]
mod tests;
const PRECISION: u128 = 1_000_000_000_000;
declare_id!("52hdHGSANrycFt1ku1qy4GWBE9cdKTRyaexTsu17SQ36");

#[program]
pub mod token_staking_rewards_vault {
    use super::*;

    pub fn vault_init(ctx: Context<Vaultinit>, reward_rate: u64) -> Result<()> {
        let data = &mut ctx.accounts.vault;
        data.set_inner(Vault {
            admin: ctx.accounts.admin.key(),
            staked_mint: ctx.accounts.staked_mint.key(),
            reward_mint: ctx.accounts.reward_mint.key(),
            reward_rate,
            reward_per_token_stored: 0,
            bump: ctx.bumps.vault,
            last_updated: 0,
            total_staked: 0,
        });
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, StakingErrors::InvalidAmount);

        let vault_data = &mut ctx.accounts.vault;
        let user_profile = &mut ctx.accounts.user_stake_profile;

        let time_now = Clock::get()?.unix_timestamp;

        if vault_data.total_staked > 0 {
            let time_elapsed: u128 = time_now.checked_sub(vault_data.last_updated).unwrap() as u128;

            // Equation:
            // reward_per_token_stored_new =
            // reward_per_token_stored_old + (time_elapsed * reward_rate * PRECISION) / total_staked
            let reward = (time_elapsed * vault_data.reward_rate as u128 * PRECISION)
                / vault_data.total_staked as u128;

            vault_data.reward_per_token_stored = vault_data
                .reward_per_token_stored
                .checked_add(reward)
                .unwrap();
        }
        vault_data.last_updated = time_now;

        if user_profile.staked_amount > 0 {
            // Equation:
            // pending_rewards = staked_amount * (reward_per_token_stored - reward_per_token_paid) / PRECISION
            let pending = (user_profile.staked_amount as u128)
                .checked_mul(
                    vault_data
                        .reward_per_token_stored
                        .checked_sub(user_profile.reward_per_token_paid)
                        .unwrap(),
                )
                .unwrap()
                / PRECISION;

            // Equation:
            // rewards_earned_new = rewards_earned_old + pending_rewards
            user_profile.rewards_earned = user_profile
                .rewards_earned
                .checked_add(pending.try_into()?)
                .unwrap()
        }
        user_profile.reward_per_token_paid = vault_data.reward_per_token_stored;

        // Equation: total_staked_new = total_staked_old + deposit_amount
        vault_data.total_staked = vault_data.total_staked.checked_add(amount).unwrap();

        user_profile.authority = ctx.accounts.user.key();
        // Equation: staked_amount_new = staked_amount_old + deposit_amount
        user_profile.staked_amount = user_profile.staked_amount.checked_add(amount).unwrap();
        user_profile.last_updated = time_now;

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_stake_ata.to_account_info(),
                mint: ctx.accounts.staked_mint.to_account_info(),
                to: ctx.accounts.vault_stake_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );

        transfer_checked(cpi_ctx, amount, ctx.accounts.staked_mint.decimals)?;

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let user_profile = &mut ctx.accounts.user_stake_profile;

        let current_time = Clock::get()?.unix_timestamp;

        if vault.total_staked > 0 {
            let time_elapsed = current_time.checked_sub(vault.last_updated).unwrap() as u128;
            let rewards =
                (time_elapsed * vault.reward_rate as u128 * PRECISION) / vault.total_staked as u128;

            vault.reward_per_token_stored = vault.reward_per_token_stored.checked_add(rewards).unwrap();
        }

        vault.last_updated = current_time;

        let pending = (user_profile.staked_amount as u128)
            .checked_mul(
                (vault.reward_per_token_stored as u128)
                    .checked_sub(user_profile.reward_per_token_paid)
                    .unwrap(),
            )
            .unwrap()
            / PRECISION;

        user_profile.rewards_earned = (user_profile.rewards_earned)
            .checked_add(pending.try_into()?)
            .unwrap();

        user_profile.last_updated = current_time;
        user_profile.reward_per_token_paid = vault.reward_per_token_stored.clone();

        let bump = ctx.accounts.vault.bump;
        let adminkey = ctx.accounts.admin.key();
        let seeds = [b"vault", adminkey.as_ref(), &[bump]];
        let vaultpda = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_reward_ata.to_account_info(),
                mint: ctx.accounts.reward_mint.to_account_info(),
                to: ctx.accounts.user_reward_ata.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            vaultpda,
        );

        transfer_checked(
            cpi_ctx,
            user_profile.rewards_earned,
            ctx.accounts.reward_mint.decimals,
        )?;

        user_profile.rewards_earned = 0;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let user_profile = &mut ctx.accounts.user_profile;

        require!(
            amount <= user_profile.staked_amount,
            StakingErrors::InvalidAmount
        );
        require!(amount > 0, StakingErrors::InvalidAmount);

        require!(vault.total_staked > 0, StakingErrors::VaultEmpty);

        let current_time = Clock::get()?.unix_timestamp;

        let time_elapsed = current_time.checked_sub(vault.last_updated).unwrap() as u128;

        let rewards =
            (time_elapsed * vault.reward_rate as u128 * PRECISION) / vault.total_staked as u128;

        vault.reward_per_token_stored = vault.reward_per_token_stored.checked_add(rewards).unwrap();
        vault.last_updated = current_time;
        vault.total_staked = vault.total_staked.checked_sub(amount).unwrap();

        let pending = (user_profile.staked_amount as u128)
            .checked_mul(
                (vault.reward_per_token_stored as u128)
                    .checked_sub(user_profile.reward_per_token_paid)
                    .unwrap(),
            )
            .unwrap()
            / PRECISION;
        user_profile.rewards_earned = user_profile
            .rewards_earned
            .checked_add(pending.try_into()?)
            .unwrap();

        user_profile.reward_per_token_paid = vault.reward_per_token_stored;
        user_profile.last_updated = current_time;

        let bump = ctx.accounts.vault.bump;
        let adminkey = ctx.accounts.admin.key();
        let seeds = &[b"vault", adminkey.as_ref(), &[bump]];
        let vaultpda = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_staked_ata.to_account_info(),
                mint: ctx.accounts.staked_mint.to_account_info(),
                to: ctx.accounts.user_staked_ata.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            vaultpda,
        );

        transfer_checked(cpi_ctx, amount, ctx.accounts.staked_mint.decimals)?;
        user_profile.staked_amount = user_profile.staked_amount.checked_sub(amount).unwrap();
        Ok(())
    }
}

#[error_code]
pub enum StakingErrors {
    #[msg("user doesnt have stakes")]
    NotStaked,
    #[msg("user has stakend invalid amount")]
    InvalidAmount,
    #[msg("vault is empty")]
    VaultEmpty,
}
