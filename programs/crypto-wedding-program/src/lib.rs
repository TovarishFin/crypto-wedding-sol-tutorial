use anchor_lang::{error_code, prelude::*};
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum Status {
    Created,
    Marrying,
    Married,
    Divorcing,
    Divorced,
}

pub fn sort_pubkeys<'a>(pubkey_a: &'a Pubkey, pubkey_b: &'a Pubkey) -> (&'a Pubkey, &'a Pubkey) {
    match pubkey_a.cmp(pubkey_b) {
        Ordering::Less => (pubkey_a, pubkey_b),
        Ordering::Greater => (pubkey_b, pubkey_a),
        Ordering::Equal => (pubkey_a, pubkey_b),
    }
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

#[error_code]
pub enum WeddingError {
    #[msg("partner data not empty")]
    PartnerDataNotEmpty,
    #[msg("partner lamports not zero")]
    PartnerBalanceNotZero,
}
