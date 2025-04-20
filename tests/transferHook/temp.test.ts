import { Keypair } from "@solana/web3.js";
import { expect } from "chai";
import {
  createConnection,
  getManagerKeypair,
  getTesterKeypair,
} from "tests/utils.new";

import {
  getVaultConfigPda,
  getVaultPda,
  getVtokenMintPda,
} from "@starke/sdk/lib/pdas";
import { TransferHookSdk } from "@starke/sdk/transferHook";

describe("Transfer Hook", () => {
  let tester: Keypair;
  let transferHook: TransferHookSdk;

  let manager: Keypair;

  before(async () => {
    tester = getTesterKeypair();
    manager = getManagerKeypair();
    transferHook = new TransferHookSdk(createConnection(), tester);
  });

  it("should be able to fetch the vault config", async () => {
    const [vault] = getVaultPda(manager.publicKey);
    const [vtokenMint] = getVtokenMintPda(vault);
    const [vaultConfigPda] = getVaultConfigPda(vtokenMint);

    const vaultConfig = await transferHook.getVaultConfig(vaultConfigPda);
    expect(vaultConfig).to.not.exist;
  });
});
