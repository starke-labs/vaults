import { BN } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import {
  InvestorTypeNotAllowedError,
  MaxDepositorsExceededError,
  SignatureVerificationFailedError,
  UserNotWhitelistedError,
} from "@starke/sdk/lib/errors";
import { InvestorType } from "@starke/sdk/lib/types";
import { USDC } from "@starke/sdk/whitelist";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
  getTesterKeypair,
} from "../utils.new";

describe("Investor Type Validation Tests", () => {
  let authority: Keypair;
  let manager: Keypair;
  let retailInvestor: Keypair;
  let accreditedInvestor: Keypair;
  let institutionalInvestor: Keypair;
  let qualifiedInvestor: Keypair;
  let unauthorizedUser: Keypair;

  let authorityVaults: VaultsSdk;
  let managerVaults: VaultsSdk;
  let retailVaults: VaultsSdk;
  let accreditedVaults: VaultsSdk;
  let institutionalVaults: VaultsSdk;
  let qualifiedVaults: VaultsSdk;
  let unauthorizedVaults: VaultsSdk;

  const DEPOSIT_AMOUNT = new BN(5 * 10 ** 6); // 5 USDC

  before(async () => {
    // Generate keypairs
    authority = getAuthorityKeypair();
    manager = getManagerKeypair();
    retailInvestor = getTesterKeypair();
    accreditedInvestor = Keypair.generate();
    institutionalInvestor = Keypair.generate();
    qualifiedInvestor = Keypair.generate();
    unauthorizedUser = Keypair.generate();

    // Initialize SDK instances
    const connection = createConnection();
    authorityVaults = new VaultsSdk(connection, authority);
    managerVaults = new VaultsSdk(connection, manager);
    retailVaults = new VaultsSdk(connection, retailInvestor);
    accreditedVaults = new VaultsSdk(connection, accreditedInvestor);
    institutionalVaults = new VaultsSdk(connection, institutionalInvestor);
    qualifiedVaults = new VaultsSdk(connection, qualifiedInvestor);
    unauthorizedVaults = new VaultsSdk(connection, unauthorizedUser);

    // Add users to whitelist with different investor types
    await authorityVaults.addUserToWhitelist(
      retailInvestor.publicKey,
      InvestorType.Retail,
      [authority]
    );
    await authorityVaults.addUserToWhitelist(
      accreditedInvestor.publicKey,
      InvestorType.Accredited,
      [authority]
    );
    await authorityVaults.addUserToWhitelist(
      institutionalInvestor.publicKey,
      InvestorType.Institutional,
      [authority]
    );
    await authorityVaults.addUserToWhitelist(
      qualifiedInvestor.publicKey,
      InvestorType.Qualified,
      [authority]
    );
    // Note: unauthorizedUser is NOT added to whitelist
  });

  describe("Vault with All Investor Types Allowed", () => {
    let allTypesManager: Keypair;
    let allTypesVaults: VaultsSdk;

    before(async () => {
      allTypesManager = Keypair.generate();
      allTypesVaults = new VaultsSdk(createConnection(), allTypesManager);

      // Add manager to whitelist
      await authorityVaults.addManagerToWhitelist(allTypesManager.publicKey, [authority]);

      // Create vault allowing all investor types
      await allTypesVaults.createVault(
        "All Types Vault",
        "ALLTYPES",
        "https://example.com/metadata.json",
        allTypesManager.publicKey,
        USDC.mint,
        true,
        null, // maxAllowedAum
        true, // allowRetail
        true, // allowAccredited
        true, // allowInstitutional
        true, // allowQualified
        1000000, // individualMinDeposit (1 USDC)
        10000000, // institutionalMinDeposit (10 USDC)
        0, // maxDepositors (0 = unlimited)
        [allTypesManager]
      );
    });

    it("should allow retail investor to deposit", async () => {
      const signature = await retailVaults.deposit(
        DEPOSIT_AMOUNT,
        retailInvestor.publicKey,
        allTypesManager.publicKey,
        [retailInvestor, authority]
      );
      expect(signature).to.not.be.undefined;
    });

    it("should allow accredited investor to deposit", async () => {
      const signature = await accreditedVaults.deposit(
        DEPOSIT_AMOUNT,
        accreditedInvestor.publicKey,
        allTypesManager.publicKey,
        [accreditedInvestor, authority]
      );
      expect(signature).to.not.be.undefined;
    });

    it("should allow institutional investor to deposit", async () => {
      const signature = await institutionalVaults.deposit(
        DEPOSIT_AMOUNT.mul(new BN(2)), // 10 USDC to meet institutional minimum
        institutionalInvestor.publicKey,
        allTypesManager.publicKey,
        [institutionalInvestor, authority]
      );
      expect(signature).to.not.be.undefined;
    });

    it("should allow qualified investor to deposit", async () => {
      const signature = await qualifiedVaults.deposit(
        DEPOSIT_AMOUNT.mul(new BN(2)), // 10 USDC to meet institutional minimum
        qualifiedInvestor.publicKey,
        allTypesManager.publicKey,
        [qualifiedInvestor, authority]
      );
      expect(signature).to.not.be.undefined;
    });

    it("should reject unauthorized user", async () => {
      try {
        await unauthorizedVaults.deposit(
          DEPOSIT_AMOUNT,
          unauthorizedUser.publicKey,
          allTypesManager.publicKey,
          [unauthorizedUser, authority]
        );
        expect.fail("Should have rejected unauthorized user");
      } catch (e) {
        expect(e).to.be.instanceOf(UserNotWhitelistedError);
      }
    });
  });

  describe("Vault with Only Retail Investors Allowed", () => {
    let retailOnlyManager: Keypair;
    let retailOnlyVaults: VaultsSdk;

    before(async () => {
      retailOnlyManager = Keypair.generate();
      retailOnlyVaults = new VaultsSdk(createConnection(), retailOnlyManager);

      // Add manager to whitelist
      await authorityVaults.addManagerToWhitelist(retailOnlyManager.publicKey, [authority]);

      // Create vault allowing only retail investors
      await retailOnlyVaults.createVault(
        "Retail Only Vault",
        "RETAIL",
        "https://example.com/metadata.json",
        retailOnlyManager.publicKey,
        USDC.mint,
        true,
        null, // maxAllowedAum
        true, // allowRetail - ONLY retail allowed
        false, // allowAccredited
        false, // allowInstitutional
        false, // allowQualified
        1000000, // individualMinDeposit (1 USDC)
        10000000, // institutionalMinDeposit (10 USDC)
        0, // maxDepositors (0 = unlimited)
        [retailOnlyManager]
      );
    });

    it("should allow retail investor to deposit", async () => {
      const signature = await retailVaults.deposit(
        DEPOSIT_AMOUNT,
        retailInvestor.publicKey,
        retailOnlyManager.publicKey,
        [retailInvestor, authority]
      );
      expect(signature).to.not.be.undefined;
    });

    it("should reject accredited investor", async () => {
      try {
        await accreditedVaults.deposit(
          DEPOSIT_AMOUNT,
          accreditedInvestor.publicKey,
          retailOnlyManager.publicKey,
          [accreditedInvestor, authority]
        );
        expect.fail("Should have rejected accredited investor");
      } catch (e) {
        expect(e).to.be.instanceOf(InvestorTypeNotAllowedError);
      }
    });

    it("should reject institutional investor", async () => {
      try {
        await institutionalVaults.deposit(
          DEPOSIT_AMOUNT.mul(new BN(2)),
          institutionalInvestor.publicKey,
          retailOnlyManager.publicKey,
          [institutionalInvestor, authority]
        );
        expect.fail("Should have rejected institutional investor");
      } catch (e) {
        expect(e).to.be.instanceOf(InvestorTypeNotAllowedError);
      }
    });

    it("should reject qualified investor", async () => {
      try {
        await qualifiedVaults.deposit(
          DEPOSIT_AMOUNT.mul(new BN(2)),
          qualifiedInvestor.publicKey,
          retailOnlyManager.publicKey,
          [qualifiedInvestor, authority]
        );
        expect.fail("Should have rejected qualified investor");
      } catch (e) {
        expect(e).to.be.instanceOf(InvestorTypeNotAllowedError);
      }
    });
  });

  describe("Vault with Only Institutional Investors Allowed", () => {
    let institutionalOnlyManager: Keypair;
    let institutionalOnlyVaults: VaultsSdk;

    before(async () => {
      institutionalOnlyManager = Keypair.generate();
      institutionalOnlyVaults = new VaultsSdk(createConnection(), institutionalOnlyManager);

      // Add manager to whitelist
      await authorityVaults.addManagerToWhitelist(institutionalOnlyManager.publicKey, [authority]);

      // Create vault allowing only institutional investors
      await institutionalOnlyVaults.createVault(
        "Institutional Only Vault",
        "INSTITUTIONAL",
        "https://example.com/metadata.json",
        institutionalOnlyManager.publicKey,
        USDC.mint,
        true,
        new BN(1000000000000), // maxAllowedAum
        false, // allowRetail
        false, // allowAccredited
        true, // allowInstitutional - ONLY institutional allowed
        false, // allowQualified
        1000000, // individualMinDeposit (1 USDC)
        50000000, // institutionalMinDeposit (50 USDC)
        0, // maxDepositors (0 = unlimited)
        [institutionalOnlyManager]
      );
    });

    it("should allow institutional investor to deposit", async () => {
      const signature = await institutionalVaults.deposit(
        DEPOSIT_AMOUNT.mul(new BN(10)), // 50 USDC to meet institutional minimum
        institutionalInvestor.publicKey,
        institutionalOnlyManager.publicKey,
        [institutionalInvestor, authority]
      );
      expect(signature).to.not.be.undefined;
    });

    it("should reject retail investor", async () => {
      try {
        await retailVaults.deposit(
          DEPOSIT_AMOUNT,
          retailInvestor.publicKey,
          institutionalOnlyManager.publicKey,
          [retailInvestor, authority]
        );
        expect.fail("Should have rejected retail investor");
      } catch (e) {
        expect(e).to.be.instanceOf(InvestorTypeNotAllowedError);
      }
    });

    it("should reject accredited investor", async () => {
      try {
        await accreditedVaults.deposit(
          DEPOSIT_AMOUNT,
          accreditedInvestor.publicKey,
          institutionalOnlyManager.publicKey,
          [accreditedInvestor, authority]
        );
        expect.fail("Should have rejected accredited investor");
      } catch (e) {
        expect(e).to.be.instanceOf(InvestorTypeNotAllowedError);
      }
    });
  });

  describe("Max Depositors Validation", () => {
    let limitedManager: Keypair;
    let limitedVaults: VaultsSdk;
    let extraUser1: Keypair;
    let extraUser2: Keypair;
    let extraUser3: Keypair;

    before(async () => {
      limitedManager = Keypair.generate();
      limitedVaults = new VaultsSdk(createConnection(), limitedManager);
      extraUser1 = Keypair.generate();
      extraUser2 = Keypair.generate();
      extraUser3 = Keypair.generate();

      // Add manager to whitelist
      await authorityVaults.addManagerToWhitelist(limitedManager.publicKey, [authority]);

      // Add extra users to whitelist
      await authorityVaults.addUserToWhitelist(
        extraUser1.publicKey,
        InvestorType.Retail,
        [authority]
      );
      await authorityVaults.addUserToWhitelist(
        extraUser2.publicKey,
        InvestorType.Retail,
        [authority]
      );
      await authorityVaults.addUserToWhitelist(
        extraUser3.publicKey,
        InvestorType.Retail,
        [authority]
      );

      // Create vault with max 2 depositors
      await limitedVaults.createVault(
        "Limited Depositors Vault",
        "LIMITED",
        "https://example.com/metadata.json",
        limitedManager.publicKey,
        USDC.mint,
        true,
        null, // maxAllowedAum
        true, // allowRetail
        true, // allowAccredited
        true, // allowInstitutional
        true, // allowQualified
        1000000, // individualMinDeposit (1 USDC)
        10000000, // institutionalMinDeposit (10 USDC)
        2, // maxDepositors - only 2 allowed
        [limitedManager]
      );
    });

    it("should allow first depositor", async () => {
      const extraVaults1 = new VaultsSdk(createConnection(), extraUser1);
      const signature = await extraVaults1.deposit(
        DEPOSIT_AMOUNT,
        extraUser1.publicKey,
        limitedManager.publicKey,
        [extraUser1, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count increased
      const vault = await limitedVaults.fetchVault(limitedManager.publicKey);
      expect(vault.currentDepositors).to.equal(1);
    });

    it("should allow second depositor", async () => {
      const extraVaults2 = new VaultsSdk(createConnection(), extraUser2);
      const signature = await extraVaults2.deposit(
        DEPOSIT_AMOUNT,
        extraUser2.publicKey,
        limitedManager.publicKey,
        [extraUser2, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count increased
      const vault = await limitedVaults.fetchVault(limitedManager.publicKey);
      expect(vault.currentDepositors).to.equal(2);
    });

    it("should reject third depositor (exceeds max)", async () => {
      const extraVaults3 = new VaultsSdk(createConnection(), extraUser3);
      try {
        await extraVaults3.deposit(
          DEPOSIT_AMOUNT,
          extraUser3.publicKey,
          limitedManager.publicKey,
          [extraUser3, authority]
        );
        expect.fail("Should have rejected third depositor");
      } catch (e) {
        expect(e).to.be.instanceOf(MaxDepositorsExceededError);
      }

      // Verify depositor count unchanged
      const vault = await limitedVaults.fetchVault(limitedManager.publicKey);
      expect(vault.currentDepositors).to.equal(2);
    });

    it("should allow existing depositor to deposit more", async () => {
      const extraVaults1 = new VaultsSdk(createConnection(), extraUser1);
      const signature = await extraVaults1.deposit(
        DEPOSIT_AMOUNT,
        extraUser1.publicKey,
        limitedManager.publicKey,
        [extraUser1, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count unchanged (same user, not new)
      const vault = await limitedVaults.fetchVault(limitedManager.publicKey);
      expect(vault.currentDepositors).to.equal(2);
    });
  });

  describe("Authority Signature Validation", () => {
    it("should require authority signature for deposits", async () => {
      try {
        await retailVaults.deposit(
          DEPOSIT_AMOUNT,
          retailInvestor.publicKey,
          manager.publicKey,
          [retailInvestor] // Missing authority signature
        );
        expect.fail("Should have required authority signature");
      } catch (e) {
        expect(e).to.be.instanceOf(SignatureVerificationFailedError);
      }
    });

    it("should require user signature for deposits", async () => {
      try {
        await retailVaults.deposit(
          DEPOSIT_AMOUNT,
          retailInvestor.publicKey,
          manager.publicKey,
          [authority] // Missing user signature
        );
        expect.fail("Should have required user signature");
      } catch (e) {
        expect(e).to.be.instanceOf(SignatureVerificationFailedError);
      }
    });
  });
});