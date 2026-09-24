use anchor_lang::prelude::*;
use anchor_lang::system_program::{Transfer, transfer};

declare_id!("vBWBj17Eu6xCK3FPxNYnvxch6sxe64TW9dVLDwcqD5S");

#[program]
pub mod blueshift_anchor_vault {
    use super::*;

    pub fn deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {

        msg!("depositing {} into the account",amount);
        require_eq!(ctx.accounts.vault.lamports(), 0, VaultErrors::VaultAlreadyExists);
        require_gt!(amount, Rent::get()?.minimum_balance(0), VaultErrors::InvalidAmount );
        transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(), 
                Transfer{
          from:ctx.accounts.user.to_account_info() ,
          to: ctx.accounts.vault.to_account_info() ,  
        },
        ),
        amount,
        )?;
                
        Ok(())
    }

    pub fn withdraw(ctx: Context<VaultAction>) -> Result<()> {
        require_neq!(ctx.accounts.vault.lamports(), 0, VaultErrors::InvalidAmount);

        let signer_key = ctx.accounts.user.key();
        let vaultpda = [b"vault", signer_key.as_ref(),&[ctx.bumps.vault]];

        transfer(CpiContext::new_with_signer(ctx.accounts.system_program.to_account_info(),
        Transfer { from: ctx.accounts.vault.to_account_info(), to: ctx.accounts.user.to_account_info() },
       &[&vaultpda]), 
            ctx.accounts.vault.lamports())?;
        Ok(())
    }
}


#[derive(Accounts)]
pub struct VaultAction<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds= [b"vault",user.key.as_ref()],
        bump,
        )]

    pub vault: SystemAccount<'info>,

    system_program: Program<'info,System>
}

#[error_code]
pub enum VaultErrors{
   #[msg ("vault already exists")]
    VaultAlreadyExists,
    #[msg ("invalid amount")]
    InvalidAmount,
}
