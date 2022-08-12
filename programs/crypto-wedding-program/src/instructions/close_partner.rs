use crate::errors::WeddingError;
use crate::state::*;
use crate::util::check_account_initialized;
use anchor_lang::{prelude::*, AccountsClose};

pub fn close_partner(ctx: Context<ClosePartner>) -> Result<()> {
    // should only be able to close if Wedding PDA no longer exists
    let wedding_initialized = check_account_initialized(&ctx.accounts.wedding);
    if wedding_initialized {
        return Err(WeddingError::WeddingInitialized.into());
    }

    // return storage costs to user who created partner PDA storage
    ctx.accounts
        .partner
        .close(ctx.accounts.user.to_account_info())?;

    Ok(())
}

#[derive(Accounts)]
pub struct ClosePartner<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: this is only used to compute wedding PDA
    pub other: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [
            b"partner",
            user.key().as_ref(),
        ],
        bump,
    )]
    // ensures that user cannot supply false `other` account by checking partner storage
    #[account(has_one = wedding @ WeddingError::PartnerWeddingNotWedding)]
    pub partner: Account<'info, Partner>,
    #[account(
        seeds = [
            b"wedding",
            Wedding::seed_partner0(user.key, other.key).key().as_ref(),
            Wedding::seed_partner1(user.key, other.key).key().as_ref(),
        ],
        bump,
    )]
    /// CHECK: this is only used for ensuring non-existance false `other` account is checked via
    /// user partner PDA
    pub wedding: UncheckedAccount<'info>,
}
