import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import { USDC } from "@starke/sdk/whitelist";

import {
  createConnection,
  getAuthorityKeypair,
  getTesterKeypair,
} from "../utils.new";

describe("Close Vault", () => {
  let authority: Keypair;
  let testManager: Keypair;
  let authorityVaults: VaultsSdk;
  let testVaults: VaultsSdk;

  before(async () => {
    // Get keypairs
    authority = getAuthorityKeypair();
    testManager = Keypair.generate(); // Use a fresh manager for clean testing

    // Initialize SDKs
    const connection = createConnection();
    authorityVaults = new VaultsSdk(connection, authority);
    testVaults = new VaultsSdk(connection, testManager);

    // Add test manager to whitelist
    await authorityVaults.addManagerToWhitelist(testManager.publicKey, [authority]);
  });

  it("should successfully close an empty vault", async () => {
    // Create a test vault
    await testVaults.createVault(
      "Test Close Vault",
      "CLOSE",
      "https://example.com/close-vault.json",
      testManager.publicKey,
      USDC.mint,
      false, // not transferrable
      null, // no max AUM
      true, // allow retail
      true, // allow accredited
      true, // allow institutional
      true, // allow qualified
      0, // no individual min deposit
      0, // no institutional min deposit
      0, // no max depositors limit
      [testManager]
    );

    // Verify vault exists
    const vault = await testVaults.fetchVault(testManager.publicKey);
    expect(vault).to.not.be.undefined;
    expect(vault.currentDepositors).to.equal(0);

    // Close the vault
    const signature = await testVaults.closeVault(testManager.publicKey, [testManager]);
    expect(signature).to.not.be.undefined;

    // Verify vault is closed (should throw an error when trying to fetch)
    try {
      await testVaults.fetchVault(testManager.publicKey);
      expect.fail("Vault should be closed and not fetchable");
    } catch (e) {
      // Expected - vault should not exist anymore
      expect(e.toString()).to.include("Account does not exist");
    }
  });

  it("should fail to close vault with non-manager signer", async () => {
    const anotherManager = Keypair.generate();
    const anotherVaults = new VaultsSdk(createConnection(), anotherManager);

    // Add another manager to whitelist
    await authorityVaults.addManagerToWhitelist(anotherManager.publicKey, [authority]);

    // Create vault
    await anotherVaults.createVault(
      "Another Test Vault",
      "ANOTHER",
      "https://example.com/another.json",
      anotherManager.publicKey,
      USDC.mint,
      false,
      null,
      true, true, true, true,
      0, 0, 0,
      [anotherManager]
    );

    // Try to close with wrong signer (should fail)
    try {
      await testVaults.closeVault(anotherManager.publicKey, [testManager]);
      expect.fail("Should not be able to close vault with wrong signer");
    } catch (e) {
      expect(e.toString()).to.include("unauthorized");
    }

    // Clean up - close with correct signer
    await anotherVaults.closeVault(anotherManager.publicKey, [anotherManager]);
  });

  // Note: Testing vault closure with active depositors would require:
  // 1. Adding users to user whitelist
  // 2. Funding user accounts with USDC
  // 3. Making deposits
  // 4. Then trying to close (should fail)
  // This is more complex and can be added later if needed
});