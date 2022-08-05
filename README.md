# solana crypto wedding tutorial

## About

This tutorial covers the following concepts:

- some basic rust programming
- how to use different networks (localnet, devnet, etc.)
- program development using Anchor
- program testing using Anchor
- how to structure a program
- the solana account model
- how to handle errors
- PDAs (program derived accounts)
- some basic defensive programming
- interacting with programs using rust based clients

The "finished" code (very much WIP) can be found here:

- [crypto wedding program](https://github.com/TovarishFin/crypto-wedding-sol)
- [crypto wedding cli](https://github.com/TovarishFin/crypto-wedding-sol-cli)

## Required before starting

- [install nodejs for testing our programs](https://nodejs.dev/learn/how-to-install-nodejs)
- [install rust for building our programs](https://www.rust-lang.org/tools/install)
- [install solana cli for running a local node during testing](https://docs.solana.com/cli?utm_source=solana.com)
  - have an account created to use with solana-cli (details below)
- [install anchor cli for deployment and testing of our programs](https://www.anchor-lang.com/docs/installation)

To create an account for the solana-cli simply run the following (`solana-keygen` is installed after installing solana cli):

```sh
solana-keygen new
```

This will **NOT** overwrite any old keys unless you use `--force` **DO NOT** use `--force` unless you
know what you are doing and are **sure** that you have **NO FUNDS** on the account.

### reccommended

- [some sort of rust tooling for your code editor rust-analyzer is considered the best AFAIK](https://rust-analyzer.github.io/)

### optional

- [install yarn you can use npm which comes with nodejs as well](https://yarnpkg.com/getting-started/install)

## This Tutorial was Inspired by...

- [this paulx blog post](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/)
- [anchor tutorials](https://www.anchor-lang.com/docs/hello-world)
- [doors vs wheels](https://medium.com/@nicoeft/doors-or-wheels-a-solana-voting-app-in-anchor-using-pdas-and-sol-transfers-9c521cda0b99)
- [learning how to build on solana](https://www.brianfriel.xyz/learning-how-to-build-on-solana/)

After going through the above tutorials, I more or less felt comfortable branching out and trying something on my own.

## Other nice resources

- [great super fast blunt explanation on how things generally work in solana](https://2501babe.github.io/posts/solana101.html)
- [great super fast blunt explanation on how things work in anchor](https://2501babe.github.io/posts/anchor101.html)
- [Metaplex docs. These are the best explanations on how both fungible and non-fungible tokens work that I have found!](https://docs.metaplex.com/programs/)

## What to do when Stuck?

Check out the [solana stackexchange](https://solana.stackexchange.com). There are some very knowledgeable people there. If you post a unique question there it will likely eventually get answered. You just need to be patient and wait :)

## What should I do after completing this tutorial?

Go build your own project! At this point you probably know enough to build something and/or figure out how to do the new thing that you may need in your future projects.
