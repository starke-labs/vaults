import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import {
  ManagerAlreadyInWhitelistError,
  ManagerNotInWhitelistError,
  SignatureVerificationFailedError,
} from "@starke/sdk/lib/errors";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
  getTesterKeypair,
} from "../utils.new";

describe("Manager Whitelist Tests", () => {
  let vaults: VaultsSdk;
  let tester: Keypair;
  let authority: Keypair;
  let manager: Keypair;
  let dummyManager: Keypair;

  before(async () => {
    // Get keypairs
    tester = getTesterKeypair();
    authority = getAuthorityKeypair();
    manager = getManagerKeypair();
    dummyManager = Keypair.generate();

    // Initialize SDK
    vaults = new VaultsSdk(createConnection(), tester);
  });

  it("should not add manager to whitelist without proper authority", async () => {
    // Check the length of the whitelist
    let whitelist = await vaults.fetchManagerWhitelist();
    const initialLength = whitelist.managers.length;

    // Test with no signer
    try {
      await vaults.addManagerToWhitelist(dummyManager.publicKey, []);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }

    // Verify manager was not added
    whitelist = await vaults.fetchManagerWhitelist();
    let finalLength = whitelist.managers.length;
    expect(finalLength).to.equal(initialLength);

    // Test with non-authority signer
    try {
      await vaults.addManagerToWhitelist(dummyManager.publicKey, [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }

    // Verify manager was still not added
    whitelist = await vaults.fetchManagerWhitelist();
    finalLength = whitelist.managers.length;
    expect(finalLength).to.equal(initialLength);
  });

  it("should successfully add manager to whitelist if not already in the whitelist", async () => {
    try {
      await vaults.addManagerToWhitelist(manager.publicKey, [authority]);
    } catch (e) {
      if (!(e instanceof ManagerAlreadyInWhitelistError)) {
        throw e;
      }
    }

    // Verify manager was added
    const whitelist = await vaults.fetchManagerWhitelist();
    const managerInWhitelist = whitelist.managers.find(
      (m) => m.toBase58() === manager.publicKey.toBase58()
    );
    expect(managerInWhitelist).to.not.be.undefined;
    expect(managerInWhitelist?.toBase58()).to.equal(
      manager.publicKey.toBase58()
    );
  });

  it("should not add same manager twice", async () => {
    try {
      await vaults.addManagerToWhitelist(manager.publicKey, [authority]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(ManagerAlreadyInWhitelistError);
    }
  });

  it("should not remove manager from whitelist without proper authority", async () => {
    // Try to remove manager from whitelist without any signers
    try {
      await vaults.removeManagerFromWhitelist(manager.publicKey, []);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }

    // Try to remove manager from whitelist with non-authority signer
    try {
      await vaults.removeManagerFromWhitelist(manager.publicKey, [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(SignatureVerificationFailedError);
    }
  });

  it("should successfully remove manager from whitelist", async () => {
    // Add manager to whitelist first
    await vaults.addManagerToWhitelist(dummyManager.publicKey, [authority]);

    // Verify manager was added
    let whitelist = await vaults.fetchManagerWhitelist();
    let managerInWhitelist = whitelist.managers.find(
      (m) => m.toBase58() === dummyManager.publicKey.toBase58()
    );
    expect(managerInWhitelist).to.not.be.undefined;
    expect(managerInWhitelist?.toBase58()).to.equal(
      dummyManager.publicKey.toBase58()
    );

    // Remove manager from whitelist
    await vaults.removeManagerFromWhitelist(dummyManager.publicKey, [
      authority,
    ]);

    // Verify manager was removed
    whitelist = await vaults.fetchManagerWhitelist();
    managerInWhitelist = whitelist.managers.find(
      (m) => m.toBase58() === dummyManager.publicKey.toBase58()
    );
    expect(managerInWhitelist).to.be.undefined;
  });

  it("should not remove manager from whitelist if manager is not in the whitelist", async () => {
    // Try to remove manager from whitelist
    try {
      await vaults.removeManagerFromWhitelist(dummyManager.publicKey, [
        authority,
      ]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e).to.be.instanceOf(ManagerNotInWhitelistError);
    }
  });
});
