import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import {
  SignatureVerificationFailedError,
  StarkeAlreadyInitializedError,
} from "@starke/sdk/lib/errors";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
} from "../utils.new";

describe("Initialize Starke Tests", () => {
  let vaults: VaultsSdk;
  let tester: Keypair;
  let authority: Keypair;

  before(async () => {
    // Get keypairs
    tester = getManagerKeypair();
    authority = getAuthorityKeypair();

    // Initialize SDK
    vaults = new VaultsSdk(createConnection(), tester);
  });

  // TODO: Enchance this test when sdk has error handling for initializeWhitelist
  it("should not initialize Starke without proper authority", async () => {
    // Try with no signer
    try {
      await vaults.initializeStarke([]); // TODO: Fix wrong test case - this has tester as signer
      expect.fail("Should have thrown an error");
    } catch (e) {
      if (
        !(
          e instanceof StarkeAlreadyInitializedError ||
          e instanceof SignatureVerificationFailedError
        )
      ) {
        throw e;
      }
    }

    // Try with non-authority signer
    try {
      await vaults.initializeStarke([tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      if (
        !(
          e instanceof StarkeAlreadyInitializedError ||
          e instanceof SignatureVerificationFailedError
        )
      ) {
        throw e;
      }
    }
  });

  it("should successfully initialize starke if not initialized already", async () => {
    let wasAlreadyInitialized = false;

    // Try to initialize starke
    try {
      await vaults.initializeStarke([authority]);
    } catch (e) {
      if (e instanceof StarkeAlreadyInitializedError) {
        wasAlreadyInitialized = true;
      } else {
        throw e;
      }
    }

    // Verify starke config was initialized
    const starkeConfig = await vaults.fetchStarkeConfig();
    expect(starkeConfig).to.exist;
    expect(starkeConfig.isPaused).to.equal(false);

    // Verify token whitelist was initialized
    const tokenWhitelist = await vaults.fetchTokenWhitelist();
    expect(tokenWhitelist).to.exist;
    expect(tokenWhitelist.authority.toBase58()).to.equal(
      authority.publicKey.toBase58()
    );

    // Verify manager whitelist was initialized
    const managerWhitelist = await vaults.fetchManagerWhitelist();
    expect(managerWhitelist).to.exist;
    expect(managerWhitelist.managers).to.be.an("array");

    // Verify user whitelist was initialized
    const userWhitelist = await vaults.fetchUserWhitelist();
    expect(userWhitelist).to.exist;
    expect(userWhitelist.authority.toBase58()).to.equal(
      authority.publicKey.toBase58()
    );
  });

  it("should not initialize whitelist twice", async () => {
    try {
      await vaults.initializeStarke([authority]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(StarkeAlreadyInitializedError);
    }
  });
});
