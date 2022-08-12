use crate::errors::WeddingError;
use crate::state::{Status, Wedding};
use anchor_lang::{prelude::*, AccountsClose};

pub fn cancel_wedding(ctx: Context<CancelWedding>) -> Result<()> {
    // wedding PDA storage stores partner PDAs...
    // we need to determine signing user's partner PDA
    let (partner_pda, _) = Pubkey::find_program_address(
        &[b"partner", ctx.accounts.user.key().to_bytes().as_ref()],
        ctx.program_id,
    );
    let user_matches0 = partner_pda == ctx.accounts.wedding.partner0.key();
    let user_matches1 = partner_pda == ctx.accounts.wedding.partner1.key();
    let user_matches_creator = ctx.accounts.user.key() == ctx.accounts.wedding.creator.key();
    // ensure signer is partner0 or partner1
    require!(
        user_matches0 || user_matches1 || user_matches_creator,
        WeddingError::NotWeddingMember
    );

    let wedding = &ctx.accounts.wedding;

    // should only be able to cancel if wedding is in Status::Created or Statusq::Marrying
    if wedding.status != Status::Created && wedding.status != Status::Marrying {
        return Err(WeddingError::CannotCancel.into());
    }

    // return storage costs to creator
    wedding.close(ctx.accounts.creator.to_account_info())?;

    Ok(())
}

#[derive(Accounts)]
pub struct CancelWedding<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // one of the partners or creator who wants to cancel
    /// CHECK: TODO: we are only sending lamps... dont need to check anything
    #[account(mut)]
    pub creator: AccountInfo<'info>, // wedding creator... could be anyone
    /// CHECK: TODO: only being used to compute PDAs
    pub user_partner0: AccountInfo<'info>,
    /// CHECK: TODO: only being used to compute PDAs
    pub user_partner1: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [
            b"wedding",
            Wedding::seed_partner0(user_partner0.key, user_partner1.key).key().as_ref(),
            Wedding::seed_partner1(user_partner0.key, user_partner1.key).key().as_ref(),
        ],
        bump,
        has_one = creator @ WeddingError::InvalidCreator,
    )]
    pub wedding: Account<'info, Wedding>,
}
