use anchor_lang::{error_code, prelude::*, AccountsClose};
use std::cmp::Ordering;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod crypto_wedding_program {
    use super::*;

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

    /// sets up partner storage account for a user. Gets everything ready short of the answer.
    pub fn setup_partner(ctx: Context<SetupPartner>, name: String, vows: String) -> Result<()> {
        let partner = &mut ctx.accounts.partner;
        partner.wedding = ctx.accounts.wedding.key();
        partner.user = ctx.accounts.user.key();
        partner.name = name;
        partner.vows = vows;

        Ok(())
    }

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
}

// checks if partner is able to get married (not married to another initialized elsewhere)
fn validate_partner(partner: &UncheckedAccount) -> Result<()> {
    let partner = partner.to_account_info();

    // partner PDA should not have any data yet
    let data_empty = partner.data_is_empty();
    if !data_empty {
        return Err(WeddingError::PartnerDataNotEmpty.into());
    }

    // partner PDA should not have rent paid yet
    let lamps = partner.lamports();
    let has_lamps = lamps > 0;
    if has_lamps {
        return Err(WeddingError::PartnerBalanceNotZero.into());
    }

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum Status {
    Created,
    Marrying,
    Married,
    Divorcing,
    Divorced,
}

fn sort_pubkeys<'a>(pubkey_a: &'a Pubkey, pubkey_b: &'a Pubkey) -> (&'a Pubkey, &'a Pubkey) {
    match pubkey_a.cmp(pubkey_b) {
        Ordering::Less => (pubkey_a, pubkey_b),
        Ordering::Greater => (pubkey_b, pubkey_a),
        Ordering::Equal => (pubkey_a, pubkey_b),
    }
}

fn check_account_initialized(account: &UncheckedAccount) -> bool {
    let account = account.to_account_info();

    let data_empty = account.data_is_empty();
    let lamps = account.lamports();
    let has_lamps = lamps > 0;

    !data_empty || has_lamps
}

#[account]
pub struct Wedding {
    pub creator: Pubkey,
    pub partner0: Pubkey, // pubkey is a user derived PDA, not a user account
    pub partner1: Pubkey, // pubkey is a user derived PDA , not a user account
    pub status: Status,
}

impl Wedding {
    pub fn space() -> usize {
        // discriminator + 3 * pubkey + enum
        8 + (32 * 3) + 2
    }

    pub fn seed_partner0<'a>(pubkey_a: &'a Pubkey, pubkey_b: &'a Pubkey) -> &'a Pubkey {
        let (pubkey0, _) = sort_pubkeys(pubkey_a, pubkey_b);

        pubkey0
    }

    pub fn seed_partner1<'a>(pubkey_a: &'a Pubkey, pubkey_b: &'a Pubkey) -> &'a Pubkey {
        let (_, pubkey1) = sort_pubkeys(pubkey_a, pubkey_b);

        pubkey1
    }
}

#[account]
pub struct Partner {
    pub wedding: Pubkey,
    pub user: Pubkey,
    pub name: String,
    pub vows: String,
    pub answer: bool,
}

impl Partner {
    pub fn space(name: &str, vows: &str) -> usize {
        // discriminator + 2 * pubkey + nameLen + name + vowsLen + vows + bool
        8 + (32 * 2) + 4 + name.len() + 4 + vows.len() + 1
    }
}

#[error_code]
pub enum WeddingError {
    #[msg("partner data not empty")]
    PartnerDataNotEmpty,
    #[msg("partner lamports not zero")]
    PartnerBalanceNotZero,
    #[msg("signer is not wedding member")]
    NotWeddingMember,
    #[msg("cannot cancel after created status")]
    CannotCancel,
    #[msg("creator does not match wedding storage")]
    InvalidCreator,
    #[msg("partner cannot be closed while wedding is initialized")]
    WeddingInitialized,
    #[msg("partner wedding does not match account wedding")]
    PartnerWeddingNotWedding,
    #[msg("cannot answer during invalid status")]
    InvalidAnswerStatus,
    #[msg("cannot divorce during invalid status")]
    InvalidDivorceStatus,
}
