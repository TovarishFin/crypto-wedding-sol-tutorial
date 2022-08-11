# step 4

In this step we will implement two functions. one called `setup_partner`, and another called
`close_partner`. `setup_partner` will enable a user to create a storage account tied to
our program where they can save data such as their `name`, `vows`, and their `answer` (whether or not they agree to marry).

## setup_partner placeholder

Like last time, lets setup a placeholder function and work our way from there...

```rust
pub fn setup_partner(ctx: Context<SetupPartner>, name: String, vows: String) -> Result<()> {
    Ok(())
}
```

In the above code, we finally see some arguments to a function other than our `ctx`. Nice.

Let's change the order a bit this time and think about what we want to store...

## Partner storage account

```rust
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

We add the `#[account]` macro again same as `Wedding` before to make this useable as an
account where we want to store things.

As said earlier, we are going to store, `name`, `vows`, and an `answer`. We probably also want
to tie a `Partner` storage account to a `Wedding` storage account. So let's also save `wedding`
as a field. Let's also just save the `user`s pubkey here for clarity as well.

Lastly, and again like `Wedding`, we add an associated function where we calculate the space.
It is more useful and even necessary here because we need to calculate the dynamic space of
`String` types. This function will be used in our constraints when creating the account.

Let's go ahead and look at our `SetupPartner` context struct.

```rust
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
```

Everything here should be somewhat familiar to you from our previous steps. The only strange thing
here should be:

```rust
#[instruction(name: String, vows: String)]
```

This macro is giving us access to our function arguments we defined in the beginning of the
section. Why do we need this?

Here is your answer:

```rust
space = Partner::space(&name, &vows),
```

In our partner constraint we need to tell the anchor how big the account is going to be before
creating it. In order to know how big it is we need our earlier defined function to have
access to these string arguments.

So summarizing what is going on here:

We have a user which should be either `partner0` or `partner1` We have `other` which should be
the other partner. We have our `partner` field which is going to be a storage account of type
`Partner`. It is going to be created in this transaction and will be paid for by the `user` account.
We pass in the `other` account in order to compute the `Wedding` storage acccount PDA. By passing
in `wedding` with the seeds, we are ensuring that the wedding exists. Which means that user's need
to wait for a wedding to be created before calling `setup_partner`. Because `init` is added for
`partner` we also ensure that the account doesn't exist yet. A user cannot create multiple
`partner` storage accounts because of the seeds we are using.

Alright... let's implement the function

## Implementing setup_partner

Because we are letting Anchor take care of most of our constraints, this function is pretty
simple after we finally get to the actual implementation:

```rust
/// sets up partner storage account for a user. Gets everything ready short of the answer.
pub fn setup_partner(ctx: Context<SetupPartner>, name: String, vows: String) -> Result<()> {
    let partner = &mut ctx.accounts.partner;
    partner.wedding = ctx.accounts.wedding.key();
    partner.user = ctx.accounts.user.key();
    partner.name = name;
    partner.vows = vows;

    Ok(())
}
```

All we are doing here is setting the storage that we defined earlier.

Onto the tests...

## Testing setup_partner

Remember to run `anchor build` before getting into the tests to get access to our new function.

Go ahead and open up `tests/crypto-wedding-program.ts` and add this test block to the very
start of the tests (before the original wedding test).

```typescript
it("should NOT setupPartner when no wedding PDA", async () => {
  try {
    await eCryptoWedding.methods
      .setupPartner("bob", "stuff")
      .accounts({
        user: uPartner0.publicKey,
        other: uPartner1.publicKey,
        partner: pPartner0,
        wedding: pWedding,
      })
      .signers([uPartner0])
      .rpc();
    expect.fail("setupPartner should fail before a pWedding is created");
  } catch (err) {
    expect(String(err)).to.contain("Error Code: AccountNotInitialized.");
  }
});
```

We are again calling our new method `setupPartner` but with arguments this time. We then pass
in our related accounts like we have done each time. We call as `uPartner0` and still expect
it to fail because no `Wedding` storage exists yet... AKA "AccountNotInitialized".

Add this next test block between the **setup wedding** and **cancel wedding** blocks:

```typescript
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
```

Here we expect the test to pass calling it in the same way. The only thing that is different
here is that we have alrelady created the `Wedding` storage account. It should succeed,
the `Partner` storage should exist, and the storage should match what was passed in.

Run `anchor test` to make sure all of our tests are passing.

## On to close_partner...

We want to allow users to close their `Partner` storage account and get the rent back when there
is no longer an associated `Wedding` storage account. This function will facilitate that. Let's
create another placeholder:

```rust
pub fn close_partner(ctx: Context<ClosePartner>) -> Result<()> {
    Ok(())
}
```

## New error enums

Let's just get the new `WeddingError` enums that we are going to use out of the way:

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
}
```

On to our `ClosePartner` context struct...

## ClosePartner context

```rust
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
```

Most all of these constraints have been seen already so let's just summarize what we are trying
to do here:

We have two partners associated with a `Wedding` storage account. One should be `user` and the `other`, the other.
We ensure that the user's `Partner` storage account exists to be closed by giving Anchor a
concrete account type of `Partner`. We ensure the `Partner` account is the `user`'s account
through the use of the `seeds` constraint. We check that the `wedding` field in the `Partner`
storage account has a field called wedding and that it matches our `wedding` account passed in
and computed through `seeds`.

Alright on to the implementation...

## close_partner implementation

```rust
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
```

Again, after our extensive set of constraints, we have a rather simple function.
We call a function which we have not yet implemented. Don't worry we will do that next. It just
checks that the wedding account no longer exists. As far as I know there is not a way to do this
though Anchor constraints.

We then close the account and refund the user the rent. Easy.

## check_account_initialized

Below we have our promised function. We simply check if the account has any data or any balance.
If both are empty, we can be pretty sure that it is not initialized.

```rust
fn check_account_initialized(account: &UncheckedAccount) -> bool {
    let account = account.to_account_info();

    let data_empty = account.data_is_empty();
    let lamps = account.lamports();
    let has_lamps = lamps > 0;

    !data_empty || has_lamps
}
```

## Again with the tests...

Run `anchor build` first.

In the interest of keeping this tutorial from going on forever, we will only test the success case:

```typescript
it("should close partner0 as user0", async () => {
  try {
    await eCryptoWedding.methods
      .closePartner()
      .accounts({
        user: uPartner0.publicKey,
        other: uPartner1.publicKey,
        partner: pPartner0,
        wedding: pWedding,
      })
      .signers([uPartner0])
      .rpc();

    try {
      await eCryptoWedding.account.partner.fetch(pPartner0);
      expect.fail("pPartner0 should no longer exist");
    } catch (err) {
      expect(String(err)).to.include("Account does not exist");
    }
  } catch (err) {
    console.error(err);
    throw new Error(err);
  }
});
```

Nothing really new here... this test is very simple to our `cancel_wedding` test. We make sure
the call succeeds and that trying to retrieve data for the no longer existing account fails
with the expected error.

Go ahead and call `anchor test` to make sure our tests pass.

## Summary

In this step we have created both the `setup_partner` and `close_partner` functions. These allow
user's to save data about themselves in relation to the wedding. Allowing users to close these
accounts makes sense under very specific circumstances and we attempt to enforce that.

In the next step we will implement a few more functions. This should hopefully start to feel
a bit more routine at this point :) .
