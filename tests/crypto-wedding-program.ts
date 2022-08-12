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
  WeddingMarried,
  WeddingMarrying,
  WeddingDivorcing,
} from "./helpers";

describe("when using CryptoWeddingProgram", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  // e for executable
  const eCryptoWedding = anchor.workspace
    .CryptoWeddingProgram as Program<CryptoWedding>;
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
});

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
});
