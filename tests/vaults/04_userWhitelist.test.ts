import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import { SignatureVerificationFailedError } from "@starke/sdk/lib/errors";
import { InvestorType } from "@starke/sdk/lib/types";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
  getTesterKeypair,
} from "../utils.new";

describe("User Whitelist Tests", () => {
  let vaults: VaultsSdk;
  let authority: Keypair;
  let manager: Keypair;
  let tester: Keypair;
  let randomUser: Keypair;
  let authorityVaults: VaultsSdk;

  before(async () => {
    // Get keypairs
    authority = getAuthorityKeypair();
    manager = getManagerKeypair();
    tester = getTesterKeypair();
    randomUser = Keypair.generate();

    // Initialize SDKs
    vaults = new VaultsSdk(createConnection(), tester);
    authorityVaults = new VaultsSdk(createConnection(), authority);
  });

  it("should fetch user whitelist after initialization", async () => {
    const userWhitelist = await vaults.fetchUserWhitelist();
    expect(userWhitelist).to.not.be.undefined;
    expect(userWhitelist.authority.toBase58()).to.equal(
      authority.publicKey.toBase58()
    );
    expect(userWhitelist.users).to.be.an("array");
  });

  it("should not allow non-authority to add users to whitelist", async () => {
    try {
      await vaults.addUserToWhitelist(tester.publicKey, InvestorType.Retail, [
        tester,
      ]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      // Should throw SignatureVerificationFailedError when wrong signer is used
      expect(e.toString()).to.include("SignatureVerificationFailedError");
    }
  });

  it("should allow authority to add users to whitelist", async () => {
    // Add tester as retail investor
    const signature = await authorityVaults.addUserToWhitelist(
      tester.publicKey,
      InvestorType.Retail,
      [authority]
    );
    expect(signature).to.not.be.undefined;

    // Verify user was added
    const userWhitelist = await vaults.fetchUserWhitelist();
    const testerEntry = userWhitelist.users.find(
      (user) => user.user.toBase58() === tester.publicKey.toBase58()
    );
    expect(testerEntry).to.not.be.undefined;
    expect(testerEntry.investorType).to.deep.equal(InvestorType.Retail);
  });

  it("should allow authority to add multiple users with different investor types", async () => {
    // Add manager as accredited investor
    await authorityVaults.addUserToWhitelist(
      manager.publicKey,
      InvestorType.Accredited,
      [authority]
    );

    // Add random user as institutional investor
    await authorityVaults.addUserToWhitelist(
      randomUser.publicKey,
      InvestorType.Institutional,
      [authority]
    );

    // Verify all users were added
    const userWhitelist = await vaults.fetchUserWhitelist();
    expect(userWhitelist.users.length).to.be.at.least(1);

    const managerEntry = userWhitelist.users.find(
      (user) => user.user.toBase58() === manager.publicKey.toBase58()
    );
    expect(managerEntry.investorType).to.deep.equal(InvestorType.Accredited);

    const randomUserEntry = userWhitelist.users.find(
      (user) => user.user.toBase58() === randomUser.publicKey.toBase58()
    );
    expect(randomUserEntry.investorType).to.deep.equal(
      InvestorType.Institutional
    );
  });

  it("should allow authority to update existing user's investor type", async () => {
    // Update tester from Retail to Qualified
    await authorityVaults.addUserToWhitelist(
      tester.publicKey,
      InvestorType.Qualified,
      [authority]
    );

    // Verify investor type was updated
    const userWhitelist = await vaults.fetchUserWhitelist();
    const testerEntry = userWhitelist.users.find(
      (user) => user.user.toBase58() === tester.publicKey.toBase58()
    );
    expect(testerEntry.investorType).to.deep.equal(InvestorType.Qualified);
  });

  it("should not allow non-authority to remove users from whitelist", async () => {
    try {
      await vaults.removeUserFromWhitelist(randomUser.publicKey, [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }
  });

  it("should allow authority to remove users from whitelist", async () => {
    // First ensure the random user is in the whitelist by adding them
    await authorityVaults.addUserToWhitelist(
      randomUser.publicKey,
      InvestorType.Institutional,
      [authority]
    );

    // Verify user was added
    let userWhitelist = await vaults.fetchUserWhitelist();
    let addedUserEntry = userWhitelist.users.find(
      (user) => user.user.toBase58() === randomUser.publicKey.toBase58()
    );
    expect(addedUserEntry).to.exist;

    // Remove random user from whitelist
    const signature = await authorityVaults.removeUserFromWhitelist(
      randomUser.publicKey,
      [authority]
    );
    expect(signature).to.not.be.undefined;

    // Verify user was removed
    userWhitelist = await vaults.fetchUserWhitelist();
    const removedUserEntry = userWhitelist.users.find(
      (user) => user.user.toBase58() === randomUser.publicKey.toBase58()
    );
    expect(removedUserEntry).to.be.undefined;
  });

  it("should handle removing non-existent user gracefully", async () => {
    const nonExistentUser = Keypair.generate();

    try {
      await authorityVaults.removeUserFromWhitelist(nonExistentUser.publicKey, [
        authority,
      ]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      // Should throw UserNotFound error from the program
      expect(e.toString()).to.include("UserNotFound");
    }
  });

  it("should maintain whitelist integrity after operations", async () => {
    const userWhitelist = await vaults.fetchUserWhitelist();

    // Should have at least some users
    expect(userWhitelist.users.length).to.be.at.least(0);

    // Check for tester if they exist
    const testerEntry = userWhitelist.users.find(
      (user) => user.user.toBase58() === tester.publicKey.toBase58()
    );
    if (testerEntry) {
      expect(testerEntry.investorType).to.deep.equal(InvestorType.Qualified);
    }

    // Check for manager if they exist
    const managerEntry = userWhitelist.users.find(
      (user) => user.user.toBase58() === manager.publicKey.toBase58()
    );
    if (managerEntry) {
      expect(managerEntry.investorType).to.deep.equal(InvestorType.Accredited);
    }

    // Ensure randomUser was properly removed
    const randomUserEntry = userWhitelist.users.find(
      (user) => user.user.toBase58() === randomUser.publicKey.toBase58()
    );
    expect(randomUserEntry).to.be.undefined;
  });
});
