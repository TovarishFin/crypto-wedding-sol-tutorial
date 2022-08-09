# step 1

## Everything has a beginning...

Lets start things off by using `anchor` to create a new project. This will setup everything you need to get started with building a program.
In your preferred directory, run:

```sh
anchor init crypto-wedding-program
```

**TEMPORARY FIX:** due to some issues with anchor as of 09.08.22, you need to do the following to make rust-analyzer work:

replace

```toml
[[package]]
name = "anyhow"
version = "1.0.60"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c794e162a5eff65c72ef524dfe393eb923c354e350bb78b9c7383df13f3bc142"
```

with

```toml
[[package]]
name = "anyhow"
version = "1.0.58"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bb07d2053ccdbe10e2af2995a2f116c1330396493dc1269f6a91d0ae82e19704"
```

In the `Cargo.lock` file.

You can track this issue at: https://github.com/coral-xyz/anchor/issues/2111

You should now have a new directory called `crypto-wedding-program`. Let's go check it out!

```sh
cd crypto-wedding-program
```

Here is what we have in there:

```
├── Anchor.toml
├── Cargo.toml
├── README.md
├── app
├── migrations
│   └── deploy.ts
├── package.json
├── programs
│   └── crypto-wedding-program
│       ├── Cargo.toml
│       ├── Xargo.toml
│       └── src
│           └── lib.rs
├── tests
│   └── crypto-wedding-program.ts
├── tsconfig.json
└── yarn.lock
```

Let's try to go through everything quickly so we have an idea of what just happened. We will start with the most important areas and work away into less important areas...

### programs

This is where we are going to write the code for our program. You can store more than one program here if you are building something more complex. For our purposes, we only have the single
folder/program which was created for us when calling `anchor init crypto-wedding-program`, `crypto-wedding-program`. The real "meat" of the code is stored in `src`. The starting point for all anchor
based programs is `lib.rs`. Let's take a quick look at it...

```rust
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod crypto_wedding_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
```

This is the default, most basic program that anchor automatically creates for you when calling `anchor init`. Because this might be your first look at a solana program, let's go through it line by line:

```rust
use anchor_lang::prelude::*;
```

In rust, there is a concept of `prelude`s. The idea is that if you have a complex crate that needs a lot of different stuff, you can use everything in the `prelude` path to bring everything commonly
needed into scope. So this is saying: "bring everything that the `anchor-lang` package thinks is commonly needed into scope, so we don't have to worry about it".

```rust
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
```

We have another interesting rust concept here, macros. Macros are ways for you to essentially, have a package write some sort of code in place of the macro. So what is it actually doing here?
Here, we are declaring an id, or address where the program lives. This is important for security reasons. One of the main attack vectors in solana is to sneak in an account where it shouldn't be.
Declaring this explicitly helps to ensure that this does not happen. Account checking and control is one of the big selling points of anchor. So to summarize, there is a bit of magic happening here
through a macro, where we call this macro specifying where the program should live by an address. That then gets turned into more complicated code that helps to ensure these guarantees. We can't see
that code in our editor but if we wanted to, we can actually see what it is doing if we install `cargo-expand` (out of scope for this tutorial).

```rust
#[program]
pub mod crypto_wedding_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}
```

`#[program]` is another macro which operates on `pub mod crypto_wedding_program`. It sets up everything we need in order to have a program. If you are familiar with solidity,
this could almost be thought of as `contract{}`.

A quick bit about rust: In rust we have modules where we have code inside that can be shared via the rust module system. `pub` says that this is public and we can share this code with other modules.
`mod crypto_wedding_program` declares the module with the name of `crypto_wedding_program`.

Next, we have `use super::*;`. This basically says "use everything we have in the code above this module inside of this module". In this case, it means use everything from `anchor_lang::prelude*`,
whatever code results from `declare_id`, and our `Initialize` struct further down.

Lastly, we have a function! This is a function that a user can call on the program. Here it is not doing much because this is just default code from `anchor init`. `pub` again means that it is public
and that it can be called from outside of this module. `fn` declares it as a function. `initialize` is the name of the function we are declaring. `ctx: Context<Initialize>` is an argument to the function.
This is a pattern specific to anchor. `Context` is what we call a generic type. We can pass in a more specific type here to get something specific to our use case. In our case we are passing in `Initialize`.
This is declared further down in the code outside of the program. `ctx` will generally contain information about which accounts are being used. Anchor enables us, with the use of `constraints` (more on this later),
to contol which accounts are passed into the program.

```rust
#[derive(Accounts)]
pub struct Initialize {}
```

Lastly we have this bit of code. `#[derive(Accounts)]` is another macro which handles some of the more tedious code. Normally we would add different accounts in here which we are interacting with in our
program. But, again, because this is the default anchor program, not much is happening here. Basically this is saying "we aren't interacting with any accounts and not restricting anything". We will more
into this later. But generally these structs are named after the functions in which they are used. So `Initialize` is used as the `ctx` for `pub fn initialize`.

### tests

This is where we are going to write our tests and get a feel for how our program works. The tests here are all written in typescript. If you are unfamiliar with typescript and only javascript, don't
worry much about it. You can write normal javascript inside of typescript files.

I won't go into as much details when it comes to typescript code. Hopefully you either know it or are able to look up the foreign concepts on your own :)

However, lets go through some of the big parts...

```typescript
import { CryptoWeddingProgram } from "../target/types/crypto_wedding_program";
```

You can see here we are importing a custom file here. If you haven't run `anchor build`, do it now and any sort of linting errors should go away. So here you can see that calling `anchor build`
will create some typescript files which help us in interacting with our program.

```typescript
anchor.setProvider(anchor.AnchorProvider.env());
```

This will setup our connection to our blockchain. In our case that will be our localnet blockchain running on our own computer. This localnet blockchain will be spun up freshly every time we run our tests.

```typescript
const tx = await program.methods.initialize().rpc();
console.log("Your transaction signature", tx);
```

Here we are actually interacting with our default program and calling the `initialize` function defined in the program. Calling the program will give us back a signature (txhash for ethereum people).
Using this we can lookup the transaction and get more information about it. This is especially useful when operating on anything other that localnet where we can look up txs on block explorers such as
[the solana block explorer](https://explorer.solana.com).

### Anchor.toml

This is a config file for anchor related things. We probably won't touch this in this tutorial, however it does have some useful fields that are good to know about.
The two most important things here are the `cluster` and `wallet` fields. The cluster is the blockchain network you want to operate on. For development we are going
to want to use `localnet` which means that we are going to run a "local" blockchain on our computer to run our program tests on. There are other networks available
such as `devnet`, `testnet`, and `mainnet-beta`. These other networks are a shared network that run on many machines (as a normal blockchain should).

The `wallet` field tells anchor about the location of the wallet that you setup when installing the `solana` CLI. It will use this wallet when running tests or deploying programs.

There is also a `crypto_wedding_program` field. This declares an address where it should exist. In tests, it will use this default address: `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`.
It will basically cheat and put this program at this address at the start of each test. When running on any other network, you will need to deploy your program and update the address accordingly.

If you want to know more about the other fields, you can check out the [anchor docs on the subject](https://www.anchor-lang.com/docs/manifest).

### Cargo.toml, package.json, yarn.lock

`Cargo.toml` is a rust dependency file that keeps track of crates (packages) that are needed for our program to work. `package.json` is basically the same thing but for javascript. `yarn.lock` is a javascript file that locks in dependencies (don't worry about it :) ).

### app & migrations

`app` is a folder where you can build the frontend or something else that interacts with the program if you want to keep the code in one place. We will not be using it. You can even delete it if you are
feeling super daring :)

`migrations` is a place where you can store scripts that deploy and setup your programs. We can use this later when we are done building the program and have completed our tests.

### tsconfig.json

This is a config file for typescript. If you don't know what typescript is, it is basically javascript but with added features, mainly types. Let's not worry much about that here...

### Xargo.toml

This is beyond the scope of this tutorial. More info can be found [here](https://github.com/japaric/xargo).

## Running our first test

Go ahead and run `anchor test`. You should get output similar to the following:

```sh
Your transaction signature 3P18Mb9hUhvL15yu9duPPv7x4kVybHx7Ev9wCYExqdDtR8HbXDTpWPF321RgHTPpo2JDs3FDJW8RwvY5qXHfVsff
    ✔ Is initialized! (336ms)


  1 passing (338ms)
```

Note: the signature does not have to match.

Congrats! You have setup your first solana program using anchor and ran the tests to go with it!

Move on to step-2 in order to start writing our own solana program.
