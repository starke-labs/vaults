import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

import { VaultManager } from "../target/types/vault_manager";
import { confirmTransaction, requestAirdrop } from "./utils";

describe("VaultManager", () => {
  // configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const vaultManager = anchor.workspace.VaultManager as Program<VaultManager>;
  const depositToken = anchor.web3.Keypair.generate().publicKey;

  const vaultManagerSigner = anchor.web3.Keypair.generate();

  before(async () => {
    await requestAirdrop(vaultManagerSigner.publicKey);
  });

  it("is initialized", async () => {
    const tx = await vaultManager.methods
      .createVault(depositToken, "Vault 1")
      .accounts({
        manager: vaultManagerSigner.publicKey,
      })
      .signers([vaultManagerSigner])
      .rpc();
    await confirmTransaction(tx);
  });
});
