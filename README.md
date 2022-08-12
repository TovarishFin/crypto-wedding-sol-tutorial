# step 5

In this step, we will implement that last two needed functions, `give_answer` and `divorce`.
Of course we are also going to write tests for them to see them in action as well. Let's start
with `give_answer`...

## give_answer placeholder

Let's start with our placeholder again:

```rust
pub fn give_answer(ctx: Context<GiveAnswer>, answer: bool) -> Result<()> {
    Ok(())
}
```

Nothing new here... we have seen function parameters as well in `setup_partner`.
On to the `GiveAnswer` context struct...

## GiveAnswer context struct

```rust
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
```

Again, these are all things we have seen before... let's just try to summarize what we are doing
here:

We have `user` which can be either of the partners of a wedding. We are checking this by using
`seeds` to ensure they have a `Partner` storage account that is tied to a `Wedding` storage
account. We make sure that the user calling is a member of the `Wedding` storage account through
the `has_one` constraint. Passing in the wedding and each partner PDA into our context ensures
that they exist and that they are the correct account type.

So with this layout we can ensure that the user calling has a partner PDA and that that partner PDA
is tied to the wedding PDA they are trying to interact with.

Hopefully this is all starting to make sense to you.

Let's move on to the function body:

```rust
pub fn give_answer(ctx: Context<GiveAnswer>, answer: bool) -> Result<()> {
    let partner = &mut ctx.accounts.partner;
    let other_partner = &mut ctx.accounts.other_partner;
    let wedding = &mut ctx.accounts.wedding;

    // update partner's answer no matter what as long as its in the right status
    partner.answer = answer;

    match wedding.status {
        // if wedding status is created...
        Status::Created => match answer {
            // if answer passed in is true...
            true => {
                wedding.status = Status::Marrying;
                Ok(())
            }
            // if answer passed in is false... do nothing
            false => Ok(()),
        },
        // if wedding status is marrying...
        Status::Marrying => match (answer, other_partner.answer) {
            // if both user answer and other partner's answer is true...
            (true, true) => {
                // set wedding status to married
                wedding.status = Status::Married;
                Ok(())
            }
            // for any other case, do nothing...
            (_, _) => Ok(()),
        },
        // if the wedding status is not created or marrying return an error
        _ => return Err(WeddingError::InvalidAnswerStatus.into()),
    }
}
```

First off we can see that we are updating the `Partner` storage to whatever the user passed in.
We then use `match` which is something like switch in many other languages but quite a bit more
powerful. [Read here for more info on matche](https://doc.rust-lang.org/book/ch06-02-match.html).

I have added comments in every part of the match statement in order to try to add clarity
for those that don't know how `match` statements work...

Basically we are saying we only want to allow this function to run if the wedding `status` is
in `Created` or `Marrying`. We set the partner `answer` field immediately **but**, if we return
an error then that never happened... transactions are atomic... meaning they either run completely
or not at all. We check `Created` status and update it to `Marrying` if the user `answer` is `true`.
We update the wedding `status` to `Married` if both `Partner` storage accounts have an `answer`
of yes.

## Another error

```rust
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
}
```

Add the missing error as so...

Alright let's move on to a test...

## testing give_answer

Run `anchor build` first as usual...

Add the following outside and after the origial `describe` block.

```typescript
describe("when using CryptoWeddingProgram through it's full lifecycle", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  // e for executable
  const eCryptoWedding = anchor.workspace
    .CryptoWeddingProgram as Program<CryptoWedding>;
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
      addFunds(provider, uPartner0.publicKey, 100),
      addFunds(provider, uPartner1.publicKey, 100),
    ]);
  });

  it("should setup a wedding as partner0", async () => {
    try {
      await eCryptoWedding.methods
        .setupWedding()
        .accounts({
          creator: uPartner0.publicKey,
          userPartner0: uPartner0.publicKey,
          userPartner1: uPartner1.publicKey,
          partner0: pPartner0,
          partner1: pPartner1,
          wedding: pWedding,
        })
        .signers([uPartner0])
        .rpc();
    } catch (err) {
      console.error(err);
      console.log(err.programErrorStack[0].toBase58());
      throw new Error(err);
    }

    const dWedding = await eCryptoWedding.account.wedding.fetch(pWedding);
    expect(dWedding.creator).to.eql(uPartner0.publicKey);
    expect(dWedding.partner0).to.eql(pPartner0);
    expect(dWedding.partner1).to.eql(pPartner1);
    expect(dWedding.status).to.eql(WeddingCreated);
  });

  it("should setup partner0 as user0", async () => {
    const pName = "bob";
    const pVows = "stuff";

    try {
      await eCryptoWedding.methods
        .setupPartner(pName, pVows)
        .accounts({
          user: uPartner0.publicKey,
          other: uPartner1.publicKey,
          partner: pPartner0,
          wedding: pWedding,
        })
        .signers([uPartner0])
        .rpc();
    } catch (err) {
      console.error(err);
      throw new Error(err);
    }

    const sPartner0 = await eCryptoWedding.account.partner.fetch(pPartner0);
    expect(sPartner0.wedding).to.eql(pWedding);
    expect(sPartner0.user).to.eql(uPartner0.publicKey);
    expect(sPartner0.name).to.equal(pName);
    expect(sPartner0.vows).to.equal(pVows);
    expect(sPartner0.answer).to.equal(false);
  });

  it("should setup partner1 as user1", async () => {
    const pName = "alice";
    const pVows = "other stuff";

    try {
      await eCryptoWedding.methods
        .setupPartner(pName, pVows)
        .accounts({
          user: uPartner1.publicKey,
          other: uPartner0.publicKey,
          partner: pPartner1,
          wedding: pWedding,
        })
        .signers([uPartner1])
        .rpc();
    } catch (err) {
      console.error(err);
      throw new Error(err);
    }

    const sPartner1 = await eCryptoWedding.account.partner.fetch(pPartner1);
    expect(sPartner1.wedding).to.eql(pWedding);
    expect(sPartner1.user).to.eql(uPartner1.publicKey);
    expect(sPartner1.name).to.equal(pName);
    expect(sPartner1.vows).to.equal(pVows);
    expect(sPartner1.answer).to.equal(false);
  });

  it("should answer yes as user0 and be marrying", async () => {
    try {
      await eCryptoWedding.methods
        .giveAnswer(true)
        .accounts({
          user: uPartner0.publicKey,
          other: uPartner1.publicKey,
          partner: pPartner0,
          otherPartner: pPartner1,
          wedding: pWedding,
        })
        .signers([uPartner0])
        .rpc();
    } catch (err) {
      console.error(err);
      throw new Error(err);
    }

    const sPartner0 = await eCryptoWedding.account.partner.fetch(pPartner0);
    expect(sPartner0.wedding).to.eql(pWedding);
    expect(sPartner0.user).to.eql(uPartner0.publicKey);
    expect(sPartner0.answer).to.equal(true);

    const sWedding = await eCryptoWedding.account.wedding.fetch(pWedding);
    expect(sWedding.status).to.eql(WeddingMarrying);
  });

  it("should answer yes as user1 and be married", async () => {
    try {
      await eCryptoWedding.methods
        .giveAnswer(true)
        .accounts({
          user: uPartner1.publicKey,
          other: uPartner0.publicKey,
          partner: pPartner1,
          otherPartner: pPartner0,
          wedding: pWedding,
        })
        .signers([uPartner1])
        .rpc();
    } catch (err) {
      console.error(err);
      throw new Error(err);
    }

    const sPartner1 = await eCryptoWedding.account.partner.fetch(pPartner1);
    expect(sPartner1.wedding).to.eql(pWedding);
    expect(sPartner1.user).to.eql(uPartner1.publicKey);
    expect(sPartner1.answer).to.equal(true);

    const sWedding = await eCryptoWedding.account.wedding.fetch(pWedding);
    expect(sWedding.status).to.eql(WeddingMarried);
  });
});
```

So that's quite a bit... why are we going through all of this trouble? Well, we need to
have a different branch where we are not cancelling a wedding. In this branch we can also
test our `divorce` function later. There is actually not much new here...

We mostly copied the pervious describe block, changed the `setup_wedding` test to use one of
the `partner`s instead of a `creator` account just for the sake of testing another area while we
are at it. We then added another test where `partner1` also sets up their `partner` storage PDA.
Lastly, we test each of the `partner`s saying yes by giving an `answer` of `true`.

We fetch the state for the `partner` and associated `wedding` storage accounts and check that
it is what we expect.

Go ahead and run `anchor test`. Wow look at that 10 passing tests! Such professionalism, much wow.

## onto divorce

Sadly, it makes sense to add a divorce function. Let's go ahead and setup another placeholder.

```rust
pub fn divorce(ctx: Context<Divorce>) -> Result<()> {
    Ok(())
}
```

And on to our `Divorce` context...

## Divorce context

```rust
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
```

This looks identical to our `GiveAnswer`context struct. And when you think about it we are
trying to enforce the same thing. We only want a partner of a given wedding to be able to divorce.

Let's implement our function...

## divorce implementation

```rust
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
```

Again, looks very much like our `give_answer` function but we are doing the opposite.
We set the user's `Partner` storage account `answer` field to `false` and then make our checks.
If the `Wedding` storage account has a `status` of `Married` we change it to `Divorcing`.
If the other `Partner` storage account also has an `answer` of `false`, we close the `Wedding`
storage account and give the `rent` back to the `creator`.

Let's add our last error that is missing...

## Again with the errors...

```rust
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

## Our last tests...

Run `Anchor build` as usual...

We can finally add our last tests to see how a divorce should look like...

```typescript
it("should divorce as user0 and be divorcing", async () => {
  try {
    await eCryptoWedding.methods
      .divorce()
      .accounts({
        user: uPartner0.publicKey,
        other: uPartner1.publicKey,
        creator: uPartner0.publicKey,
        partner: pPartner0,
        otherPartner: pPartner1,
        wedding: pWedding,
      })
      .signers([uPartner0])
      .rpc();
  } catch (err) {
    console.error(err);
    throw new Error(err);
  }

  const sPartner0 = await eCryptoWedding.account.partner.fetch(pPartner0);
  expect(sPartner0.wedding).to.eql(pWedding);
  expect(sPartner0.user).to.eql(uPartner0.publicKey);
  expect(sPartner0.answer).to.equal(false);

  const sWedding = await eCryptoWedding.account.wedding.fetch(pWedding);
  expect(sWedding.status).to.eql(WeddingDivorcing);
});

it("should divorce as user1 and be divorced", async () => {
  try {
    await eCryptoWedding.methods
      .divorce()
      .accounts({
        user: uPartner1.publicKey,
        other: uPartner0.publicKey,
        creator: uPartner0.publicKey,
        partner: pPartner1,
        otherPartner: pPartner0,
        wedding: pWedding,
      })
      .signers([uPartner1])
      .rpc();
  } catch (err) {
    console.error(err);
    throw new Error(err);
  }

  const sPartner1 = await eCryptoWedding.account.partner.fetch(pPartner1);
  expect(sPartner1.wedding).to.eql(pWedding);
  expect(sPartner1.user).to.eql(uPartner1.publicKey);
  expect(sPartner1.answer).to.equal(false);

  try {
    await eCryptoWedding.account.wedding.fetch(pWedding);
    expect.fail("pWedding should no longer contain sWedding");
  } catch (err) {
    expect(err.message).to.contain("Account does not exist");
  }
});
```

This should all be pretty routine here now... we see `user0` `divorce` and expect the
`wedding` and `partner` storage accounts to match what we want.

We then call `divorce` as `user1` and check `partner` state and check that `wedding` no longer
exists.

## Summary

At this point most of what we are doing is hopefully starting to seem pretty routine. We
implemented a way for users to give their answer about marrying and also divorce after.
Note that users still would need to call `close_partner` after divorce. This could be done
automatically in the `divorce` function, but we are trying to get through this tutorial in
a reasonable amount of time :) . There are many other holes in this program but we are using
this more as an exercise to learn than anything else. There are additional functions implemented
in the [repo itself](https://github.com/TovarishFin/crypto-wedding-sol). As an exercise to the
reader, perhaps think about how you would implement these functions. We could add in `update_name`,
to manually update the `name` of a `Partner` storage account. Same goes for `vows`. If you are
feeling bold, you could even try to implement all of the account closing functionality in
the `divorce` function itself.

It is also important to note that in Solana, transactions can contain many calls to a contract.
So perhaps keeping things seperate is fine? This allows more granular control for those building
a client. They could, in theory, include the `divorce` and `close_partner` calls in the same
transaction. I'm not sure what is best. I am not an expert :) .

In our next step we are going to deploy our program to the `devnet` cluster. We also will
take a look at cleaning up our code. Everything for our program is in one file and is starting
to feel a bit cluttered and gross,
