use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod util;
use instructions::*;

declare_id!("36qCaAFg7XYye43TD56yqkZv6NAcoWNr87Lad15x6G4v");

#[program]
pub mod crypto_wedding_program {
    use super::*;

    pub fn setup_wedding(ctx: Context<SetupWedding>) -> Result<()> {
        instructions::setup_wedding(ctx)
    }

    pub fn cancel_wedding(ctx: Context<CancelWedding>) -> Result<()> {
        instructions::cancel_wedding(ctx)
    }

    pub fn setup_partner(ctx: Context<SetupPartner>, name: String, vows: String) -> Result<()> {
        instructions::setup_partner(ctx, name, vows)
    }

    pub fn close_partner(ctx: Context<ClosePartner>) -> Result<()> {
        instructions::close_partner(ctx)
    }

    pub fn give_answer(ctx: Context<GiveAnswer>, answer: bool) -> Result<()> {
        instructions::give_answer(ctx, answer)
    }

    pub fn divorce(ctx: Context<Divorce>) -> Result<()> {
        instructions::divorce(ctx)
    }
}
