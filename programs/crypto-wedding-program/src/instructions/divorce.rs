use crate::errors::WeddingError;
use crate::state::*;
use anchor_lang::{prelude::*, AccountsClose};

pub fn divorce(ctx: Context<Divorce>) -> Result<()> {
    let partner = &mut ctx.accounts.partner;
    let other_partner = &mut ctx.accounts.other_partner;
    let wedding = &mut ctx.accounts.wedding;

    partner.answer = false;

    match wedding.status {
        Status::Married => {
            wedding.status = Status::Divorcing;
            Ok(())
        }
        Status::Divorcing => match other_partner.answer {
            true => Ok(()),
            false => wedding.close(ctx.accounts.creator.to_account_info()),
        },
        _ => return Err(WeddingError::InvalidDivorceStatus.into()),
    }
}

#[derive(Accounts)]
pub struct Divorce<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: this is only used to compute wedding PDA
    pub other: AccountInfo<'info>,
    /// CHECK: TODO: we are only sending lamps... dont need to check anything
    #[account(mut)]
    pub creator: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [
            b"partner",
            user.key().as_ref(),
        ],
        bump,
        has_one = wedding @ WeddingError::PartnerWeddingNotWedding,
    )]
    pub partner: Account<'info, Partner>,
    #[account(
        seeds = [
            b"partner",
            other.key().as_ref(),
        ],
        bump,
        has_one = wedding @ WeddingError::PartnerWeddingNotWedding,
    )]
    pub other_partner: Account<'info, Partner>,
    #[account(
        mut,
        seeds = [
            b"wedding",
            Wedding::seed_partner0(user.key, other.key).key().as_ref(),
            Wedding::seed_partner1(user.key, other.key).key().as_ref(),
        ],
        bump,
        has_one = creator @ WeddingError::InvalidCreator,
    )]
    // ensure that Wedding PDA exists before updating
    pub wedding: Account<'info, Wedding>,
    pub system_program: Program<'info, System>,
}
