# step 6

Finally after all of our hard work and testing, we are going to deploy our program to `devnet`.

## Deploying our program

Let's start by making sure that you have set solana locally to use devnet. Run this in your
terminal.

```sh
solana config set --url devnet
```

Make sure that you have some SOL by requesting an airdrop:

```sh
solana airdrop 2
```

Now let's make sure our program is built using our latest code:

```sh
anchor build
```

Now for the weird clunky manual part... we need to get our program ID and manually set it in our code.

```sh
anchor keys list
```

You should get something like this:

```sh
crypto_wedding_program: <your-public-key>
```

Copy that public key and change the id you find in `src/lib.rs` from

```rust
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
```

to

```rust
declare_id!("<your-public-key>");
```

Obviously, `<your-public-key>` is the actual output you get from `anchor keys list`.

We need to build again after updating the program id in the code... (funky... I know...)

```
anchor build
```

Lastly, we need to update `Anchor.toml` to use `devnet`.

Change

```toml
[provider]
cluster = "localnet"
```

to

```toml
[provider]
cluster = "devnet"
```

Remember to change this back if you want to write more tests using your `localnet` network.

We can now deploy our program:

```sh
anchor deploy
```

You should get something similar to:

```
Deploying workspace: https://api.devnet.solana.com
Upgrade authority: <path-to-your-wallet>
Deploying program "crypto-wedding-program"...
Program path: <paath-to-your-build>
Program Id: <your-program-pub-key>

Deploy success
```

You can go find your program on https://explorer.solana.com/<your-program-pub-key-here>?cluster=devnet
Make sure to paste in your public key :)

We can also change this in `Anchor.toml`

```toml
[programs.localnet]
crypto_wedding_program = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
```

to this:

```toml
[programs.localnet]
crypto_wedding_program = "<your-program-pub-key-here>"
[programs.devnet]
crypto_wedding_program = "<your-program-pub-key-here>"
```

and then try running our tests on devnet:

```sh
anchor test
```

Unfortunately, this will likely fail due to sending too many requests at one time :(
If you can find a `devnet` node that will allow you to send many txs at once, you can try this
out on your own. If you want to try interacting with your deployed program, you can try
tweaking the [crypto-wedding-cli](https://github.com/TovarishFin/crypto-wedding-sol-cli) that
I built. You will need to update it with your own address. Perhaps I will make a tutorial for
that at some point...

## Optional: code cleanup

We have everything in a single file for our program right now... that doesn't feel great.
In most solana programs, things are split up into a few categories:

- instructions
- state
- errors

Depending on how big each of these are, they can be a single file, or they can be a directory.
The layout follows the normal rules for a rust project. You can find more about the rules for that
[here](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html).

## Breaking state out to its own module/file

Let's start by seprating out our state. Create new file at `programs/crypto-wedding-program/src/state.rs`.

Let's remove the following from `programs/crypto-wedding-program/src/lib.rs` and add to our new
state file.

```rust
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
        let (pubkey0, _) = crate::sort_pubkeys(pubkey_a, pubkey_b);

        pubkey0
    }

    pub fn seed_partner1<'a>(pubkey_a: &'a Pubkey, pubkey_b: &'a Pubkey) -> &'a Pubkey {
        let (_, pubkey1) = crate::sort_pubkeys(pubkey_a, pubkey_b);

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
```

Because of the way rust modules works, we are going to need to declare this as a public module
in our `lib.rs` file. We can also make everything from `state.rs` available to `lib.rs` to avoid
rewriting all of the paths for state involved things.

```rust
use anchor_lang::{error_code, prelude::*, AccountsClose};
use std::cmp::Ordering;

pub mod state;
use state::*;

declare_id!("36qCaAFg7XYye43TD56yqkZv6NAcoWNr87Lad15x6G4v");

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
```

Run `anchor build` to make sure we didn't break anything.

## Breaking errors out to their own module/file

Create a new file at `programs/crypto-wedding-program/src/errors.rs` and remove our `WeddingError`
enum into it's own `errors.rs`:

```rust
use anchor_lang::prelude::*;

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
```

Again, we need to make the errors module available to the rest of the code in `lib.rs`.

```rust
use anchor_lang::{prelude::*, AccountsClose};
use std::cmp::Ordering;

pub mod errors;
pub mod state;
use errors::*;
use state::*;

//
// ..
//
```

Only the top part of the file is shown for brevity.

## Breaking util functions out to their own module/file

Create a new file at `programs/crypto-wedding-program/src/util.rs`. In that file move from `lib.rs`
to `util.rs` the following.

```rust
use crate::errors::WeddingError;
use anchor_lang::prelude::*;

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
```

Note that we changed `validate_partner` to public so that it can be accessed from outside
of it's own module.

Update `lib.rs` with module declarations and usage as follows:

```rust
use anchor_lang::{prelude::*, AccountsClose};
use std::cmp::Ordering;

pub mod errors;
pub mod state;
pub mod util;
use errors::*;
use state::*;
use util::*;

//
// ...
//
```

## Breaking instructions out to their own module/file

We are going to take each of the public functions inside of our `crypto_wedding_program` and
put them into their own respective file inside of a directory called `instructions`.

Go ahead and make a new directory at `programs/crypto-wedding-program/src/instructions`.

Let's break out `setup_wedding` first.
Create a new file at `/programs/crypto_wedding_program/src/instructions/setup_wedding.rs`.

Remove the following from `lib.rs` and put it into `setup_wedding.rs`.

```rust
use crate::state::{Status, Wedding};
use crate::util::validate_partner;
use anchor_lang::prelude::*;

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
```

We then need to create a module file in our instructions directory like so: `programs/crypto-wedding-program/src/instructions/mod.rs`

```
pub mod setup_wedding;
pub use setup_wedding::*;
```

This makes everything in our `setup_wedding.rs` file available to `lib.rs` to import.

```rust
use anchor_lang::{prelude::*, AccountsClose};
use std::cmp::Ordering;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod util;
use errors::*;
use instructions::*;
use state::*;

//
// ...
//

#[program]
pub mod crypto_wedding_program {
    use super::*;

    pub fn setup_wedding(ctx: Context<SetupWedding>) -> Result<()> {
        instructions::setup_wedding(ctx)
    }

    //
    // ...
    //
}
```

We need to again make the `instructions` module available. We then use everything in the
`instructions` module to make it available here. We still use `instructions::setup_wedding`
in our calls because our function is named the same as our function declaration in this file.
Without this, it would become an recursive call doing nothing. Boo not cool bro.

We need to do the same thing for each of these instructions. I will go through one more and leave
the rest as an exercise for the reader. Going through each would take too much time.

Pull the `cancel_wedding` function and the related context, `CancelWedding` into its own file
at `programs/crypto-wedding-program/src/instructions/cancel_wedding.rs`

```rust
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
```

Note that we had to bring in `AccountsClose` which is a trait that is needed in order to call
close on an account, which we are doing in this function. More on rust traits [here](https://doc.rust-lang.org/book/ch10-02-traits.html)

We then need to update `programs/crypto-wedding-program/src/instructions/mod.rs` like so:

```rust
pub mod cancel_wedding;
pub mod setup_wedding;
pub use cancel_wedding::*;
pub use setup_wedding::*;
```

You will should to do this for each of your functions that you break out.

Let's go back to our `lib.rs` file and re-introduce our function.

```rust
pub fn cancel_wedding(ctx: Context<CancelWedding>) -> Result<()> {
    instructions::cancel_wedding(ctx)
}

```

Same thing as last time. The rest of the functions I will leave to you to break out. If you
get stuck the final code can be found in this repo as guidance.

Make sure to use `anchor build` along the way to make sure that everything still works.

When you are finished run the tests using `anchor test` to make sure nothing is broken. Remember
to change `Anchor.toml` to use `localnet` again when running tests.

## Summary

That's it! You now have a well structured program that you have deployed onto devnet!

Take everything in this tutorial with a grain of salt. I am not a professional in the Solana
space and am also learning. Good luck!
