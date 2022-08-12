use crate::errors::WeddingError;
use anchor_lang::prelude::*;
use std::cmp::Ordering;

// checks if partner is able to get married (not married to another initialized elsewhere)
pub fn validate_partner(partner: &UncheckedAccount) -> Result<()> {
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

pub fn sort_pubkeys<'a>(pubkey_a: &'a Pubkey, pubkey_b: &'a Pubkey) -> (&'a Pubkey, &'a Pubkey) {
    match pubkey_a.cmp(pubkey_b) {
        Ordering::Less => (pubkey_a, pubkey_b),
        Ordering::Greater => (pubkey_b, pubkey_a),
        Ordering::Equal => (pubkey_a, pubkey_b),
    }
}

pub fn check_account_initialized(account: &UncheckedAccount) -> bool {
    let account = account.to_account_info();

    let data_empty = account.data_is_empty();
    let lamps = account.lamports();
    let has_lamps = lamps > 0;

    !data_empty || has_lamps
}
