import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

import { VaultManager } from "../target/types/vault_manager";
import { confirmTransaction, requestAirdrop } from "./utils";

describe("VaultManager", () => {
  // configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const vaultManager = anchor.workspace
    .VaultManager as anchor.Program<VaultManager>;
  const depositToken = anchor.web3.Keypair.generate().publicKey;

  const vaultManagerSignerA = anchor.web3.Keypair.generate();
  const vaultManagerSignerB = anchor.web3.Keypair.generate();

  before(async () => {
    await requestAirdrop(vaultManagerSignerA.publicKey);
    await requestAirdrop(vaultManagerSignerB.publicKey);
  });

  it("vault is initialized by vault manager A", async () => {
    const tx = await vaultManager.methods
      .createVault(depositToken, "Vault A")
      .accounts({
        manager: vaultManagerSignerA.publicKey,
      })
      .signers([vaultManagerSignerA])
      .rpc();
    await confirmTransaction(tx);
  });

  it("vault is initialized again by vault manager A", async () => {
    expect(
      async () =>
        await vaultManager.methods
          .createVault(depositToken, "Vault A")
          .accounts({
            manager: vaultManagerSignerA.publicKey,
          })
          .signers([vaultManagerSignerA])
          .rpc()
    ).to.throw(Error, "Vault already exists");
  });

  it("vault is initialized by vault manager B", async () => {
    const tx = await vaultManager.methods
      .createVault(depositToken, "Vault B")
      .accounts({
        manager: vaultManagerSignerB.publicKey,
      })
      .signers([vaultManagerSignerB])
      .rpc();
    await confirmTransaction(tx);
  });
});
