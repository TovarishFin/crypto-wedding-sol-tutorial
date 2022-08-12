use crate::util::*;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum Status {
    Created,
    Marrying,
    Married,
    Divorcing,
    Divorced,
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
