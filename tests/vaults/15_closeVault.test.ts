import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
} from "../utils.new";

describe("Close Vault", () => {
  let authority: Keypair;
  let testManager: Keypair;
  let authorityVaults: VaultsSdk;
  let vaults: VaultsSdk;

  before(async () => {
    // Get keypairs
    authority = getAuthorityKeypair();
    testManager = getManagerKeypair();

    // Initialize SDKs
    const connection = createConnection();
    authorityVaults = new VaultsSdk(connection, authority);
    vaults = new VaultsSdk(connection, testManager);

    // Add test manager to whitelist
  });

  it("should successfully close an empty vault", async () => {
    const vault = await vaults.fetchVault(testManager.publicKey);
    expect(vault).to.not.be.undefined;
    expect(vault.currentDepositors).to.equal(0);

    // Close the vault
    const signature = await vaults.closeVault(testManager.publicKey, [testManager]);
    expect(signature).to.not.be.undefined;
	console.log(signature);

    // Verify vault is closed (should throw an error when trying to fetch)
    try {
      await vaults.fetchVault(testManager.publicKey);
      expect.fail("Vault should be closed and not fetchable");
    } catch (e) {
      // Expected - vault should not exist anymore
      expect(e.toString()).to.include("Account does not exist");
    }
  });

  // it("should fail to close vault with non-manager signer", async () => {
  //   const anotherManager = Keypair.generate();
  //   const anotherVaults = new VaultsSdk(createConnection(), anotherManager);

  //   // Add another manager to whitelist
  //   await authorityVaults.addManagerToWhitelist(anotherManager.publicKey, [authority]);
  //   // Try to close with wrong signer (should fail)
  //   try {
  //     await vaults.closeVault(anotherManager.publicKey, [testManager]);
  //     expect.fail("Should not be able to close vault with wrong signer");
  //   } catch (e) {
  //     expect(e.toString()).to.include("unauthorized");
  //   }

  //   // Clean up - close with correct signer
  //   await anotherVaults.closeVault(anotherManager.publicKey, [anotherManager]);
  // });

  // Note: Testing vault closure with active depositors would require:
  // 1. Adding users to user whitelist
  // 2. Funding user accounts with USDC
  // 3. Making deposits
  // 4. Then trying to close (should fail)
  // This is more complex and can be added later if needed
});

