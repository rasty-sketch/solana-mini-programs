use anchor_lang::prelude::*;

declare_id!("g64oLeSMNrDp9nesrVaPZuNs4KicKDenogR3UAJ55u3");

#[program]
pub mod cust_fb {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,name: String,status: String, twitter: String) -> Result<()> {
        msg!("account created with name:{}, and status: {}, and twitter: {}",name,status,twitter);
        let facebook_data = &mut ctx.accounts.facebook_account;

        facebook_data.name = name;
        facebook_data.status = status;
        facebook_data.twitter = twitter;
        facebook_data.bump = ctx.bumps.facebook_account;
        facebook_data.authority = *ctx.accounts.user.key;


        Ok(())
    }

    pub fn update(ctx: Context<Update>, status: String) -> Result<()> {
        msg!("Updating status to: {}", status);
        ctx.accounts.facebook_account.status = status;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account[mut]]
    user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = ["self-custodial-facebook2".as_bytes(), user.key().as_ref()],
        bump,
        space = 8 + FacebookAccount::INIT_SPACE
    )]
    facebook_account: Account<'info,FacebookAccount>,

    system_program: Program<'info,System>
}



#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut,)]
    user: Signer<'info>,

    #[account(
        mut,
        seeds = ["self-custodial-facebook2".as_bytes(), user.key().as_ref()],
        bump = facebook_account.bump
    )]
    facebook_account: Account<'info, FacebookAccount>,
}


#[account]
#[derive(InitSpace)]
pub struct FacebookAccount {
    pub authority: Pubkey,
    pub bump: u8,
    #[max_len(10)]
    pub name: String,
    #[max_len(100)]
    pub status: String,
    #[max_len(10)]
    pub twitter: String,
}


