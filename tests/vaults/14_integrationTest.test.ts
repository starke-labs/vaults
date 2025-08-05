import { BN } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import {
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

describe("Integration Test: Complete Financial Rules Flow", () => {
  let authority: Keypair;
  let fundManager: Keypair;
  let retailInvestor1: Keypair;
  let retailInvestor2: Keypair;
  let accreditedInvestor: Keypair;
  let institutionalInvestor: Keypair;
  let qualifiedInvestor: Keypair;
  let unauthorizedUser: Keypair;

  let authorityVaults: VaultsSdk;
  let managerVaults: VaultsSdk;
  let retail1Vaults: VaultsSdk;
  let retail2Vaults: VaultsSdk;
  let accreditedVaults: VaultsSdk;
  let institutionalVaults: VaultsSdk;
  let qualifiedVaults: VaultsSdk;
  let unauthorizedVaults: VaultsSdk;

  before(async () => {
    // Generate all keypairs
    authority = getAuthorityKeypair();
    fundManager = getManagerKeypair();
    retailInvestor1 = getTesterKeypair();
    retailInvestor2 = Keypair.generate();
    accreditedInvestor = Keypair.generate();
    institutionalInvestor = Keypair.generate();
    qualifiedInvestor = Keypair.generate();
    unauthorizedUser = Keypair.generate();

    // Initialize all SDK instances
    const connection = createConnection();
    authorityVaults = new VaultsSdk(connection, authority);
    managerVaults = new VaultsSdk(connection, fundManager);
    retail1Vaults = new VaultsSdk(connection, retailInvestor1);
    retail2Vaults = new VaultsSdk(connection, retailInvestor2);
    accreditedVaults = new VaultsSdk(connection, accreditedInvestor);
    institutionalVaults = new VaultsSdk(connection, institutionalInvestor);
    qualifiedVaults = new VaultsSdk(connection, qualifiedInvestor);
    unauthorizedVaults = new VaultsSdk(connection, unauthorizedUser);
  });

  describe("Complete Fund Management Scenario", () => {
    it("Step 1: Authority sets up user whitelist with various investor types", async () => {
      // Verify user whitelist was initialized during system setup
      const userWhitelist = await authorityVaults.fetchUserWhitelist();
      expect(userWhitelist.authority.toBase58()).to.equal(authority.publicKey.toBase58());

      // Add users with different investor types
      await authorityVaults.addUserToWhitelist(
        retailInvestor1.publicKey,
        InvestorType.Retail,
        [authority]
      );
      await authorityVaults.addUserToWhitelist(
        retailInvestor2.publicKey,
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

      // Verify all users were added
      const updatedWhitelist = await authorityVaults.fetchUserWhitelist();
      expect(updatedWhitelist.users.length).to.be.at.least(5);

      // Verify specific user types
      const retail1Entry = updatedWhitelist.users.find(
        u => u.user.toBase58() === retailInvestor1.publicKey.toBase58()
      );
      expect(retail1Entry.investorType).to.equal(InvestorType.Retail);

      const institutionalEntry = updatedWhitelist.users.find(
        u => u.user.toBase58() === institutionalInvestor.publicKey.toBase58()
      );
      expect(institutionalEntry.investorType).to.equal(InvestorType.Institutional);
    });

    it("Step 2: Fund manager creates a sophisticated private vault", async () => {
      // Create a private vault with complex rules:
      // - Only accredited, institutional, and qualified investors
      // - Different minimum deposits for individual vs institutional
      // - Maximum 5 depositors
      // - Maximum AUM of $500,000
      const signature = await managerVaults.createVault(
        "Sophisticated Private Fund",
        "SPF",
        "https://example.com/spf-metadata.json",
        fundManager.publicKey,
        USDC.mint,
        false, // vTokens not transferrable
        500000000000, // maxAllowedAum: $500,000 in AUM decimals
        false, // allowRetail - NOT ALLOWED
        true, // allowAccredited
        true, // allowInstitutional
        true, // allowQualified
        10000000, // individualMinDeposit: 10 USDC for accredited
        100000000, // institutionalMinDeposit: 100 USDC for institutional
        5, // maxDepositors
        [fundManager]
      );
      expect(signature).to.not.be.undefined;

      // Verify vault configuration
      const vault = await managerVaults.fetchVault(fundManager.publicKey);
      expect(vault.isPrivateVault).to.be.true;
      expect(vault.allowRetail).to.be.false;
      expect(vault.allowAccredited).to.be.true;
      expect(vault.allowInstitutional).to.be.true;
      expect(vault.allowQualified).to.be.true;
      expect(vault.individualMinDeposit).to.equal(10000000);
      expect(vault.institutionalMinDeposit).to.equal(100000000);
      expect(vault.maxDepositors).to.equal(5);
      expect(vault.currentDepositors).to.equal(0);
    });

    it("Step 3: Retail investors should be rejected (not allowed)", async () => {
      // Both retail investors should be rejected
      try {
        await retail1Vaults.deposit(
          new BN(15000000), // 15 USDC - above minimums
          retailInvestor1.publicKey,
          fundManager.publicKey,
          [retailInvestor1, authority]
        );
        expect.fail("Retail investor should be rejected");
      } catch (e) {
        expect(e).to.be.instanceOf(InvestorTypeNotAllowedError);
      }

      try {
        await retail2Vaults.deposit(
          new BN(15000000), // 15 USDC - above minimums
          retailInvestor2.publicKey,
          fundManager.publicKey,
          [retailInvestor2, authority]
        );
        expect.fail("Retail investor should be rejected");
      } catch (e) {
        expect(e).to.be.instanceOf(InvestorTypeNotAllowedError);
      }
    });

    it("Step 4: Unauthorized user should be rejected (not whitelisted)", async () => {
      try {
        await unauthorizedVaults.deposit(
          new BN(50000000), // 50 USDC
          unauthorizedUser.publicKey,
          fundManager.publicKey,
          [unauthorizedUser, authority]
        );
        expect.fail("Unauthorized user should be rejected");
      } catch (e) {
        expect(e).to.be.instanceOf(UserNotWhitelistedError);
      }
    });

    it("Step 5: Accredited investor makes successful deposit", async () => {
      const signature = await accreditedVaults.deposit(
        new BN(15000000), // 15 USDC - above 10 USDC individual minimum
        accreditedInvestor.publicKey,
        fundManager.publicKey,
        [accreditedInvestor, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count increased
      const vault = await managerVaults.fetchVault(fundManager.publicKey);
      expect(vault.currentDepositors).to.equal(1);
    });

    it("Step 6: Institutional investor makes successful deposit", async () => {
      const signature = await institutionalVaults.deposit(
        new BN(150000000), // 150 USDC - above 100 USDC institutional minimum
        institutionalInvestor.publicKey,
        fundManager.publicKey,
        [institutionalInvestor, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count increased
      const vault = await managerVaults.fetchVault(fundManager.publicKey);
      expect(vault.currentDepositors).to.equal(2);
    });

    it("Step 7: Qualified investor makes successful deposit", async () => {
      const signature = await qualifiedVaults.deposit(
        new BN(200000000), // 200 USDC - above institutional minimum (qualified uses institutional rules)
        qualifiedInvestor.publicKey,
        fundManager.publicKey,
        [qualifiedInvestor, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count increased
      const vault = await managerVaults.fetchVault(fundManager.publicKey);
      expect(vault.currentDepositors).to.equal(3);
    });

    it("Step 8: Test depositor limits by adding two more users", async () => {
      // Create two more qualified investors
      const qualifiedInvestor2 = Keypair.generate();
      const qualifiedInvestor3 = Keypair.generate();
      const qualified2Vaults = new VaultsSdk(createConnection(), qualifiedInvestor2);
      const qualified3Vaults = new VaultsSdk(createConnection(), qualifiedInvestor3);

      // Add them to whitelist
      await authorityVaults.addUserToWhitelist(
        qualifiedInvestor2.publicKey,
        InvestorType.Qualified,
        [authority]
      );
      await authorityVaults.addUserToWhitelist(
        qualifiedInvestor3.publicKey,
        InvestorType.Qualified,
        [authority]
      );

      // Fourth depositor (should succeed)
      await qualified2Vaults.deposit(
        new BN(150000000), // 150 USDC
        qualifiedInvestor2.publicKey,
        fundManager.publicKey,
        [qualifiedInvestor2, authority]
      );

      // Fifth depositor (should succeed - at limit)
      await qualified3Vaults.deposit(
        new BN(150000000), // 150 USDC
        qualifiedInvestor3.publicKey,
        fundManager.publicKey,
        [qualifiedInvestor3, authority]
      );

      // Verify we're at the limit
      const vault = await managerVaults.fetchVault(fundManager.publicKey);
      expect(vault.currentDepositors).to.equal(5);

      // Sixth depositor should be rejected
      const qualifiedInvestor4 = Keypair.generate();
      const qualified4Vaults = new VaultsSdk(createConnection(), qualifiedInvestor4);
      
      await authorityVaults.addUserToWhitelist(
        qualifiedInvestor4.publicKey,
        InvestorType.Qualified,
        [authority]
      );

      try {
        await qualified4Vaults.deposit(
          new BN(150000000), // 150 USDC
          qualifiedInvestor4.publicKey,
          fundManager.publicKey,
          [qualifiedInvestor4, authority]
        );
        expect.fail("Should have rejected sixth depositor");
      } catch (e) {
        expect(e).to.be.instanceOf(MaxDepositorsExceededError);
      }
    });

    it("Step 9: Test withdrawal and depositor count management", async () => {
      // Get accredited investor's vToken balance
      const vault = await managerVaults.fetchVault(fundManager.publicKey);
      const accreditedBalance = await accreditedVaults.getVtokenBalance(
        vault.mint,
        accreditedInvestor.publicKey
      );

      // Partial withdrawal (should not affect depositor count)
      const partialAmount = accreditedBalance.div(new BN(2));
      await accreditedVaults.withdraw(
        partialAmount,
        accreditedInvestor.publicKey,
        fundManager.publicKey,
        [accreditedInvestor, authority]
      );

      // Verify depositor count unchanged
      const vaultAfterPartial = await managerVaults.fetchVault(fundManager.publicKey);
      expect(vaultAfterPartial.currentDepositors).to.equal(5);

      // Full withdrawal (should decrease depositor count)
      const remainingBalance = await accreditedVaults.getVtokenBalance(
        vault.mint,
        accreditedInvestor.publicKey
      );
      
      await accreditedVaults.withdraw(
        remainingBalance,
        accreditedInvestor.publicKey,
        fundManager.publicKey,
        [accreditedInvestor, authority]
      );

      // Verify depositor count decreased
      const vaultAfterFull = await managerVaults.fetchVault(fundManager.publicKey);
      expect(vaultAfterFull.currentDepositors).to.equal(4);

      // Now a new depositor should be able to join
      const newQualifiedInvestor = Keypair.generate();
      const newQualifiedVaults = new VaultsSdk(createConnection(), newQualifiedInvestor);
      
      await authorityVaults.addUserToWhitelist(
        newQualifiedInvestor.publicKey,
        InvestorType.Qualified,
        [authority]
      );

      const signature = await newQualifiedVaults.deposit(
        new BN(150000000), // 150 USDC
        newQualifiedInvestor.publicKey,
        fundManager.publicKey,
        [newQualifiedInvestor, authority]
      );
      expect(signature).to.not.be.undefined;

      // Verify depositor count back to 5
      const finalVault = await managerVaults.fetchVault(fundManager.publicKey);
      expect(finalVault.currentDepositors).to.equal(5);
    });

    it("Step 10: Test investor type changes", async () => {
      // Change retail investor to accredited
      await authorityVaults.addUserToWhitelist(
        retailInvestor1.publicKey,
        InvestorType.Accredited, // Changed from Retail
        [authority]
      );

      // Verify the change
      const updatedWhitelist = await authorityVaults.fetchUserWhitelist();
      const updatedRetail1Entry = updatedWhitelist.users.find(
        u => u.user.toBase58() === retailInvestor1.publicKey.toBase58()
      );
      expect(updatedRetail1Entry.investorType).to.equal(InvestorType.Accredited);

      // Now retail investor (now accredited) should be able to deposit
      // But the vault is at max capacity, so should fail due to max depositors
      try {
        await retail1Vaults.deposit(
          new BN(15000000), // 15 USDC
          retailInvestor1.publicKey,
          fundManager.publicKey,
          [retailInvestor1, authority]
        );
        expect.fail("Should have failed due to max depositors");
      } catch (e) {
        expect(e).to.be.instanceOf(MaxDepositorsExceededError);
      }
    });

    it("Step 11: Verify final vault state", async () => {
      const finalVault = await managerVaults.fetchVault(fundManager.publicKey);
      const finalWhitelist = await authorityVaults.fetchUserWhitelist();

      // Vault should be at capacity
      expect(finalVault.currentDepositors).to.equal(5);
      expect(finalVault.maxDepositors).to.equal(5);

      // Whitelist should have all users
      expect(finalWhitelist.users.length).to.be.at.least(7); // Original 5 + 2 qualified investors

      // Verify investor type distribution
      const retailCount = finalWhitelist.users.filter(u => u.investorType === InvestorType.Retail).length;
      const accreditedCount = finalWhitelist.users.filter(u => u.investorType === InvestorType.Accredited).length;
      const institutionalCount = finalWhitelist.users.filter(u => u.investorType === InvestorType.Institutional).length;
      const qualifiedCount = finalWhitelist.users.filter(u => u.investorType === InvestorType.Qualified).length;

      expect(retailCount).to.equal(1); // retailInvestor2 (retailInvestor1 changed to accredited)
      expect(accreditedCount).to.equal(2); // accreditedInvestor + retailInvestor1 (changed)
      expect(institutionalCount).to.equal(1); // institutionalInvestor
      expect(qualifiedCount).to.be.at.least(4); // qualifiedInvestor + 3 additional qualified investors

      console.log("Final vault state:");
      console.log(`- Current depositors: ${finalVault.currentDepositors}/${finalVault.maxDepositors}`);
      console.log(`- Retail investors: ${retailCount}`);
      console.log(`- Accredited investors: ${accreditedCount}`);
      console.log(`- Institutional investors: ${institutionalCount}`);
      console.log(`- Qualified investors: ${qualifiedCount}`);
      console.log(`- Total whitelisted users: ${finalWhitelist.users.length}`);
    });
  });

  describe("Public Vault Scenario", () => {
    let publicVaultManager: Keypair;
    let publicVaultSdk: VaultsSdk;

    it("Should create and test public vault with different rules", async () => {
      publicVaultManager = Keypair.generate();
      publicVaultSdk = new VaultsSdk(createConnection(), publicVaultManager);

      // Add manager to whitelist
      await authorityVaults.addManagerToWhitelist(publicVaultManager.publicKey, [authority]);

      // Create public vault with different rules
      await publicVaultSdk.createVault(
        "Public Retail Fund",
        "PRF",
        "https://example.com/prf-metadata.json",
        publicVaultManager.publicKey,
        USDC.mint,
        true, // vTokens transferrable
        null, // no max AUM for public vault
        true, // allowRetail
        true, // allowAccredited
        false, // allowInstitutional - NOT ALLOWED
        false, // allowQualified - NOT ALLOWED
        1000000, // individualMinDeposit: 1 USDC (lower than private vault)
        50000000, // institutionalMinDeposit: 50 USDC (not applicable since institutional not allowed)
        0, // no max depositors
        [publicVaultManager]
      );

      // Verify vault configuration
      const vault = await publicVaultSdk.fetchVault(publicVaultManager.publicKey);
      expect(vault.isPrivateVault).to.be.false;
      expect(vault.allowRetail).to.be.true;
      expect(vault.allowAccredited).to.be.true;
      expect(vault.allowInstitutional).to.be.false;
      expect(vault.allowQualified).to.be.false;
      expect(vault.maxDepositors).to.equal(0);
      expect(vault.maxAllowedAum).to.be.null;

      // Retail investor should be able to deposit (changed back to retail)
      await authorityVaults.addUserToWhitelist(
        retailInvestor1.publicKey,
        InvestorType.Retail,
        [authority]
      );

      const signature = await retail1Vaults.deposit(
        new BN(2000000), // 2 USDC
        retailInvestor1.publicKey,
        publicVaultManager.publicKey,
        [retailInvestor1, authority]
      );
      expect(signature).to.not.be.undefined;

      // Institutional investor should be rejected
      try {
        await institutionalVaults.deposit(
          new BN(60000000), // 60 USDC
          institutionalInvestor.publicKey,
          publicVaultManager.publicKey,
          [institutionalInvestor, authority]
        );
        expect.fail("Institutional investor should be rejected");
      } catch (e) {
        expect(e).to.be.instanceOf(InvestorTypeNotAllowedError);
      }
    });
  });
});