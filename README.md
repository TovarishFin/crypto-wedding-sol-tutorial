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
it to this point.

## One last check...

There is one more thing that we want to add in here...

We want to make sure that a user has not created their `Partner` account yet. We want to be
sure that the `partner0` and `partner1` accounts we pass in are empty. This is because we
want to make sure that these partners are not part of another `Wedding`. We can do this with
following code:

```rust
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
```

Here we are passing in an `UncheckedAccount` to make sure that we are passing in an
uninitialized account. We check this by ensuring that there is no data on the account and
that the balance is 0. If either of these checks fail we are going to return an error
which will cause the transaction to fail and exit early.

and we can use it here:

```rust
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
```

We also need to setup the errors that we were returning in `validate_partner`:

```rust
#[error_code]
pub enum WeddingError {
    #[msg("partner data not empty")]
    PartnerDataNotEmpty,
    #[msg("partner lamports not zero")]
    PartnerBalanceNotZero,
}
```

In the above code, we are using another "magic" macro to setup an Anchor error. The `#[error_code]`
macro sets up the code needed to make our `WeddingError` enum into something our program
can use and return in our `Result` type.

The `#[msg]` macro allows us to return an error message. We will add to this throughout the
tutorial for different error cases.

## testing

We now are going to write a typescript based test in order to see that our program works as
expected.

### testing helpers

Let's start with creating a new file named `tests/helpers.ts`. We are going to start
with building a few utility functions that we are going to commonly need to write our tests...

```typescript
import * as anchor from "@project-serum/anchor";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { PublicKey } from "@solana/web3.js";
import { BN } from "bn.js";

export const WeddingCreated = { created: {} };
export const WeddingMarrying = { marrying: {} };
export const WeddingMarried = { married: {} };
export const WeddingDivorcing = { divorcing: {} };
export const WeddingDivorced = { divorced: {} };

export const addFunds = async (
  provider: anchor.Provider,
  user: anchor.web3.PublicKey,
  amount: number
): Promise<void> => {
  const [airdropTxHash, { blockhash, lastValidBlockHeight }] =
    await Promise.all([
      provider.connection.requestAirdrop(user, amount * LAMPORTS_PER_SOL),
      provider.connection.getLatestBlockhash(),
    ]);

  await provider.connection.confirmTransaction({
    signature: airdropTxHash,
    blockhash,
    lastValidBlockHeight,
  });

  const balance = await provider.connection.getBalance(user);
  console.log(
    `airdropped ${amount} SOL to ${user.toBase58()} | new balance: ${
      balance / LAMPORTS_PER_SOL
    } SOL`
  );
};

export const sortPubKeys = (
  publicKeyA: PublicKey,
  publicKeyB: PublicKey
): PublicKey[] => {
  const a = new BN(publicKeyA.toBytes());
  const b = new BN(publicKeyB.toBytes());
  let sorted =
    a.cmp(b) == -1 ? [publicKeyA, publicKeyB] : [publicKeyB, publicKeyA];

  return sorted;
};

export const generateWeddingPDA = async (
  eCryptoWedding: PublicKey,
  uPartner0: PublicKey,
  uPartner1: PublicKey
): Promise<PublicKey> => {
  const sorted = sortPubKeys(uPartner0, uPartner1);
  const [pWedding, _] = await PublicKey.findProgramAddress(
    [
      anchor.utils.bytes.utf8.encode("wedding"),
      ...sorted.map((x) => x.toBuffer()),
    ],
    eCryptoWedding
  );

  return pWedding;
};

export const generatePartnerPDA = async (
  eCryptoWedding: PublicKey,
  partner: PublicKey
): Promise<PublicKey> => {
  const [pPartner, _] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("partner"), partner.toBuffer()],
    eCryptoWedding
  );

  return pPartner;
};
```

The `Wedding*` empty-ish objects are representations of our rust enum `Status`. This is how
it is returned to us, so it is easier to pre-define them here to match against.

Our `addFunds` function is going to make it easy for us to get SOL for our new accounts
that we are going to use for our tests. We are going to need at least two users to complete
a wedding.

Most of this is pretty standard typescript. The `Provider` type is essentially our connection
to our local blockchain that we are running tests on. So we are basically using our connection
to request funds for our account `PublicKey` that we pass in, getting the lastest block, and
using this information to wait for the transaction to complete. We then check the balance
of the acount and log it out.

`sortPubKeys` is doing something very similar to what we were doing in our program in rust.
We are putting the smaller `Pubkey` first and the larger one second. This is used in the
next function. Remember we need to do this in order to ensure that we have the same order
every time in order to ensure that our seed is deterministic.

`generateWeddingPDA` uses the same seeds that we had in our rust program. We need to turn each
part of the seed into a `Uint8Array` or `Buffer` because it expects a byte array. It is the
same in our program, but anchor handles some of this for us in the background. We then use
a function provided to use from the `@solana/webs.js` library to find our PDA. This should/must
match what we are generating in our program.

`generatePartnerPDA` does the same thing as `generateWeddingPDA` but for our `Partner` accounts.
Not much else to say here...

With that out of the way, we can move on to our actual test file and do some final setup...

### test setup and testing

Here is the full test file:

```typescript
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { CryptoWeddingProgram as CryptoWedding } from "../target/types/crypto_wedding_program";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import {
  addFunds,
  WeddingCreated,
  generateWeddingPDA,
  generatePartnerPDA,
} from "./helpers";

describe("when using CryptoWeddingProgram", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  // e for executable
  const eCryptoWedding = anchor.workspace
    .CryptoWedding as Program<CryptoWedding>;
  const uCreator = anchor.web3.Keypair.generate();
  // u for user
  const uPartner0 = anchor.web3.Keypair.generate();
  // u for user
  const uPartner1 = anchor.web3.Keypair.generate();

  let pWedding: PublicKey;
  let pPartner0: PublicKey;
  let pPartner1: PublicKey;

  before("setup", async () => {
    pWedding = await generateWeddingPDA(
      eCryptoWedding.programId,
      uPartner0.publicKey,
      uPartner1.publicKey
    );

    pPartner0 = await generatePartnerPDA(
      eCryptoWedding.programId,
      uPartner0.publicKey
    );

    pPartner1 = await generatePartnerPDA(
      eCryptoWedding.programId,
      uPartner1.publicKey
    );

    // need to add funds to each new account we created
    await Promise.all([
      addFunds(provider, uCreator.publicKey, 100),
      addFunds(provider, uPartner0.publicKey, 100),
      addFunds(provider, uPartner1.publicKey, 100),
    ]);
  });

  it("should setup a wedding as a non-partner (creator)", async () => {
    try {
      await eCryptoWedding.methods
        .setupWedding()
        .accounts({
          creator: uCreator.publicKey,
          userPartner0: uPartner0.publicKey,
          userPartner1: uPartner1.publicKey,
          partner0: pPartner0,
          partner1: pPartner1,
          wedding: pWedding,
        })
        .signers([uCreator])
        .rpc();
    } catch (err) {
      console.error(err);
      console.log(err.programErrorStack[0].toBase58());
      throw new Error(err);
    }

    const dWedding = await eCryptoWedding.account.wedding.fetch(pWedding);
    expect(dWedding.partner0).to.eql(pPartner0);
    expect(dWedding.partner1).to.eql(pPartner1);
    expect(dWedding.status).to.eql(WeddingCreated);
  });
});
```

Let's start with the setup.

```typescript
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
```

Here we are settig up our connection to our local blockchain. And having anchor use it.

```typscript
// e for executable
const eCryptoWedding = anchor.workspace
  .CryptoWedding as Program<CryptoWedding>;
const uCreator = anchor.web3.Keypair.generate();
// u for user
const uPartner0 = anchor.web3.Keypair.generate();
// u for user
const uPartner1 = anchor.web3.Keypair.generate();
```

Anchor does a bit of magic here where we are grabbing `CryptoWedding` from something called
`anchor.workspace`. We have to give it a type because typescript doesn't know about it. The
type is generated automatically when we call `anchor build` from our terminal.

After, we are calling `Keypair.generate()`. This creates a new keypair which we can use to
represent different users in our tests.

Our `before` block generates the PDA public keys for each of our partners and our wedding.
To be clear we have not created the storage here... but we need to generate these ahead of
time because of the way solana works. Remember that we need to tell solana about each
account that is going to be touched or read in our transactions. This includes the PDAs
which have not yet been created that will be created in our tx.

Lastly, we are calling `addFunds` to add some SOL to each of our newly created accounts.

We do not need to deploy our `CryptoWeddingProgram`. Anchor handles this automagically
for us in our tests.

### And finally the tests...

So we now get to the meat of our test...

```typescript
it("should setup a wedding as a non-partner (creator)", async () => {
  try {
    await eCryptoWedding.methods
      .setupWedding()
      .accounts({
        creator: uCreator.publicKey,
        userPartner0: uPartner0.publicKey,
        userPartner1: uPartner1.publicKey,
        partner0: pPartner0,
        partner1: pPartner1,
        wedding: pWedding,
      })
      .signers([uCreator])
      .rpc();
  } catch (err) {
    console.error(err);
    console.log(err.programErrorStack[0].toBase58());
    throw new Error(err);
  }

  const dWedding = await eCryptoWedding.account.wedding.fetch(pWedding);
  expect(dWedding.partner0).to.eql(pPartner0);
  expect(dWedding.partner1).to.eql(pPartner1);
  expect(dWedding.status).to.eql(WeddingCreated);
});
```

So our first call is to our program finally! We see here why we went through all of that
trouble to generate all of these PDAs here. As already said, we need to pass in all used
accounts ahead of time to our function. Luckily our type will warn us if we are not passing
in the right account or missing something. We add in the `uCreator` `Keypair` as our signer
and off we go...

We then check the weding data via this line:

```typescript
const dWedding = await eCryptoWedding.account.wedding.fetch(pWedding);
```

We make sure that everything matches what we expect and we are good to go!

### A not about variable names

Because accounts can have so many different roles and hidden meanings... I try to prepend names
with a relevant letter. `uPartner0` for user, `pPartner` for PDA, `eCryptoWedding` for executable,
`dWedding` for data etc.
