# step 0

Some background information about solana, anchor, and what we are going to be building.

## A bit about what we are building...

We are going to be building a program where users can get married on the blockchain. I have already built this on Ethereum many years ago. You can find Ethereum version deployed along with a frontend
for it [here](https://cryptoweddings.io). Overall it is a pretty simple program with an easy to grasp concepts.

In our program a user should be able to do the following:

- setup a wedding
  - cancel it if needed
- update user information
  - name
  - vows
  - whether or not they agree to get married
- agree to marry
- divorce

Thats it! Sounds simple right? What could go wrong :)

## A bit about Solana and Solana programs...

Solana is a rather new blockchain which has programs (read smart contracts Ethereans). Where ethereum uses a domain-specific language, solidity, Solana uses rust along with some specialized macros
to achieve the same goals.

A big difference from Ethereum smart contracts is that Solana programs are stateless. In basic terms, this means that you need to pass in the state upon which the program is acting.
To repeat, **state and programs are very distinctly seperated in solana**. This concept has big implications in regards to how we think about programs. Further along this line of thinking, we can
dive a bit into solana's account model.

Accounts must pay rent to be kept alive. If you pay enough rent for 2 years, the account is considered rent exempt. Nearly everyone simply pays enough to be rent exempt rather than the alternative.

Accounts as a concept means a pretty wide variety of things in Solana. A program is an account, but cannot have any data. A user has an account which can have a balance but no data. An account can
act as a data account as well which serves as state that you pass into a program to be acted upon. No matter what, these accounts all need to pay rent in order to stay alive in solana's memory.

Because state and programs are different accounts, you need to pass state into a program. This is one of the most common attack vectors in solana. This is where anchor comes in...

## A bit about Anchor...

Anchor is a framework which helps solana program developers ensure that correct accounts are passed into programs and generally abstracts away a lot of the lower level tedious parts of solana program
development. It takes care of the serialization and deserialization of program arguments which is one of the more tedious tasks.
It also makes testing and deploying programs much more simple via typescript. If you are coming from Ethereum, Anchor is something like truffle.
