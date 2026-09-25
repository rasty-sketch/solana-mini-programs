use anchor_lang::prelude::*;
use anchor_spl::token_interface::{ close_account, transfer_checked, CloseAccount, TransferChecked };
pub(crate) mod types;
use types::*;

declare_id!("72PTvNB2wg7TyyaKySLreb1kmSwajky5XKqmDbo3FW7P");

#[program]
pub mod token_swap_escrow {
    use super::*;

    pub fn make(
        ctx: Context<Make>,
        amount: u64,
        cost: u64,
        deadline: i64,
        escrow_id: u32
    ) -> Result<()> {
        require!(amount > 0, EscrowError::InvalidAmount);
        require!(cost > 0, EscrowError::InvalidCost);

        let clock = Clock::get()?;
        require!(deadline > clock.unix_timestamp, EscrowError::InvalidDeadline);
        require!(
            ctx.accounts.offered_mint.key() != ctx.accounts.wanted_mint.key(),
            EscrowError::SameMint
        );

        let data = &mut ctx.accounts.escrow;
        let authority = ctx.accounts.user.key();

        data.set_inner(Escrow {
            authority,
            amount,
            cost,
            escrow_id,
            status: Status::Open,
            deadline,
            offered_mint: ctx.accounts.offered_mint.key(),
            wanted_mint: ctx.accounts.wanted_mint.key(),
            version: 0,
            bump: ctx.bumps.escrow,
        });

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), TransferChecked {
            from: ctx.accounts.user_ata.to_account_info(),
            mint: ctx.accounts.offered_mint.to_account_info(),
            to: ctx.accounts.escrow_ata.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        });

        transfer_checked(cpi_ctx, amount, ctx.accounts.offered_mint.decimals)?;

        Ok(())
    }

    pub fn take(ctx: Context<Take>, escrow_id: u32) -> Result<()> {
        let clock = Clock::get()?;

        require!(ctx.accounts.escrow.deadline >= clock.unix_timestamp, EscrowError::Expired);
        require!(ctx.accounts.escrow.status == Status::Open, EscrowError::NotOpen);
        require!(
            ctx.accounts.buyer.key() != ctx.accounts.escrow.authority,
            EscrowError::MakerCannotTake
        );

        let user_key = ctx.accounts.user.key();
        let bump = ctx.accounts.escrow.bump;
        let cost = ctx.accounts.escrow.cost;
        let amount = ctx.accounts.escrow.amount;

        let seeds = [b"escrow".as_ref(), user_key.as_ref(), &escrow_id.to_le_bytes(), &[bump]];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), TransferChecked {
            from: ctx.accounts.buyer_wanted_ata.to_account_info(),
            mint: ctx.accounts.wanted_mint.to_account_info(),
            to: ctx.accounts.user_wanted_ata.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        });

        transfer_checked(cpi_ctx, cost, ctx.accounts.wanted_mint.decimals)?;

        let cpi_ctx2 = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.escrow_ata.to_account_info(),
                mint: ctx.accounts.offered_mint.to_account_info(),
                to: ctx.accounts.buyer_ata.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            signer_seeds
        );

        transfer_checked(cpi_ctx2, amount, ctx.accounts.offered_mint.decimals)?;

        ctx.accounts.escrow.status = Status::Completed;

        let cpi_ctx3 = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            CloseAccount {
                account: ctx.accounts.escrow_ata.to_account_info(),
                destination: ctx.accounts.user.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            signer_seeds
        );

        close_account(cpi_ctx3)?;

        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>, escrow_id: u32) -> Result<()> {
        require!(ctx.accounts.escrow.status == Status::Open, EscrowError::NotOpen);

        let user_key = ctx.accounts.user.key();

        let bump = ctx.accounts.escrow.bump;

        let seeds = [b"escrow".as_ref(), user_key.as_ref(), &escrow_id.to_le_bytes(), &[bump]];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.escrow_ata.to_account_info(),
                mint: ctx.accounts.offered_mint.to_account_info(),
                to: ctx.accounts.user_ata.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            signer_seeds
        );

        transfer_checked(cpi_ctx, ctx.accounts.escrow.amount, ctx.accounts.offered_mint.decimals)?;

        let cpi_ctx2 = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            CloseAccount {
                account: ctx.accounts.escrow_ata.to_account_info(),
                destination: ctx.accounts.user.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            signer_seeds
        );

        close_account(cpi_ctx2)?;

        ctx.accounts.escrow.status = Status::Cancelled;
        Ok(())
    }

    pub fn expired(ctx: Context<Expired>, escrow_id: u32) -> Result<()> {
        let clock = Clock::get()?;
        require!(ctx.accounts.escrow.deadline < clock.unix_timestamp, EscrowError::NotExpired);
        require!(ctx.accounts.escrow.status == Status::Open, EscrowError::NotOpen);

        let bump = ctx.accounts.escrow.bump;
        let user_key = ctx.accounts.user.key();
        let seeds = [b"escrow".as_ref(), user_key.as_ref(), &escrow_id.to_le_bytes(), &[bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx: CpiContext<'_, '_, '_, '_, TransferChecked<'_>> = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.escrow_ata.to_account_info(),
                mint: ctx.accounts.offered_mint.to_account_info(),
                to: ctx.accounts.user_ata.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            signer_seeds
        );

        transfer_checked(cpi_ctx, ctx.accounts.escrow.amount, ctx.accounts.offered_mint.decimals)?;

        let cpi_ctx2 = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            CloseAccount {
                account: ctx.accounts.escrow_ata.to_account_info(),
                destination: ctx.accounts.user.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            signer_seeds
        );

        close_account(cpi_ctx2)?;

        ctx.accounts.escrow.status = Status::Expired;

        Ok(())
    }
}
