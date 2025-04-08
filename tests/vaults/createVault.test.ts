import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSDK } from "@starke/sdk";
import {
  InvalidTokenError,
  SignatureVerificationFailedError,
  TokenNotWhitelistedError,
  VaultAlreadyCreatedError,
} from "@starke/sdk/lib/errors";
import { USDC } from "@starke/sdk/whitelist";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
} from "../utils.new";

describe("Create Vault", () => {
  let manager: Keypair;
  let vaults: VaultsSDK;

  // NOTE: Need to use another sdk instance to test this because the signer can not be the manager in some cases
  let authority: Keypair;
  let authorityVaults: VaultsSDK;

  const VAULT_NAME = "rkShares Blue Chip";
  const VAULT_SYMBOL = "rkBlueChip";
  // TODO: Check if this needs to be the png or a json
  const VAULT_URI =
    "https://starke-finance.fra1.cdn.digitaloceanspaces.com/vtoken-metadata/metadata/rkBlueChip.json";
  const ENTRY_FEE = 0;
  const EXIT_FEE = 0;

  before(async () => {
    // Get keypairs
    manager = getManagerKeypair();
    authority = getAuthorityKeypair();

    // Initialize SDK
    vaults = new VaultsSDK(createConnection(), manager);
    authorityVaults = new VaultsSDK(createConnection(), authority);
  });

  it("should fail creating a vault without a valid signer", async () => {
    try {
      // NOTE: Need to use another sdk instance to test this because the signer can not be the manager
      await authorityVaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        ENTRY_FEE,
        EXIT_FEE,
        manager.publicKey,
        USDC.mint,
        []
      );
      expect.fail("Vault creation should have failed");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }
  });

  it("should fail creating a vault with an invalid or non-whitelisted token", async () => {
    const invalidToken = Keypair.generate().publicKey;
    try {
      await authorityVaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        ENTRY_FEE,
        EXIT_FEE,
        authority.publicKey,
        invalidToken
      );
      expect.fail("Vault creation should have failed");
    } catch (e) {
      expect(e).to.be.instanceOf(InvalidTokenError);
    }

    // rkSOL, which is not whitelisted
    const nonWhitelistedToken = new PublicKey(
      "EPCz5LK372vmvCkZH3HgSuGNKACJJwwxsofW6fypCPZL"
    );
    try {
      await authorityVaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        ENTRY_FEE,
        EXIT_FEE,
        authority.publicKey,
        nonWhitelistedToken
      );
      expect.fail("Vault creation should have failed");
    } catch (e) {
      expect(e).to.be.instanceOf(TokenNotWhitelistedError);
    }
  });

  it("should successfully create a vault if not already created", async () => {
    try {
      const signature = await vaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        ENTRY_FEE,
        EXIT_FEE,
        manager.publicKey,
        USDC.mint
      );
      expect(signature).to.not.be.undefined;
    } catch (e) {
      if (!(e instanceof VaultAlreadyCreatedError)) {
        throw e;
      }
    }
  });
});
