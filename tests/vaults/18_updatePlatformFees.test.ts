import { Connection, Keypair } from "@solana/web3.js";
import { expect } from "chai";
import { VaultsSdk } from "@starke/sdk";
import {
  SignatureVerificationFailedError,
} from "@starke/sdk/lib/errors";
import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
} from "../utils.new";

describe("Update Platform Fees", () => {
  let connection: Connection;
  let authority: Keypair;
  let manager: Keypair;
  let authorityVaults: VaultsSdk;
  let managerVaults: VaultsSdk;

  before(async () => {
    connection = createConnection();
    authority = getAuthorityKeypair();
    manager = getManagerKeypair();
    authorityVaults = new VaultsSdk(connection, authority);
    managerVaults = new VaultsSdk(connection, manager);
  });

  it("Should allow STARKE_AUTHORITY to set platform fees for a vault", async () => {
    // Update platform fees to 100 bps (1%)
    await authorityVaults.updatePlatformFees(
      manager.publicKey,
      100,
      [authority]
    );

    // Verify the rate was set correctly
    const vault = await authorityVaults.fetchVault(manager.publicKey);
    expect(vault.platformFeesRate).to.equal(100);
  });

  it("Should allow updating platform fees to 0 (disable fees)", async () => {
    // First set to 100 bps
    await authorityVaults.updatePlatformFees(
      manager.publicKey,
      100,
      [authority]
    );

    let vault = await authorityVaults.fetchVault(manager.publicKey);
    expect(vault.platformFeesRate).to.equal(100);

    // Then update to 0 bps
    await authorityVaults.updatePlatformFees(
      manager.publicKey,
      0,
      [authority]
    );

    vault = await authorityVaults.fetchVault(manager.publicKey);
    expect(vault.platformFeesRate).to.equal(0);
  });

  it("Should allow updating platform fees multiple times", async () => {
    // Set to 100 bps
    await authorityVaults.updatePlatformFees(
      manager.publicKey,
      100,
      [authority]
    );

    let vault = await authorityVaults.fetchVault(manager.publicKey);
    expect(vault.platformFeesRate).to.equal(100);

    // Update to 200 bps
    await authorityVaults.updatePlatformFees(
      manager.publicKey,
      200,
      [authority]
    );

    vault = await authorityVaults.fetchVault(manager.publicKey);
    expect(vault.platformFeesRate).to.equal(200);

    // Update to 150 bps
    await authorityVaults.updatePlatformFees(
      manager.publicKey,
      150,
      [authority]
    );

    vault = await authorityVaults.fetchVault(manager.publicKey);
    expect(vault.platformFeesRate).to.equal(150);
  });

  it("Should reject non-authority attempts to set platform fees", async () => {
    try {
      // Try to call with manager account instead of authority
      await managerVaults.updatePlatformFees(
        manager.publicKey,
        100,
        [manager]
      );
      expect.fail("Should have thrown an error");
    } catch (e) {
      // Expect Unauthorized error
      expect(e.toString()).to.include("Unauthorized");
    }
  });

  it("Should reject invalid fee rates greater than 10000 bps (100%)", async () => {
    try {
      await authorityVaults.updatePlatformFees(
        manager.publicKey,
        10001, // More than 100%
        [authority]
      );
      expect.fail("Should have thrown an error");
    } catch (e) {
      // Expect InvalidFee error
      expect(e.toString()).to.include("InvalidFee");
    }
  });

  it("Should accept maximum fee rate of 10000 bps (100%)", async () => {
    // Set to 10000 bps (100%)
    await authorityVaults.updatePlatformFees(
      manager.publicKey,
      10000,
      [authority]
    );

    const vault = await authorityVaults.fetchVault(manager.publicKey);
    expect(vault.platformFeesRate).to.equal(10000);

    // Reset to a reasonable value
    await authorityVaults.updatePlatformFees(
      manager.publicKey,
      100,
      [authority]
    );
  });
});
