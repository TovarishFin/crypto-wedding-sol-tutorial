use crate::state::*;
use anchor_lang::prelude::*;

/// sets up partner storage account for a user. Gets everything ready short of the answer.
pub fn setup_partner(ctx: Context<SetupPartner>, name: String, vows: String) -> Result<()> {
    let partner = &mut ctx.accounts.partner;
    partner.wedding = ctx.accounts.wedding.key();
    partner.user = ctx.accounts.user.key();
    partner.name = name;
    partner.vows = vows;

    Ok(())
}

#[derive(Accounts)]
#[instruction(name: String, vows: String)]
pub struct SetupPartner<'info> {
    #[account(mut)]
    /// One of the user partners getting married
    pub user: Signer<'info>,
    /// The other user partner that `user` is getting married to.
    /// CHECK: only used for computing wedding PDA
    pub other: AccountInfo<'info>,
    #[account(
        init,
        payer = user,
        space = Partner::space(&name, &vows),
        seeds = [
            b"partner",
            user.key().as_ref(),
        ],
        bump,
    )]
    /// Partner storage account derived from the `user`
    pub partner: Account<'info, Partner>,
    #[account(
        seeds = [
            b"wedding",
            Wedding::seed_partner0(user.key, other.key).key().as_ref(),
            Wedding::seed_partner1(user.key, other.key).key().as_ref(),
        ],
        bump,
    )]
    /// Wedding storage account
    pub wedding: Account<'info, Wedding>,
    pub system_program: Program<'info, System>,
}
