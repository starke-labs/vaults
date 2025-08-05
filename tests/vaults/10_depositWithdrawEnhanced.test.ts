import { BN } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import {
  DepositBelowMinimumError,
  InvestorTypeNotAllowedError,
  MaxDepositorsExceededError,
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

describe("Enhanced Deposit/Withdraw Tests", () => {
  let authority: Keypair;
  let manager: Keypair;
  let retailInvestor: Keypair;
  let institutionalInvestor: Keypair;
  let testVault: Keypair;

  let authorityVaults: VaultsSdk;
  let managerVaults: VaultsSdk;
  let retailVaults: VaultsSdk;
  let institutionalVaults: VaultsSdk;
  let testVaultSdk: VaultsSdk;

  before(async () => {
    // Generate keypairs
    authority = getAuthorityKeypair();
    manager = getManagerKeypair();
    retailInvestor = getTesterKeypair();
    institutionalInvestor = Keypair.generate();
    testVault = Keypair.generate();

    // Initialize SDK instances
    const connection = createConnection();
    authorityVaults = new VaultsSdk(connection, authority);
    managerVaults = new VaultsSdk(connection, manager);
    retailVaults = new VaultsSdk(connection, retailInvestor);
    institutionalVaults = new VaultsSdk(connection, institutionalInvestor);
    testVaultSdk = new VaultsSdk(connection, testVault);

    // Add test vault manager to whitelist
    await authorityVaults.addManagerToWhitelist(testVault.publicKey, [authority]);

    // Add users to whitelist
    await authorityVaults.addUserToWhitelist(
      retailInvestor.publicKey,
      InvestorType.Retail,
      [authority]
    );
    await authorityVaults.addUserToWhitelist(
      institutionalInvestor.publicKey,
      InvestorType.Institutional,
      [authority]
    );

    // Create test vault with specific parameters
    await testVaultSdk.createVault(
      "Enhanced Test Vault",
      "ENHANCED",
      "https://example.com/metadata.json",
      testVault.publicKey,
      USDC.mint,
      true,
      null,
      true, // allowRetail
      true, // allowAccredited
      true, // allowInstitutional
      true, // allowQualified
      2000000, // individualMinDeposit (2 USDC)
      20000000, // institutionalMinDeposit (20 USDC)
      3, // maxDepositors
      [testVault]
    );
  });

  describe("Minimum Deposit Validation", () => {
    it("should reject retail investor deposit below individual minimum", async () => {
      try {
        await retailVaults.deposit(
          new BN(1000000), // 1 USDC - below 2 USDC minimum
          retailInvestor.publicKey,
          testVault.publicKey,
          [retailInvestor, authority]
        );
        expect.fail("Should have rejected deposit below minimum");
      } catch (e) {
        expect(e).to.be.instanceOf(DepositBelowMinimumError);
      }
    });

    it("should accept retail investor deposit above individual minimum", async () => {
      const signature = await retailVaults.deposit(
        new BN(3000000), // 3 USDC - above 2 USDC minimum
        retailInvestor.publicKey,
        testVault.publicKey,
        [retailInvestor, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count increased
      const vault = await testVaultSdk.fetchVault(testVault.publicKey);
      expect(vault.currentDepositors).to.equal(1);
    });

    it("should reject institutional investor deposit below institutional minimum", async () => {
      try {
        await institutionalVaults.deposit(
          new BN(10000000), // 10 USDC - below 20 USDC institutional minimum
          institutionalInvestor.publicKey,
          testVault.publicKey,
          [institutionalInvestor, authority]
        );
        expect.fail("Should have rejected institutional deposit below minimum");
      } catch (e) {
        expect(e).to.be.instanceOf(DepositBelowMinimumError);
      }
    });

    it("should accept institutional investor deposit above institutional minimum", async () => {
      const signature = await institutionalVaults.deposit(
        new BN(25000000), // 25 USDC - above 20 USDC institutional minimum
        institutionalInvestor.publicKey,
        testVault.publicKey,
        [institutionalInvestor, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count increased
      const vault = await testVaultSdk.fetchVault(testVault.publicKey);
      expect(vault.currentDepositors).to.equal(2);
    });
  });

  describe("Depositor Tracking", () => {
    let thirdUser: Keypair;
    let fourthUser: Keypair;
    let thirdVaults: VaultsSdk;
    let fourthVaults: VaultsSdk;

    before(async () => {
      thirdUser = Keypair.generate();
      fourthUser = Keypair.generate();
      thirdVaults = new VaultsSdk(createConnection(), thirdUser);
      fourthVaults = new VaultsSdk(createConnection(), fourthUser);

      // Add users to whitelist
      await authorityVaults.addUserToWhitelist(
        thirdUser.publicKey,
        InvestorType.Retail,
        [authority]
      );
      await authorityVaults.addUserToWhitelist(
        fourthUser.publicKey,
        InvestorType.Retail,
        [authority]
      );
    });

    it("should accept third depositor (at limit)", async () => {
      const signature = await thirdVaults.deposit(
        new BN(3000000), // 3 USDC
        thirdUser.publicKey,
        testVault.publicKey,
        [thirdUser, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count at limit
      const vault = await testVaultSdk.fetchVault(testVault.publicKey);
      expect(vault.currentDepositors).to.equal(3);
    });

    it("should reject fourth depositor (exceeds limit)", async () => {
      try {
        await fourthVaults.deposit(
          new BN(3000000), // 3 USDC
          fourthUser.publicKey,
          testVault.publicKey,
          [fourthUser, authority]
        );
        expect.fail("Should have rejected fourth depositor");
      } catch (e) {
        expect(e).to.be.instanceOf(MaxDepositorsExceededError);
      }

      // Verify depositor count unchanged
      const vault = await testVaultSdk.fetchVault(testVault.publicKey);
      expect(vault.currentDepositors).to.equal(3);
    });

    it("should allow existing depositor to deposit more", async () => {
      const signature = await retailVaults.deposit(
        new BN(2000000), // 2 USDC more
        retailInvestor.publicKey,
        testVault.publicKey,
        [retailInvestor, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count unchanged (same user)
      const vault = await testVaultSdk.fetchVault(testVault.publicKey);
      expect(vault.currentDepositors).to.equal(3);
    });
  });

  describe("Withdrawal and Depositor Count Management", () => {
    it("should maintain depositor count when partial withdrawal", async () => {
      // Get initial vToken balance
      const initialBalance = await retailVaults.getVtokenBalance(
        await testVaultSdk.fetchVault(testVault.publicKey).then(v => v.mint),
        retailInvestor.publicKey
      );

      // Partial withdrawal (half of balance)
      const partialAmount = initialBalance.div(new BN(2));
      const signature = await retailVaults.withdraw(
        partialAmount,
        retailInvestor.publicKey,
        testVault.publicKey,
        [retailInvestor, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count unchanged (user still has balance)
      const vault = await testVaultSdk.fetchVault(testVault.publicKey);
      expect(vault.currentDepositors).to.equal(3);
    });

    it("should decrease depositor count when full withdrawal", async () => {
      // Get remaining vToken balance
      const vault = await testVaultSdk.fetchVault(testVault.publicKey);
      const remainingBalance = await retailVaults.getVtokenBalance(
        vault.mint,
        retailInvestor.publicKey
      );

      // Full withdrawal
      const signature = await retailVaults.withdraw(
        remainingBalance,
        retailInvestor.publicKey,
        testVault.publicKey,
        [retailInvestor, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count decreased
      const updatedVault = await testVaultSdk.fetchVault(testVault.publicKey);
      expect(updatedVault.currentDepositors).to.equal(2);
    });

    it("should allow new depositor after someone fully withdrew", async () => {
      // Fourth user should now be able to deposit since retail investor fully withdrew
      const fourthVaults = new VaultsSdk(createConnection(), Keypair.generate());
      const fourthUser = fourthVaults.provider.wallet.publicKey;

      // Add fourth user to whitelist
      await authorityVaults.addUserToWhitelist(
        fourthUser,
        InvestorType.Retail,
        [authority]
      );

      const signature = await fourthVaults.deposit(
        new BN(3000000), // 3 USDC
        fourthUser,
        testVault.publicKey,
        [fourthVaults.provider.wallet.payer, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count back to 3
      const vault = await testVaultSdk.fetchVault(testVault.publicKey);
      expect(vault.currentDepositors).to.equal(3);
    });
  });

  describe("Investor Type Specific Scenarios", () => {
    let restrictedVault: Keypair;
    let restrictedVaultSdk: VaultsSdk;
    let retailOnly: Keypair;
    let institutionalOnly: Keypair;
    let retailOnlyVaults: VaultsSdk;
    let institutionalOnlyVaults: VaultsSdk;

    before(async () => {
      restrictedVault = Keypair.generate();
      restrictedVaultSdk = new VaultsSdk(createConnection(), restrictedVault);
      retailOnly = Keypair.generate();
      institutionalOnly = Keypair.generate();
      retailOnlyVaults = new VaultsSdk(createConnection(), retailOnly);
      institutionalOnlyVaults = new VaultsSdk(createConnection(), institutionalOnly);

      // Add managers to whitelist
      await authorityVaults.addManagerToWhitelist(restrictedVault.publicKey, [authority]);

      // Add users to whitelist
      await authorityVaults.addUserToWhitelist(
        retailOnly.publicKey,
        InvestorType.Retail,
        [authority]
      );
      await authorityVaults.addUserToWhitelist(
        institutionalOnly.publicKey,
        InvestorType.Institutional,
        [authority]
      );

      // Create vault that only allows retail investors
      await restrictedVaultSdk.createVault(
        "Retail Only Vault",
        "RETAIL",
        "https://example.com/metadata.json",
        restrictedVault.publicKey,
        USDC.mint,
        true,
        null,
        true, // allowRetail - ONLY retail
        false, // allowAccredited
        false, // allowInstitutional
        false, // allowQualified
        1000000, // individualMinDeposit (1 USDC)
        10000000, // institutionalMinDeposit (10 USDC)
        0, // unlimited depositors
        [restrictedVault]
      );
    });

    it("should allow retail investor in retail-only vault", async () => {
      const signature = await retailOnlyVaults.deposit(
        new BN(2000000), // 2 USDC
        retailOnly.publicKey,
        restrictedVault.publicKey,
        [retailOnly, authority]
      );
      expect(signature).to.not.be.undefined;
    });

    it("should reject institutional investor in retail-only vault", async () => {
      try {
        await institutionalOnlyVaults.deposit(
          new BN(15000000), // 15 USDC (meets institutional minimum)
          institutionalOnly.publicKey,
          restrictedVault.publicKey,
          [institutionalOnly, authority]
        );
        expect.fail("Should have rejected institutional investor");
      } catch (e) {
        expect(e).to.be.instanceOf(InvestorTypeNotAllowedError);
      }
    });
  });

  describe("Edge Cases and Error Handling", () => {
    it("should handle zero deposit amount", async () => {
      try {
        await retailVaults.deposit(
          new BN(0), // 0 USDC
          retailInvestor.publicKey,
          testVault.publicKey,
          [retailInvestor, authority]
        );
        expect.fail("Should have rejected zero deposit");
      } catch (e) {
        expect(e.toString()).to.include("InvalidAmount");
      }
    });

    it("should handle very large deposit amounts", async () => {
      const largeAmount = new BN("1000000000000000"); // Very large amount
      try {
        await retailVaults.deposit(
          largeAmount,
          retailInvestor.publicKey,
          testVault.publicKey,
          [retailInvestor, authority]
        );
        expect.fail("Should have failed due to insufficient balance");
      } catch (e) {
        // Should fail due to insufficient balance, not validation error
        expect(e.toString()).to.include("insufficient");
      }
    });

    it("should handle withdrawal of zero amount", async () => {
      try {
        await retailVaults.withdraw(
          new BN(0), // 0 vTokens
          retailInvestor.publicKey,
          testVault.publicKey,
          [retailInvestor, authority]
        );
        expect.fail("Should have rejected zero withdrawal");
      } catch (e) {
        expect(e.toString()).to.include("InvalidAmount");
      }
    });

    it("should handle withdrawal exceeding balance", async () => {
      const vault = await testVaultSdk.fetchVault(testVault.publicKey);
      const balance = await retailVaults.getVtokenBalance(
        vault.mint,
        retailInvestor.publicKey
      );
      const excessiveAmount = balance.add(new BN(1000000)); // More than balance

      try {
        await retailVaults.withdraw(
          excessiveAmount,
          retailInvestor.publicKey,
          testVault.publicKey,
          [retailInvestor, authority]
        );
        expect.fail("Should have failed due to insufficient vToken balance");
      } catch (e) {
        expect(e.toString()).to.include("insufficient");
      }
    });
  });

  describe("User Whitelist Integration", () => {
    it("should reject deposit from non-whitelisted user", async () => {
      const nonWhitelistedUser = Keypair.generate();
      const nonWhitelistedVaults = new VaultsSdk(createConnection(), nonWhitelistedUser);

      try {
        await nonWhitelistedVaults.deposit(
          new BN(3000000), // 3 USDC
          nonWhitelistedUser.publicKey,
          testVault.publicKey,
          [nonWhitelistedUser, authority]
        );
        expect.fail("Should have rejected non-whitelisted user");
      } catch (e) {
        expect(e).to.be.instanceOf(UserNotWhitelistedError);
      }
    });

    it("should allow deposit after adding user to whitelist", async () => {
      const newUser = Keypair.generate();
      const newUserVaults = new VaultsSdk(createConnection(), newUser);

      // Initially should fail
      try {
        await newUserVaults.deposit(
          new BN(3000000), // 3 USDC
          newUser.publicKey,
          testVault.publicKey,
          [newUser, authority]
        );
        expect.fail("Should have rejected non-whitelisted user");
      } catch (e) {
        expect(e).to.be.instanceOf(UserNotWhitelistedError);
      }

      // Add to whitelist
      await authorityVaults.addUserToWhitelist(
        newUser.publicKey,
        InvestorType.Retail,
        [authority]
      );

      // Should now succeed (but may fail due to max depositors)
      try {
        const signature = await newUserVaults.deposit(
          new BN(3000000), // 3 USDC
          newUser.publicKey,
          testVault.publicKey,
          [newUser, authority]
        );
        expect(signature).to.not.be.undefined;
      } catch (e) {
        // If max depositors reached, that's expected
        if (!(e instanceof MaxDepositorsExceededError)) {
          throw e;
        }
      }
    });
  });
});