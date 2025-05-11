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
    const starkeConfig1 = await vaults.fetchStarkeConfig();

    // Try to initialize starke
    try {
      await vaults.initializeStarke([authority]);
    } catch (e) {
      if (!(e instanceof StarkeAlreadyInitializedError)) {
        throw e;
      }
    }

    // Verify starke config was initialized
    const starkeConfig = await vaults.fetchStarkeConfig();
    expect(starkeConfig.isPaused).to.equal(false);

    // Verify token whitelist was initialized
    const whitelist = await vaults.fetchTokenWhitelist();
    expect(whitelist.authority.toBase58()).to.equal(
      authority.publicKey.toBase58()
    );

    // Verify manager whitelist was initialized
    const managerWhitelist = await vaults.fetchManagerWhitelist();
    expect(managerWhitelist.managers.length).to.equal(0);
  });
});
