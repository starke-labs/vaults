import { BN } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import { VaultsSDK } from "@starke/sdk";
import {
  InsufficientBalanceError,
  SignatureVerificationFailedError,
} from "@starke/sdk/lib/errors";
import { Vault } from "@starke/sdk/lib/types";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
  getTesterKeypair,
} from "../utils.new";

describe("Deposit", () => {
  let depositor: Keypair;
  let vaults: VaultsSDK;
  let manager: PublicKey;
  let vault: Vault;
  let authority: Keypair;
  let randomKeypair: Keypair;
  let randomVaults: VaultsSDK;

  before(async () => {
    depositor = getTesterKeypair();
    vaults = new VaultsSDK(createConnection(), depositor);
    manager = getManagerKeypair().publicKey;
    vault = await vaults.fetchVault(manager);
    authority = getAuthorityKeypair();
    randomKeypair = Keypair.generate();
    randomVaults = new VaultsSDK(createConnection(), randomKeypair);
  });

  it("should not be able to deposit without the depositor's or the authority's signature", async () => {
    try {
      await randomVaults.deposit(
        new BN(1 * 10 ** 6),
        depositor.publicKey,
        manager,
        [authority]
      );
      expect.fail(
        "Should not be able to deposit without the depositor's signature"
      );
    } catch (error) {
      expect(error).instanceOf(SignatureVerificationFailedError);
    }

    try {
      await vaults.deposit(new BN(1 * 10 ** 6), depositor.publicKey, manager);
      expect.fail(
        "Should not be able to deposit without the authority's signature"
      );
    } catch (error) {
      expect(error).instanceOf(SignatureVerificationFailedError);
    }
  });

  it("should not be able to deposit more than the depositor's balance", async () => {
    try {
      await vaults.deposit(
        new BN(100 * 10 ** 6),
        depositor.publicKey,
        manager,
        [authority]
      );
      expect.fail(
        "Should not be able to deposit more than the depositor's balance"
      );
    } catch (error) {
      expect(error).instanceOf(InsufficientBalanceError);
    }
  });

  it("should be able to deposit", async () => {
    await vaults.deposit(new BN(1 * 10 ** 6), depositor.publicKey, manager, [
      authority,
    ]);
  });
});
