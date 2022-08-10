# step 2

## Creating our first function
The first thing we need to do is setup a place to store our wedding and partner data.
```rust
use anchor_lang::{error_code, prelude::*};
use std::cmp::Ordering;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod crypto_wedding_program {
    use super::*;

    pub fn setup_wedding(ctx: Context<SetupWedding>) -> Result<()> {
        let wedding = &mut ctx.accounts.wedding;

        // TODO: ensure that partners cannot start a wedding if they are already in a wedding

        wedding.status = Status::Created;
        wedding.creator = *ctx.accounts.creator.key;
        wedding.partner0 = *ctx.accounts.partner0.key;
        wedding.partner1 = *ctx.accounts.partner1.key;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetupWedding<'info> {
    #[account(mut)]
    pub creator: Signer<'info>, // creator can be someone other than the two partners
    /// CHECK: we only use this to compute PDAs
    pub user_partner0: AccountInfo<'info>,
    /// CHECK: we only use this to compute PDAs
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

fn sort_pubkeys<'a>(pubkey_a: &'a Pubkey, pubkey_b: &'a Pubkey) -> (&'a Pubkey, &'a Pubkey) {
    match pubkey_a.cmp(pubkey_b) {
        Ordering::Less => (pubkey_a, pubkey_b),
        Ordering::Greater => (pubkey_b, pubkey_a),
        Ordering::Equal => (pubkey_a, pubkey_b),
    }
}

#[account]
pub struct Wedding {
    pub creator: Pubkey,  // this can be one of the partners or someone else
    pub partner0: Pubkey, // pubkey is a user derived PDA, not a user account
    pub partner1: Pubkey, // pubkey is a user derived PDA , not a user account
    pub status: Status,
}

impl Wedding {
    pub fn space() -> usize {
        // discriminator + 3 * pubkey + enum
        8 + (32 * 3) + 2
    }

    // we need this in order to ensure that the ordering in wedding seeds remains constant
    pub fn seed_partner0<'a>(pubkey_a: &'a Pubkey, pubkey_b: &'a Pubkey) -> &'a Pubkey {
        let (pubkey0, _) = sort_pubkeys(pubkey_a, pubkey_b);

        pubkey0
    }

    // we need this in order to ensure that the ordering in wedding seeds remains constant
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
```
## The start of setup_wedding

Oof... that's a lot of code with probably a lot of new concepts... lets start by checking out the `setup_wedding` function first.
```rust
pub fn setup_wedding(ctx: Context<SetupWedding>) -> Result<()> {
    //...
}
```
First off we are using a new type for `ctx`: `Context<SetupWedding>`. This gives us access to the related accounts that we pass into the function call. We will check the `SetupWedding` struct shortly.
Let's skip the body of the function for now and try looking at what `SetupWedding` is doing...

## SetupWedding

```rust
#[derive(Accounts)]
pub struct SetupWedding<'info> {
    #[account(mut)]
    pub creator: Signer<'info>, // creator can be someone other than the two partners
    /// CHECK: we only use this to compute PDAs
    pub user_partner0: AccountInfo<'info>,
    /// CHECK: we only use this to compute PDAs
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
So... there's a lot of macro magic happening here... 
`#[derive(Accounts)]` creates the extra code needed to use this as part of our `ctx` argument. This will enable us to pass in, create, and check accounts that are being passed into our function.
Without the macros, we can see that we have the following fields: `creator`, `user_partner0`, `user_partner1`, `wedding`, `partner0`, `partner1`, and `system_program`. Why are all of these accounts
being passed in? In solana you must pass in all of the accounts that you intend to interact with. This is Solana's way of gaining higher performance. The solana client can process multiple 
transactions in parallel if it knows that the accounts are not being touched by other transactions. 

But what are these `#[account()]` macros?

## Anchor account consntraints

In Anchor, you are able to set what are called constraints. These do different things depending on what is passed into the `#[account]` macro. Let's look at the macro for the `wedding` field.
```rust
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
```

First off we have `init`. Tells our program that we expect `wedding` to be an uninitialized account and that we are going to create it in our transaction. 

`payer` does what you probably think it does... it set's the payer to the `creator` account that was listed above. 

`space` is satisfying another requirement related to solana programs. We need to know how much data we are saving to the account ahead of time. We can call our space function found in our Wedding 
functions found in `impl Wedding{}`. You can find more about how to determinte what the space is for different types [here](https://www.anchor-lang.com/docs/space).

We then have `seeds`. This allows us to deterministically create a program derived account (PDA). Let's Thinkn about it this way... if we are creating a program where we want to have many
people getting married using this single program, each of them are going to need a unique account where the wedding data is stored. We can ensure this by including "wedding", which is basically our 
use case, as well as the two public keys of the users who are getting married. However, we want to make it easy to be able to recreate these accounts from outside. This means that we are going
to need to find a way to ensure that the order of the users passed in is always the same. We can do this by sorting the public keys from least to greatest (public keys are basically just really
big numbers when you think about it). So, to summarize: we are ensuring that we are creating a unique account to store wedding data for each couple which is tied to our wedding program.

`bump` just tells Anchor to automatically handle the bump to use. Let's leave it at that for now...

But what is a PDA?!

## Program Derived Accounts
Bear with me, we are almost through the tough stuff. Everything is downhill after this section. The hardest stuff comes first... that's how you should write tutorials right? :)

This blog entry does a really great job of explaining many concepts. There is a [section here](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/#pdas-part-2) which epxlains PDAs 
well. 

If you are too lazy to click, in a nutshell, we can say that a PDA is a special account which has no private key. This means that no regular user can use this account. It is an account that can 
only be created from a program. You can sort of think about it as a way for the program to own and control storage related to it. When you use certain seeds, you can then recreate the public
key that you need to pass into function calls to the program. It also means that you know where certain storage is because you know how to create the pub key from the seeds. The important thing here
is that we are certain that there never can be a private key associated... you don't want to have an account owned by a program suddenly becoming controllable by some random user 
(even if the odds of that happening are extremely low). The `bump` mentioned in the section before is a way of "pushing" the account off of the `ed25519` elliptic curve.

Back to our `SetupWedding` constraints...

## Anchor constraints part 2
We have 2 more places where the `#[account]` macro is being used: `partner0` and `partner1`:

```rust
#[account(
    seeds = [
        b"partner",
        user_partner0.key().as_ref(),
    ],
    bump,
)]
/// CHECK: we are doing all needed checks manually on this account
pub partner0: UncheckedAccount<'info>,

```

After that whole bit about `wedding`, this should be pretty easy...

We again see `seeds` but this time we are passing in "partner" which means we are using this for partner storage (our own choice of naming), and we are passing in the user's public key. This means
that we are creating a unique PDA for each partner where we can store some data. `bump` has the same meaning as above in the `wedding` constraints.

It is worth noting that we are not using `init`, `payer`, or `space`. We are not trying to create the accounts here. We just want access to them. This relates to this line as well:
```rust
/// CHECK: we are doing all needed checks manually on this account
```

This is a requirement by Anchor. The program will not compile when you are using generalized types such as `UncheckedAccount` unless you add this line... basically it is supposed to make you
be very sure that you are not passing in a specific type for a very good reason. Our explanation will come into play later...

## system_program
`system_program` is our last field. We need to pass this as an account because `system_program` is called when we are creating an account (our `wedding` PDA).

At this point we have gone through the keys and constraints in the `SetupWedding` struct. But what about the values?!

## Checked account types
Let's go through each field one more time:

```rust
pub creator: Signer<'info>, // creator can be someone other than the two partners
```
Here, we have the `Signer<'info>` type. If you know rust then you know `'info` this is a lifetime annotation. If you don't know about it let's not worry about it at this moment. However, the 
`Signer` type means that this is the account that is signing the transaction. Meaning this is likely the person sending the transaction and paying for it.

```rust
pub user_partner1: AccountInfo<'info>,
```

Next, we have `AccountInfo`, this is a basic account type where we expect to have an initialized account.
We can use this to get basic data about an account such as data, the account balance, and othe basic information.

```rust
pub wedding: Account<'info, Wedding>,
```

This is a bit more interesting. This is a custom type that we are defining ourselves.
```rust
#[account]
pub struct Wedding {
    pub creator: Pubkey,  // this can be one of the partners or someone else
    pub partner0: Pubkey, // pubkey is a user derived PDA, not a user account
    pub partner1: Pubkey, // pubkey is a user derived PDA , not a user account
    pub status: Status,
}
```

Basically what we are saying here is that the `wedding` account that we are passing in
contains the fields we defined above. This is where we are storing our wedding data 
for a given wedding.

Let's go back and look at our `setup_wedding` function that we started with:

## Back to setup_wedding

```rust
pub fn setup_wedding(ctx: Context<SetupWedding>) -> Result<()> {
    let wedding = &mut ctx.accounts.wedding;

    // TODO: ensure that partners cannot start a wedding if they are already in a wedding

    wedding.status = Status::Created;
    wedding.creator = *ctx.accounts.creator.key;
    wedding.partner0 = *ctx.accounts.partner0.key;
    wedding.partner1 = *ctx.accounts.partner1.key;

    Ok(())
}
```

We can now see that we are accessing the wedding storage account that was passed in. 
Remember that we created the account through the Anchor constraint `init`. We are then setting
the `status`, `creator`, `partner0`, and `partner1` fields that we defined earlier.

Lastly, we return `Ok(())` because the function must return a `Result` type (rust things).

## A summary of what we just did...

So let's summarize what all of this code just did...

We created a new function called `setup_wedding`. This function uses an Anchor derived 
generic type `Context` which uses `SetupWedding`. That `SetupWedding` type defines which
accounts we are passing in, what types they are, what data they contain, ensures their
 public keys are what we expect them to be, and creates the `Wedding` storage account for 
 our two partners. In the function body, we set the `Wedding` storage and retun `Ok(())`.

 Whew... that was quite a bit... everything after this should be easier. Congrats on making
 it to this point. Go have a beer or something and then try the next step. :)
