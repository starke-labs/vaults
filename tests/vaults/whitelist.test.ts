import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSDK } from "@starke/sdk";
import {
  SignatureVerificationFailedError,
  TokenAlreadyInWhitelistError,
  TokenNotWhitelistedError,
  WhitelistAlreadyInitializedError,
} from "@starke/sdk/lib/errors";
import { Token } from "@starke/sdk/lib/types";
import { JUP, PYTH, USDC, USDT } from "@starke/sdk/whitelist";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
} from "../utils.new";

describe("Whitelist Tests", () => {
  let vaults: VaultsSDK;
  let tester: Keypair;
  let authority: Keypair;

  const dummyToken: Token = {
    mint: Keypair.generate().publicKey,
    priceFeedId: "dummy_price_feed_id",
    priceUpdate: Keypair.generate().publicKey,
  };

  before(async () => {
    // Get keypairs
    tester = getManagerKeypair();
    authority = getAuthorityKeypair();

    // Initialize SDK
    vaults = new VaultsSDK(createConnection(), tester);
  });

  // TODO: Enchance this test when sdk has error handling for initializeWhitelist
  it("should not initialize whitelist without proper authority", async () => {
    // Try with no signer
    try {
      await vaults.initializeWhitelist([]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      if (
        !(
          e instanceof WhitelistAlreadyInitializedError ||
          e instanceof SignatureVerificationFailedError
        )
      ) {
        throw e;
      }
    }

    // Try with non-authority signer
    try {
      await vaults.initializeWhitelist([tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      if (
        !(
          e instanceof WhitelistAlreadyInitializedError ||
          e instanceof SignatureVerificationFailedError
        )
      ) {
        throw e;
      }
    }
  });

  it("should successfully initialize token whitelist if not initialized already", async () => {
    // Try to initialize whitelist
    try {
      await vaults.initializeWhitelist([authority]);
    } catch (e) {
      if (!(e instanceof WhitelistAlreadyInitializedError)) {
        throw e;
      }
    }

    // Verify whitelist was initialized
    const whitelist = await vaults.fetchWhitelist();
    expect(whitelist.authority.toBase58()).to.equal(
      authority.publicKey.toBase58()
    );
  });

  it("should not add token to whitelist without proper authority", async () => {
    // Check the length of the whitelist
    let whitelist = await vaults.fetchWhitelist();
    const initialLength = whitelist.tokens.length;

    // Test with no signer
    try {
      await vaults.addTokenToWhitelist(dummyToken, []);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }

    // Verify token was not added
    whitelist = await vaults.fetchWhitelist();
    let finalLength = whitelist.tokens.length;
    expect(finalLength).to.equal(initialLength);

    // Test with non-authority signer
    try {
      await vaults.addTokenToWhitelist(dummyToken, [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }

    // Verify token was still not added
    whitelist = await vaults.fetchWhitelist();
    finalLength = whitelist.tokens.length;
    expect(finalLength).to.equal(initialLength);
  });

  it("should not initialize whitelist twice", async () => {
    try {
      await vaults.initializeWhitelist([authority]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(WhitelistAlreadyInitializedError);
    }
  });

  it("should successfully add USDC to whitelist if it is not already in the whitelist", async () => {
    try {
      await vaults.addTokenToWhitelist(USDC, [authority]);
    } catch (e) {
      if (!(e instanceof TokenAlreadyInWhitelistError)) {
        throw e;
      }
    }

    // Verify token was added
    const whitelistedToken = await vaults.fetchWhitelistedTokens(USDC.mint);
    expect(whitelistedToken.mint.toBase58()).to.equal(USDC.mint.toBase58());
    expect(whitelistedToken.priceFeedId).to.equal(USDC.priceFeedId);
    expect(whitelistedToken.priceUpdate.toBase58()).to.equal(
      USDC.priceUpdate.toBase58()
    );
  });

  it("should not add same token twice", async () => {
    try {
      await vaults.addTokenToWhitelist(USDC, [authority]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(TokenAlreadyInWhitelistError);
    }
  });

  it("should successfully add the rest of the whitelisted tokens if they are not already in the whitelist", async () => {
    try {
      await vaults.addTokenToWhitelist(USDT, [authority]);
    } catch (e) {
      if (!(e instanceof TokenAlreadyInWhitelistError)) {
        throw e;
      }
    }

    try {
      await vaults.addTokenToWhitelist(PYTH, [authority]);
    } catch (e) {
      if (!(e instanceof TokenAlreadyInWhitelistError)) {
        throw e;
      }
    }

    try {
      await vaults.addTokenToWhitelist(JUP, [authority]);
    } catch (e) {
      if (!(e instanceof TokenAlreadyInWhitelistError)) {
        throw e;
      }
    }

    // Verify all tokens were added
    const whitelist = await vaults.fetchWhitelist();
    [USDC, USDT, PYTH, JUP].forEach((token) => {
      const whitelistedToken = whitelist.tokens.find(
        (t) => t.mint.toBase58() === token.mint.toBase58()
      );
      expect(whitelistedToken).to.not.be.undefined;
      expect(whitelistedToken?.priceFeedId).to.equal(token.priceFeedId);
      expect(whitelistedToken?.priceUpdate.toBase58()).to.equal(
        token.priceUpdate.toBase58()
      );
    });
  });

  it("should not remove token from whitelist without proper authority", async () => {
    // Try to remove token from whitelist without any signers
    try {
      await vaults.removeTokenFromWhitelist(USDC.mint, []);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }

    // Try to remove token from whitelist with non-authority signer
    try {
      await vaults.removeTokenFromWhitelist(USDC.mint, [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }
  });

  it("should successfully remove token from whitelist", async () => {
    // Add token to whitelist
    await vaults.addTokenToWhitelist(dummyToken, [authority]);

    // Verify token was added
    let tokenInWhitelist = await vaults.fetchWhitelistedTokens(dummyToken.mint);
    expect(tokenInWhitelist.mint.toBase58()).to.equal(
      dummyToken.mint.toBase58()
    );
    expect(tokenInWhitelist.priceFeedId).to.equal(dummyToken.priceFeedId);
    expect(tokenInWhitelist.priceUpdate.toBase58()).to.equal(
      dummyToken.priceUpdate.toBase58()
    );

    // Remove token from whitelist
    await vaults.removeTokenFromWhitelist(dummyToken.mint, [authority]);

    // Verify token was removed
    try {
      tokenInWhitelist = await vaults.fetchWhitelistedTokens(dummyToken.mint);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(TokenNotWhitelistedError);
    }
  });

  it("should not remove token from whitelist if token is not in the whitelist", async () => {
    // Try to remove token from whitelist without any signers
    try {
      await vaults.removeTokenFromWhitelist(dummyToken.mint, []);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(TokenNotWhitelistedError);
    }
  });
});
