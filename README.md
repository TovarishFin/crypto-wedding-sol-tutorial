# step 3

In this step we will continue building more functions. We will be focusing on enabling
users to cancel a wedding. This can be useful for many different scenarios. We might need to
cancel if there was a mistake made, someone created a wedding for two people that don't
want to get married, or someone changes their mind :)

## Cancelling a Wedding

Let's start by making a placeholder function.

```rust
pub fn cancel_wedding(ctx: Context<CancelWedding>) -> Result<()> {
    Ok(())
}
```

## CancelWedding context

Next, let's define our `CancelWedding`struct.

```rust
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

This should mostly look familiar to you after our `setup_wedding` function. There are, however,
a few new constraints. Let's get into the accounts and these new constraints.

`user` is the person signing the transaction. In our case, it can either be one of the partners
or it could be someone else who created the `Wedding`.

`creator`, `user_partner0`, and `user_partner1` have the type of `AccountInfo`. We see the
`CHECK` above each again because we are using a basic type. We are using the basic type
because we are only using it to compute the wedding PDA.

`creator` and `user` are both marked with `#[account(mut)]` to declare that these accounts
will have some changes made to them (mutable). `user` is mutable because we are signing the
tx with this account which means that some balance will be deducted for calling this program.
User's familiar with Ethereum would call this gas costs. `creator` is marked as mutable because
When we close the `Wedding` storage account, the rent costs will be refunded. We are going
to send this balance back to the `creator` who originally created this storage account.

Lastly we have `wedding`. It is also marked as `mut` because we intend to close this account.
That sounds like a mutation to me. `seeds` is the same as our last function. When `seeds` is
used without `init`, it is a way to ensure that the address we are passing in as wedding is
the address that we expect. We expect the address to match the result of these `seeds`. `bump`
is the same as last time, have Anchor automagically choose our bump. `has_one` is a new one...

`has_one` ensure that our `Wedding` storage has a field named `creator` which matches our
`creator` account passed into `CancelWedding`. `@` allows us to return a custom error if there
is no match. So here we are returning a new error enum `WeddingError::InvalidCreator`.

Speaking of that errors... let's quickly add all the enum variants that we will use in this step:

## More errors...

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
}
```

These seem pretty self explanatory to me so I will leave it at that.

## cancel_wedding implementation

Alright, we finally have our `CancelWedding` context type ready for our `cancel_wedding`
function. Let's go ahead and implement it.

```rust
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
```

The first function we are calling is a function very similar to what we were calling from
typescript in the previous step. We are generating a partner PDA address (if the user has one at all).
We can then check this against the `partner0` or `partner1` PDAS. If either of them match, then
the user is a partner of the `Wedding` and is allowed to cancel. We also check the `user` public key
to see if it matches the `creator` set in the `Wedding` storage account. We then use the
`require!()` macro to automagically return an error if the user calling this function is neither
a `partner` nor a `creator`. `WeddingError::NotWeddingMember` is an error we defined earlier.

We then check the `Wedding` storage account to see if it is in a status where it makes sense
to cancel. If it doesn't make sense, we return another error that we defined earlier: `WeddingError::CannotCancel`.

Lastly, if all of our checks passed, we call `close` on the `wedding` account. We pass in `creator`
as an argument which tells the function to send the rent refund to `creator`.

We then return `Ok(())` which is a `Result` type.

Note that our check against our `user` account and our `Status` are functionally similar.
`require!()` is just a prettier way of doing it. Both work and are fine!

## Summary

That's it for the implementation of our `cancel_wedding` function... a lot easier than `setup_wedding` right?
It's all downhill from here :)

In this function we allow one of the `partner`s and/or the `creator` to cancel their `Wedding`.
We check that the person calling cancel is one of these users. We also refund the `rent` to
the creator. Easy.

## Testing

First off, we need to update our program and the resulting typescript files by running:

```sh
anchor build
```

Here is our test:

```typescript
it("should cancel a wedding", async () => {
  try {
    await eCryptoWedding.methods
      .cancelWedding()
      .accounts({
        user: uPartner0.publicKey,
        creator: uCreator.publicKey,
        userPartner0: uPartner0.publicKey,
        userPartner1: uPartner1.publicKey,
        wedding: pWedding,
      })
      .signers([uPartner0])
      .rpc();

    try {
      await eCryptoWedding.account.wedding.fetch(pWedding);
      throw new Error("pWedding should not exist");
    } catch (err) {
      expect(String(err)).to.include("Account does not exist");
    }
  } catch (err) {
    console.error(err);
    throw new Error(err);
  }
});
```

This again should feel pretty similar to our previous test. We call our new function and pass
in the required accounts, including the `pWedding` which is the PDA we computed earlier.

We sign as `partner0`. We then try to get the data for our `Wedding` storage account which **used to exist**
at the public key `pWedding`. If this does not cause an error we fail the test. If it does fail,
we make sure that it is the right kind of error. That error is one telling us that the
account no longer exists.

That's it! go ahead and run `anchor test`. Both of your tests should be nice and green :)

In our next step, we will move on to setting up the storage accounts for each of our `partner`s.
