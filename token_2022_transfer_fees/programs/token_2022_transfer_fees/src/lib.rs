use anchor_lang::prelude::*;
use anchor_lang::system_program::{ create_account, CreateAccount };
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{
        initialize_mint2,
        spl_token_2022::{
            extension::{
                transfer_fee::TransferFeeConfig,
                BaseStateWithExtensions,
                ExtensionType,
                StateWithExtensions,
            },
            pod::PodMint,
            state::Mint as MintState,
        },
        InitializeMint2,
    },
    token_interface::{
        transfer_checked_with_fee,
        transfer_fee_initialize,
        Mint,
        Token2022,
        TokenAccount,
        TransferCheckedWithFee,
        TransferFeeInitialize,
    },
};

declare_id!("592nNMRg8cz6Y4KFTMeE4YXtqmWeNnbFZNz2zSmqB4uD");

#[program]
pub mod token_2022_transfer_fees {
    use anchor_spl::token_2022;

    use super::*;

    pub fn initialize_transfer_fee_config(
        ctx: Context<Initialize>,
        transfer_fee_basis_points: u16,
        maximum_fee: u64
    ) -> Result<()> {
        let mint_size = ExtensionType::try_calculate_account_len::<PodMint>(
            &[ExtensionType::TransferFeeConfig]
        )?;

        let lamports = Rent::get()?.minimum_balance(mint_size);

        let create_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            CreateAccount {
                from: ctx.accounts.signer.to_account_info(),
                to: ctx.accounts.mint_account.to_account_info(),
            }
        );
        create_account(create_ctx, lamports, mint_size as u64, &token_2022::ID)?;

        let transfer_fee_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferFeeInitialize {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
            }
        );
        transfer_fee_initialize(
            transfer_fee_ctx,
            Some(&ctx.accounts.signer.key()),
            Some(&ctx.accounts.signer.key()),
            transfer_fee_basis_points,
            maximum_fee
        )?;

        let mint_init_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            InitializeMint2 {
                mint: ctx.accounts.mint_account.to_account_info(),
            }
        );
        initialize_mint2(
            mint_init_ctx,
            2,
            &ctx.accounts.signer.key(),
            Some(&ctx.accounts.signer.key())
        )?;

        Ok(())
    }

    pub fn transfer_checked_alsowith_fee(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let mint = &ctx.accounts.mint_account.to_account_info();
        let mint_data = mint.data.borrow();
        let mint_with_extensions = StateWithExtensions::<MintState>::unpack(&mint_data)?;
        let extension_data = mint_with_extensions.get_extension::<TransferFeeConfig>()?;

        let epoch = Clock::get()?.epoch;
        let fee = extension_data.calculate_epoch_fee(epoch, amount).unwrap();

        let decimals = ctx.accounts.mint_account.decimals;
        let ctx_transfer_checked_fee = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferCheckedWithFee {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                source: ctx.accounts.sender_associated_account.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
                destination: ctx.accounts.recipient_associated_account.to_account_info(),
                authority: ctx.accounts.sender.to_account_info(),
            }
        );

        transfer_checked_with_fee(ctx_transfer_checked_fee, amount, decimals, fee)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub mint_account: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    sender: Signer<'info>,
    recipient: SystemAccount<'info>,
    #[account(mut)]
    mint_account: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_account,
        associated_token::authority = sender,
        associated_token::token_program = token_program
    )]
    pub sender_associated_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = sender,
        associated_token::mint = mint_account,
        associated_token::authority = recipient,
        associated_token::token_program = token_program
    )]
    pub recipient_associated_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token2022>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}
