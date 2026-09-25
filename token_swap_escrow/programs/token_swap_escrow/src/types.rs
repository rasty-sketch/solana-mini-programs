use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
#[instruction(amount: u64,cost:u64,deadline:i64,escrow_id:u32)]
pub struct Make<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub offered_mint: InterfaceAccount<'info, Mint>,
    pub wanted_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = offered_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        seeds = [b"escrow",user.key().as_ref(), &escrow_id.to_le_bytes()],
        bump,
        space = 8 + Escrow::INIT_SPACE
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = user,
        associated_token::mint = offered_mint,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub escrow_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = wanted_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_wanted_ata: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(escrow_id:u32)]
pub struct Take<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    ///CHECK: make sure its the right seller
    #[account(constraint = user.key() == escrow.authority,
    mut
)]
    pub user: UncheckedAccount<'info>,

    #[account(address = escrow.offered_mint)]
    pub offered_mint: InterfaceAccount<'info, Mint>,

    #[account(address = escrow.wanted_mint)]
    pub wanted_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        close = user,
        seeds = [b"escrow", user.key().as_ref(),&escrow_id.to_le_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = offered_mint,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub escrow_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = offered_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = wanted_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_wanted_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = wanted_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_wanted_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(escrow_id:u32)]
pub struct Cancel<'info> {
    #[account(
        mut,
        address = escrow.authority
    )]
    pub user: Signer<'info>,

    #[account(address = escrow.offered_mint)]
    pub offered_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"escrow", user.key().as_ref(), &escrow_id.to_le_bytes()],
        bump = escrow.bump,
        close = user
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = offered_mint,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub escrow_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = offered_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(escrow_id:u32)]
pub struct Expired<'info> {
    #[account(mut)]
    pub anyone: Signer<'info>,

    /// CHECK: validated via escrow.authority constraint
    #[account(address = escrow.authority)]
    pub user: UncheckedAccount<'info>,

    #[account(address = escrow.offered_mint)]
    pub offered_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"escrow", user.key().as_ref(), &escrow_id.to_le_bytes()],
        bump = escrow.bump,
        close = anyone,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = offered_mint,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub escrow_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = anyone,
        associated_token::mint = offered_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub authority: Pubkey,
    pub amount: u64,
    pub cost: u64,
    pub escrow_id: u32,
    pub status: Status,
    pub deadline: i64,
    pub offered_mint: Pubkey,
    pub wanted_mint: Pubkey,
    pub version: u8,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Clone, InitSpace)]
pub enum Status {
    Open,
    Completed,
    Cancelled,
    Expired,
}

#[error_code]
pub enum EscrowError {
    #[msg("amount should be more than 0")]
    InvalidAmount,
    #[msg("cost should be more than 0")]
    InvalidCost,
    #[msg("Invalid ID")]
    InvalidId,
    #[msg("deadline shouldnt be in the past")]
    InvalidDeadline,
    #[msg("Escrow has expired")]
    Expired,
    #[msg("Escrow isnt open")]
    NotOpen,
    #[msg("maker cannot take")]
    MakerCannotTake,
    #[msg("offered cant be wanted")]
    SameMint,
    #[msg("deadline is still due")]
    NotExpired,
}
