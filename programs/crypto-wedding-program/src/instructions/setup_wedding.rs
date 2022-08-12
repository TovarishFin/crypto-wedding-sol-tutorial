use crate::state::{Status, Wedding};
use crate::util::validate_partner;
use anchor_lang::prelude::*;

pub fn setup_wedding(ctx: Context<SetupWedding>) -> Result<()> {
    let wedding = &mut ctx.accounts.wedding;

    // check that both partners have 0 balance, data, and not owned by anyone
    validate_partner(&ctx.accounts.partner0)?;
    validate_partner(&ctx.accounts.partner1)?;

    wedding.status = Status::Created;
    wedding.creator = *ctx.accounts.creator.key;
    wedding.partner0 = *ctx.accounts.partner0.key;
    wedding.partner1 = *ctx.accounts.partner1.key;

    Ok(())
}

#[derive(Accounts)]
pub struct SetupWedding<'info> {
    #[account(mut)]
    pub creator: Signer<'info>, // creator can be someone other than the two partners
    /// CHECK: only being used to compute PDAs
    pub user_partner0: AccountInfo<'info>,
    /// CHECK: TODO: only being used to compute PDAs
    pub user_partner1: AccountInfo<'info>,
    #[account(
        init,
        payer = creator,
        space = Wedding::space(),
        seeds = [
            b"wedding",
            Wedding::seed_partner0(user_partner0.key, user_partner1.key).key().as_ref(),
            Wedding::seed_partner1(user_partner0.key, user_partner1.key).key().as_ref(),
        ],
        bump,
    )]
    pub wedding: Account<'info, Wedding>,
    #[account(
        seeds = [
            b"partner",
            user_partner0.key().as_ref(),
        ],
        bump,
    )]
    /// CHECK: we are doing all needed checks manually on this account
    pub partner0: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"partner",
            user_partner1.key().as_ref(),
        ],
        bump,
    )]
    /// CHECK: we are doing all needed checks manually on this account
    pub partner1: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
