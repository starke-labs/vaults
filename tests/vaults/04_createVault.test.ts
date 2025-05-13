import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import {
  InvalidTokenError,
  ManagerNotWhitelistedError,
  SignatureVerificationFailedError,
  TokenNotWhitelistedError,
  VaultAlreadyCreatedError,
} from "@starke/sdk/lib/errors";
import { getVtokenMetadataPda } from "@starke/sdk/lib/pdas";
import { TransferHookSdk } from "@starke/sdk/transferHook";
import { USDC } from "@starke/sdk/whitelist";

import {
  createConnection,
  getAuthorityKeypair, // getManager2Keypair,
  getManagerKeypair,
} from "../utils.new";

describe("Create Vault", () => {
  let connection: Connection;

  let manager: Keypair;
  let vaults: VaultsSdk;
  let transferHook: TransferHookSdk;

  // let manager2: Keypair;
  // let vaults2: VaultsSdk;

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
    // manager2 = getManager2Keypair();
    authority = getAuthorityKeypair();

    // Initialize SDKs
    connection = createConnection();
    vaults = new VaultsSdk(connection, manager);
    // vaults2 = new VaultsSdk(connection, manager2);
    authorityVaults = new VaultsSdk(connection, authority);
    transferHook = new TransferHookSdk(connection, manager);
  });

  it("should fail creating a vault without a valid signer", async () => {
    try {
      const signer = Keypair.generate();
      const newVaults = new VaultsSdk(createConnection(), signer);
      await newVaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        ENTRY_FEE,
        EXIT_FEE,
        manager.publicKey,
        USDC.mint,
        IS_VTOKEN_TRANSFERRABLE
      );
      expect.fail("Vault creation should have failed");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }
  });

  it("should fail creating a vault if the manager is not whitelisted", async () => {
    try {
      await authorityVaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        ENTRY_FEE,
        EXIT_FEE,
        authority.publicKey,
        USDC.mint,
        IS_VTOKEN_TRANSFERRABLE
      );
      expect.fail("Vault creation should have failed");
    } catch (e) {
      expect(e).to.be.instanceOf(ManagerNotWhitelistedError);
    }
  });

  it("should fail creating a vault with an invalid or non-whitelisted token", async () => {
    const invalidToken = Keypair.generate().publicKey;
    try {
      await vaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        ENTRY_FEE,
        EXIT_FEE,
        manager.publicKey,
        invalidToken,
        IS_VTOKEN_TRANSFERRABLE
      );
      expect.fail("Vault creation should have failed");
    } catch (e) {
      expect(
        e instanceof InvalidTokenError || e instanceof VaultAlreadyCreatedError
      ).to.be.true;
    }

    // rkSOL, which is not whitelisted
    const nonWhitelistedToken = new PublicKey(
      "EPCz5LK372vmvCkZH3HgSuGNKACJJwwxsofW6fypCPZL"
    );
    try {
      await vaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        ENTRY_FEE,
        EXIT_FEE,
        manager.publicKey,
        nonWhitelistedToken,
        IS_VTOKEN_TRANSFERRABLE
      );
      expect.fail("Vault creation should have failed");
    } catch (e) {
      expect(
        e instanceof TokenNotWhitelistedError ||
          e instanceof VaultAlreadyCreatedError
      ).to.be.true;
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
        USDC.mint,
        IS_VTOKEN_TRANSFERRABLE
      );
      expect(signature).to.not.be.undefined;
    } catch (e) {
      if (!(e instanceof VaultAlreadyCreatedError)) {
        throw e;
      }
    }

    // Check if the vault was created
    const vault = await vaults.fetchVault(manager.publicKey);
    expect(vault).to.not.be.undefined;
    expect(vault.manager.toBase58()).to.equal(manager.publicKey.toBase58());

    // Check if the metadata was created
    const [metadataPda] = getVtokenMetadataPda(vault.mint);
    const metadata = await connection.getAccountInfo(metadataPda);
    expect(metadata).to.not.be.undefined;

    // Check if the vtoken config was created
    const vtokenConfig = await transferHook.fetchVtokenConfig(vault.mint);
    expect(vtokenConfig).to.not.be.undefined;
    expect(vtokenConfig.vtokenIsTransferrable).to.equal(
      IS_VTOKEN_TRANSFERRABLE
    );
    expect(vtokenConfig.vtokenMint.toBase58()).to.equal(vault.mint.toBase58());
    expect(vtokenConfig.manager.toBase58()).to.equal(
      manager.publicKey.toBase58()
    );
  });

  // TODO: Test that vaults can't be created if Starke is paused
  // it("should not be able to create a vault if Starke is paused", async () => {
  //   // Pause Starke
  //   await authorityVaults.pauseStarke();

  //   // Make sure Starke is paused
  //   let starkeConfig = await authorityVaults.fetchStarkeConfig();
  //   expect(starkeConfig.isPaused).to.be.true;

  //   // Try to create a vault
  //   try {
  //     await vaults2.createVault(
  //       VAULT_NAME,
  //       VAULT_SYMBOL,
  //       VAULT_URI,
  //       ENTRY_FEE,
  //       EXIT_FEE,
  //       manager2.publicKey,
  //       USDC.mint,
  //       IS_VTOKEN_TRANSFERRABLE
  //     );
  //     expect.fail("Vault creation should have failed");
  //   } catch (e) {
  //     console.log(e);
  //     expect(e).to.be.instanceOf(Error);
  //   }

  //   // Resume Starke
  //   await authorityVaults.resumeStarke();

  //   // Make sure that Starke was resumed
  //   starkeConfig = await authorityVaults.fetchStarkeConfig();
  //   expect(starkeConfig.isPaused).to.be.false;
  // });

  // TODO: Test that vtokens created with not transferrable flag actually aren't transferrable
});
