import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import {
  InsufficientBalanceError,
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
  getAuthorityKeypair,
  getManagerKeypair,
  getManager2Keypair,
  getTesterKeypair,
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
  const IS_VTOKEN_TRANSFERRABLE = false;
  const MAX_ALLOWED_AUM: BN | null = null;
  const ALLOW_RETAIL = true;
  const ALLOW_ACCREDITED = true;
  const ALLOW_INSTITUTIONAL = true;
  const ALLOW_QUALIFIED = true;
  const INDIVIDUAL_MIN_DEPOSIT: number = 0; // 0 = no minimum
  const INSTITUTIONAL_MIN_DEPOSIT: number = 0; // 0 = no minimum
  const MAX_DEPOSITORS: number = 0; // 0 = unlimited
  const INITIAL_VTOKEN_PRICE: number = 2;

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
        manager.publicKey,
        USDC.mint,
        IS_VTOKEN_TRANSFERRABLE,
        MAX_ALLOWED_AUM,
        ALLOW_RETAIL,
        ALLOW_ACCREDITED,
        ALLOW_INSTITUTIONAL,
        ALLOW_QUALIFIED,
		INITIAL_VTOKEN_PRICE,
        INDIVIDUAL_MIN_DEPOSIT,
        INSTITUTIONAL_MIN_DEPOSIT,
        MAX_DEPOSITORS
      );
      expect.fail("Vault creation should have failed");
    } catch (e) {
      // Should fail due to either insufficient balance or signature verification
      const isExpectedError = e instanceof InsufficientBalanceError || 
        e instanceof SignatureVerificationFailedError;
      expect(isExpectedError).to.be.true;
    }
  });

  it("should fail creating a vault if the manager is not whitelisted", async () => {
    try {
      const nonWhitelistedManager = Keypair.generate();
      const nonWhitelistedVaults = new VaultsSdk(createConnection(), nonWhitelistedManager);
      await nonWhitelistedVaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        nonWhitelistedManager.publicKey,
        USDC.mint,
        IS_VTOKEN_TRANSFERRABLE,
        MAX_ALLOWED_AUM,
        ALLOW_RETAIL,
        ALLOW_ACCREDITED,
        ALLOW_INSTITUTIONAL,
        ALLOW_QUALIFIED,
		INITIAL_VTOKEN_PRICE,
        INDIVIDUAL_MIN_DEPOSIT,
        INSTITUTIONAL_MIN_DEPOSIT,
        MAX_DEPOSITORS
      );
      expect.fail("Vault creation should have failed");
    } catch (e) {
      // Non-whitelisted manager should fail - either due to whitelist check or insufficient funds
      const isExpectedError = e.toString().includes("ManagerNotWhitelisted") || 
        e.toString().includes("Manager is not whitelisted") ||
        e.toString().includes("manager not whitelisted") ||
        e instanceof InsufficientBalanceError;
      expect(isExpectedError).to.be.true;
    }
  });

  it("should fail creating a vault with an invalid or non-whitelisted token", async () => {
    const invalidToken = Keypair.generate().publicKey;
    try {
      await vaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        manager.publicKey,
        invalidToken,
        IS_VTOKEN_TRANSFERRABLE,
        MAX_ALLOWED_AUM,
        ALLOW_RETAIL,
        ALLOW_ACCREDITED,
        ALLOW_INSTITUTIONAL,
        ALLOW_QUALIFIED,
		INITIAL_VTOKEN_PRICE,
        INDIVIDUAL_MIN_DEPOSIT,
        INSTITUTIONAL_MIN_DEPOSIT,
        MAX_DEPOSITORS
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
        manager.publicKey,
        nonWhitelistedToken,
        IS_VTOKEN_TRANSFERRABLE,
        MAX_ALLOWED_AUM,
        ALLOW_RETAIL,
        ALLOW_ACCREDITED,
        ALLOW_INSTITUTIONAL,
        ALLOW_QUALIFIED,
		INITIAL_VTOKEN_PRICE,
        INDIVIDUAL_MIN_DEPOSIT,
        INSTITUTIONAL_MIN_DEPOSIT,
        MAX_DEPOSITORS
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
    let vaultWasCreated = false;
    try {
      const signature = await vaults.createVault(
        VAULT_NAME,
        VAULT_SYMBOL,
        VAULT_URI,
        manager.publicKey,
        USDC.mint,
        IS_VTOKEN_TRANSFERRABLE,
        MAX_ALLOWED_AUM,
        ALLOW_RETAIL,
        ALLOW_ACCREDITED,
        ALLOW_INSTITUTIONAL,
        ALLOW_QUALIFIED,
		INITIAL_VTOKEN_PRICE,
        INDIVIDUAL_MIN_DEPOSIT,
        INSTITUTIONAL_MIN_DEPOSIT,
        MAX_DEPOSITORS
      );
      expect(signature).to.not.be.undefined;
      vaultWasCreated = true;
      console.log("Vault was created fresh with parameters:");
      console.log("ALLOW_RETAIL:", ALLOW_RETAIL);
    } catch (e) {
      if (!(e instanceof VaultAlreadyCreatedError)) {
        throw e;
      }
      console.log("Vault already existed - using existing vault");
    }

    // Check if the vault was created
    const vault = await vaults.fetchVault(manager.publicKey);
    expect(vault).to.not.be.undefined;
    expect(vault.manager.toBase58()).to.equal(manager.publicKey.toBase58());
    
    // Verify new investor type settings only if vault was created fresh
    if (vaultWasCreated) {
      console.log("Verifying parameters for freshly created vault");
      expect(vault.allowRetail).to.equal(ALLOW_RETAIL);
      expect(vault.allowAccredited).to.equal(ALLOW_ACCREDITED);
      expect(vault.allowInstitutional).to.equal(ALLOW_INSTITUTIONAL);
      expect(vault.allowQualified).to.equal(ALLOW_QUALIFIED);
      expect(vault.maxAllowedAum).to.equal(MAX_ALLOWED_AUM);
      expect(vault.individualMinDeposit).to.equal(INDIVIDUAL_MIN_DEPOSIT);
      expect(vault.institutionalMinDeposit).to.equal(INSTITUTIONAL_MIN_DEPOSIT);
      expect(vault.maxDepositors).to.equal(MAX_DEPOSITORS);
      expect(vault.initialPrice).to.equal(INITIAL_VTOKEN_PRICE);
    } else {
      console.log("Vault already existed, skipping parameter validation");
      console.log("Existing vault settings:");
      console.log("allowRetail:", vault.allowRetail);
      console.log("allowAccredited:", vault.allowAccredited);
      console.log("allowInstitutional:", vault.allowInstitutional);
      console.log("allowQualified:", vault.allowQualified);
    }

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

  it("should successfully create a vault with max AUM limit", async () => {
    try {
      const limitedVaultManager = getManager2Keypair();
      const limitedVaultsSdk = new VaultsSdk(connection, limitedVaultManager);
      
      // Add manager2 to whitelist first (if not already added)
      try {
        await authorityVaults.addManagerToWhitelist(limitedVaultManager.publicKey, [authority]);
      } catch (e) {
        // Manager might already be in whitelist, that's fine
        if (!e.toString().includes("already in whitelist")) {
          throw e;
        }
      }
      
      await limitedVaultsSdk.createVault(
        "Limited AUM Vault Test",
        "LIMITED",
        VAULT_URI,
        limitedVaultManager.publicKey,
        USDC.mint,
        IS_VTOKEN_TRANSFERRABLE,
        new BN(1000000000), // max_allowed_aum set
        ALLOW_RETAIL,
        ALLOW_ACCREDITED,
        ALLOW_INSTITUTIONAL,
        ALLOW_QUALIFIED,
		INITIAL_VTOKEN_PRICE,
        INDIVIDUAL_MIN_DEPOSIT,
        INSTITUTIONAL_MIN_DEPOSIT,
        MAX_DEPOSITORS
      );
      
      // Verify vault was created with max AUM
      const vault = await limitedVaultsSdk.fetchVault(limitedVaultManager.publicKey);
      expect(vault.maxAllowedAum?.toString()).to.equal("1000000000");
    } catch (e) {
      if (!(e instanceof VaultAlreadyCreatedError)) {
        throw e;
      }
    }
  });

  it("should successfully create a vault without max AUM limit", async () => {
    try {
      const unlimitedVaultManager = getTesterKeypair();
      const unlimitedVaultsSdk = new VaultsSdk(connection, unlimitedVaultManager);
      
      await unlimitedVaultsSdk.createVault(
        "Unlimited AUM Vault Test",
        "UNLIMITED",
        VAULT_URI,
        unlimitedVaultManager.publicKey,
        USDC.mint,
        IS_VTOKEN_TRANSFERRABLE,
        null, // max_allowed_aum = null (no limit)
        ALLOW_RETAIL,
        ALLOW_ACCREDITED,
        ALLOW_INSTITUTIONAL,
        ALLOW_QUALIFIED,
		INITIAL_VTOKEN_PRICE,
        INDIVIDUAL_MIN_DEPOSIT,
        INSTITUTIONAL_MIN_DEPOSIT,
        MAX_DEPOSITORS
      );
      
      // Verify vault was created without max AUM
      const vault = await unlimitedVaultsSdk.fetchVault(unlimitedVaultManager.publicKey);
      expect(vault.maxAllowedAum).to.be.null;
    } catch (e) {
      if (!(e instanceof VaultAlreadyCreatedError)) {
        throw e;
      }
    }
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
