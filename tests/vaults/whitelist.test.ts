import { AnchorProvider, Idl, Wallet, setProvider } from "@coral-xyz/anchor";
import { createMint } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import idl from "@starke/idl/vaults.json";
import { VaultsSDK } from "@starke/sdk";
import { AddTokenAccounts, AddTokenParams } from "@starke/sdk/types";

import {
  getAuthorityKeypair,
  getProvider,
  requestAirdropIfNecessary,
} from "../utils.new";
import {
  DEFAULT_MINT_DECIMALS,
  DUMMY_PRICE_FEED_ID,
} from "../utils.new/constants";

describe("Whitelist Tests", () => {
  let sdk: VaultsSDK;
  let provider: AnchorProvider;
  let tester: Keypair;
  let tokenMint: PublicKey;
  let authority: Keypair;

  before(async () => {
    // Setup provider and authority
    tester = Keypair.generate();
    provider = getProvider(tester);

    // Get program authority keypair
    authority = getAuthorityKeypair();

    // Request SOL for authority if needed
    await requestAirdropIfNecessary(provider.connection, tester.publicKey);

    // Initialize SDK
    sdk = new VaultsSDK(
      provider.connection,
      tester,
      new PublicKey(idl.address),
      idl as Idl
    );

    // Create a test token mint
    tokenMint = await createMint(
      provider.connection,
      tester,
      tester.publicKey,
      null,
      DEFAULT_MINT_DECIMALS
    );
  });

  it("should not initialize whitelist without proper authority", async () => {
    const ix = await sdk.initializeWhitelist();

    // Try with no signer
    try {
      await sdk.sendTransaction([ix]);
    } catch (e) {
      expect(e.toString()).to.have.string("Signature verification failed");
    }

    // Try with non-authority signer
    try {
      await sdk.sendTransaction([ix], [tester]);
    } catch (e) {
      expect(e.toString()).to.have.string("Signature verification failed");
    }
  });

  it("should successfully initialize token whitelist", async () => {
    const ix = await sdk.initializeWhitelist();
    const signature = await sdk.sendTransaction([ix], [authority]);
    expect(signature).to.not.be.empty;

    // Verify whitelist was initialized
    const whitelist = await sdk.fetchWhitelist();
    expect(whitelist.authority.toString()).to.equal(
      authority.publicKey.toString()
    );
    expect(whitelist.programAuthority.toString()).to.equal(
      authority.publicKey.toString()
    );
    expect(whitelist.tokens).to.be.empty;
  });

  it("should not add token to whitelist without proper authority", async () => {
    const params: AddTokenParams = {
      token: tokenMint,
      priceFeedId: DUMMY_PRICE_FEED_ID,
    };

    // Test with no signer
    const noSignerAccounts: AddTokenAccounts = {
      authority: tester.publicKey,
    };
    const noSignerIx = await sdk.addToken(params, noSignerAccounts);
    try {
      await sdk.sendTransaction([noSignerIx]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("UnauthorizedAccess");
    }

    // Verify token was not added
    let whitelist = await sdk.fetchWhitelist();
    expect(whitelist.tokens).to.be.empty;

    // Test with non-authority signer
    const nonAuthorityAccounts: AddTokenAccounts = {
      authority: tester.publicKey,
    };
    const nonAuthorityIx = await sdk.addToken(params, nonAuthorityAccounts);
    try {
      await sdk.sendTransaction([nonAuthorityIx], [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("UnauthorizedAccess");
    }

    // Verify token was still not added
    whitelist = await sdk.fetchWhitelist();
    expect(whitelist.tokens).to.be.empty;
  });

  it("should successfully add token to whitelist", async () => {
    const params: AddTokenParams = {
      token: tokenMint,
      priceFeedId: DUMMY_PRICE_FEED_ID,
    };

    const accounts: AddTokenAccounts = {
      authority: authority.publicKey,
    };

    const ix = await sdk.addToken(params, accounts);
    const signature = await sdk.sendTransaction([ix], [authority]);
    expect(signature).to.not.be.empty;

    // Verify token was added
    const whitelist = await sdk.fetchWhitelist();
    expect(whitelist.tokens).to.have.length(1);
    expect(whitelist.tokens[0].mint.toString()).to.equal(tokenMint.toString());
    expect(whitelist.tokens[0].priceFeedId).to.equal(DUMMY_PRICE_FEED_ID);
  });

  it("should not add same token twice", async () => {
    const params: AddTokenParams = {
      token: tokenMint,
      priceFeedId: DUMMY_PRICE_FEED_ID,
    };

    const accounts: AddTokenAccounts = {
      authority: authority.publicKey,
    };

    const ix = await sdk.addToken(params, accounts);

    try {
      await sdk.sendTransaction([ix], [authority]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("Token is already whitelisted");
    }

    // Verify no duplicate token was added
    const whitelist = await sdk.fetchWhitelist();
    expect(whitelist.tokens).to.have.length(1);
    expect(whitelist.tokens[0].mint.toString()).to.equal(tokenMint.toString());
  });

  // TODO: Implement this
  // it("should not add token with invalid price feed ID", async () => {
  //   // Create a new token mint with program authority as mint authority
  //   const newToken = await createMint(
  //     provider.connection,
  //     programAuthority,
  //     programAuthority.publicKey,
  //     programAuthority.publicKey,
  //     DEFAULT_MINT_DECIMALS
  //   );

  //   const params: AddTokenParams = {
  //     token: newToken,
  //     priceFeedId: "invalid_price_feed_id", // Invalid format
  //   };

  //   const accounts: AddTokenAccounts = {
  //     authority: programAuthority.publicKey,
  //   };

  //   const ix = await sdk.addToken(params, accounts);

  //   try {
  //     await sdk.sendTransaction([ix], [programAuthority]);
  //     expect.fail("Should have thrown an error");
  //   } catch (e) {
  //     // TODO: Add more specific error check
  //     expect(e.toString()).to.have.string("Invalid price feed ID");
  //   }

  //   // Verify token was not added
  //   const whitelist = await sdk.fetchWhitelist();
  //   expect(whitelist.tokens).to.have.length(1); // Still only the first token
  // });
});
