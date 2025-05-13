import { BN } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSdk } from "@starke/sdk";
import {
  InsufficientBalanceError,
  SignatureVerificationFailedError,
} from "@starke/sdk/lib/errors";
import { getVaultPda } from "@starke/sdk/lib/pdas";
import { Vault } from "@starke/sdk/lib/types";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
  getTesterKeypair,
} from "../utils.new";

describe("Withdraw", () => {
  let connection: Connection;
  let withdrawer: Keypair;
  let vaults: VaultsSdk;
  let manager: PublicKey;
  let vaultPda: PublicKey;
  let vault: Vault;
  let authority: Keypair;
  let randomKeypair: Keypair;
  let randomVaults: VaultsSdk;

  before(async () => {
    connection = createConnection();
    withdrawer = getTesterKeypair();
    vaults = new VaultsSdk(connection, withdrawer);
    manager = getManagerKeypair().publicKey;
    [vaultPda] = getVaultPda(manager);
    vault = await vaults.fetchVault(manager);
    authority = getAuthorityKeypair();
    randomKeypair = Keypair.generate();
    randomVaults = new VaultsSdk(connection, randomKeypair);
  });

  it("should not be able to withdraw without the withdrawer's or the authority's signature", async () => {
    try {
      await randomVaults.withdraw(
        new BN(1 * 10 ** 6),
        withdrawer.publicKey,
        manager,
        [authority]
      );
      expect.fail(
        "Should not be able to withdraw without the withdrawer's signature"
      );
    } catch (error) {
      expect(error).instanceOf(SignatureVerificationFailedError);
    }

    try {
      await vaults.withdraw(new BN(1 * 10 ** 6), withdrawer.publicKey, manager);
      expect.fail(
        "Should not be able to withdraw without the authority's signature"
      );
    } catch (error) {
      expect(error).instanceOf(SignatureVerificationFailedError);
    }
  });

  it("should not be able to withdraw more than the available vtoken balance", async () => {
    const vtokenBalance = await vaults.getVtokenBalance(
      vaultPda,
      withdrawer.publicKey
    );
    try {
      await vaults.withdraw(
        vtokenBalance.add(new BN(1)),
        withdrawer.publicKey,
        manager,
        [authority]
      );
      expect.fail(
        "Should not be able to withdraw more than the available balance"
      );
    } catch (error) {
      expect(error).instanceOf(InsufficientBalanceError);
    }
  });

  it("should be able to withdraw", async () => {
    // NOTE: This test could be flaky because it depends on the vtoken balance which might take some time to update
    const initialVtokenBalance = await vaults.getVtokenBalance(
      vaultPda,
      withdrawer.publicKey
    );
    console.log("initialVtokenBalance", initialVtokenBalance.toString());

    if (initialVtokenBalance.gt(new BN(10 ** -3 * 10 ** 9))) {
      await vaults.withdraw(
        initialVtokenBalance,
        withdrawer.publicKey,
        manager,
        [authority]
      );

      // Check final vtoken balance
      const finalVtokenBalance = await vaults.getVtokenBalance(
        vaultPda,
        withdrawer.publicKey
      );

      // Verify balance reduction matches withdraw amount
      expect(finalVtokenBalance.toString()).to.equal(new BN(0).toString());
    } else {
      expect.fail(
        "Insufficient vtoken balance to withdraw, run the deposit test first"
      );
    }
  });

  // TODO: Test with multiple tokens
});
