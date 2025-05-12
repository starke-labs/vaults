import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import {
  InvalidTokenError,
  ManagerNotWhitelistedError,
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
  let vaults: VaultsSdk;

  // NOTE: Need to use another sdk instance to test this because the signer can not be the manager in some cases
  let authority: Keypair;
  let authorityVaults: VaultsSdk;

  const VAULT_NAME = "rkShares Blue Chip";
  const VAULT_SYMBOL = "rkBlueChip";
  const VAULT_URI =
    "https://starke-finance.fra1.cdn.digitaloceanspaces.com/vtoken-metadata/metadata/rkBlueChip.json";
  const ENTRY_FEE = 0;
  const EXIT_FEE = 0;
  const IS_VTOKEN_TRANSFERRABLE = false;

  before(async () => {
    // Get keypairs
    manager = getManagerKeypair();
    authority = getAuthorityKeypair();

    // Initialize SDK
    vaults = new VaultsSdk(createConnection(), manager);
    authorityVaults = new VaultsSdk(createConnection(), authority);
  });

  // it("should fail creating a vault without a valid signer", async () => {
  //   try {
  //     const signer = Keypair.generate();
  //     const newVaults = new VaultsSdk(createConnection(), signer);
  //     await newVaults.createVault(
  //       VAULT_NAME,
  //       VAULT_SYMBOL,
  //       VAULT_URI,
  //       ENTRY_FEE,
  //       EXIT_FEE,
  //       manager.publicKey,
  //       USDC.mint,
  //       IS_VTOKEN_TRANSFERRABLE
  //     );
  //     expect.fail("Vault creation should have failed");
  //   } catch (e) {
  //     expect(e).to.be.instanceOf(SignatureVerificationFailedError);
  //   }
  // });

  // it("should fail creating a vault if the manager is not whitelisted", async () => {
  //   try {
  //     await authorityVaults.createVault(
  //       VAULT_NAME,
  //       VAULT_SYMBOL,
  //       VAULT_URI,
  //       ENTRY_FEE,
  //       EXIT_FEE,
  //       authority.publicKey,
  //       USDC.mint,
  //       IS_VTOKEN_TRANSFERRABLE
  //     );
  //     expect.fail("Vault creation should have failed");
  //   } catch (e) {
  //     expect(e).to.be.instanceOf(ManagerNotWhitelistedError);
  //   }
  // });

  // it("should fail creating a vault with an invalid or non-whitelisted token", async () => {
  //   const invalidToken = Keypair.generate().publicKey;
  //   try {
  //     await vaults.createVault(
  //       VAULT_NAME,
  //       VAULT_SYMBOL,
  //       VAULT_URI,
  //       ENTRY_FEE,
  //       EXIT_FEE,
  //       manager.publicKey,
  //       invalidToken,
  //       IS_VTOKEN_TRANSFERRABLE
  //     );
  //     expect.fail("Vault creation should have failed");
  //   } catch (e) {
  //     expect(e).to.be.instanceOf(InvalidTokenError);
  //   }

  //   // rkSOL, which is not whitelisted
  //   const nonWhitelistedToken = new PublicKey(
  //     "EPCz5LK372vmvCkZH3HgSuGNKACJJwwxsofW6fypCPZL"
  //   );
  //   try {
  //     await vaults.createVault(
  //       VAULT_NAME,
  //       VAULT_SYMBOL,
  //       VAULT_URI,
  //       ENTRY_FEE,
  //       EXIT_FEE,
  //       manager.publicKey,
  //       nonWhitelistedToken,
  //       IS_VTOKEN_TRANSFERRABLE
  //     );
  //     expect.fail("Vault creation should have failed");
  //   } catch (e) {
  //     expect(e).to.be.instanceOf(TokenNotWhitelistedError);
  //   }
  // });

  it("should successfully create a vault if not already created", async () => {
    try {
      const signature = await vaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        ENTRY_FEE,
        EXIT_FEE,
        manager.publicKey,
        USDC.mint,
        IS_VTOKEN_TRANSFERRABLE
      );
      expect(signature).to.not.be.undefined;
    } catch (e) {
      if (!(e instanceof VaultAlreadyCreatedError)) {
        throw e;
      }
    }
  });

  // TODO: Test that vaults can't be created if Starke is paused
  // TODO: Test that vtokens created with not transferrable flag actually aren't transferrable
});
