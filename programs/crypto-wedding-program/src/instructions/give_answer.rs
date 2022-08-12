use crate::errors::WeddingError;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn give_answer(ctx: Context<GiveAnswer>, answer: bool) -> Result<()> {
    let partner = &mut ctx.accounts.partner;
    let other_partner = &mut ctx.accounts.other_partner;
    let wedding = &mut ctx.accounts.wedding;

    // update partner's answer no matter what as long as its in the right status
    partner.answer = answer;

    match wedding.status {
        Status::Created => match answer {
            true => {
                wedding.status = Status::Marrying;
                Ok(())
            }
            false => Ok(()),
        },
        Status::Marrying => match (answer, other_partner.answer) {
            (true, true) => {
                wedding.status = Status::Married;
                Ok(())
            }
            (_, _) => Ok(()),
        },
        _ => return Err(WeddingError::InvalidAnswerStatus.into()),
    }
}

#[derive(Accounts)]
pub struct GiveAnswer<'info> {
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
    )]
    // ensure that Wedding PDA exists before updating
    pub wedding: Account<'info, Wedding>,
    pub system_program: Program<'info, System>,
}
