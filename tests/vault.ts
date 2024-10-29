import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

import { Vault } from "../target/types/vault";
import { confirmTransaction, requestAirdrop } from "./utils";

describe("vault", () => {
  // configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const vaultProgram = anchor.workspace.Vault as Program<Vault>;
  const depositToken = anchor.web3.Keypair.generate().publicKey;

  const vault = anchor.web3.Keypair.generate();
  const vaultOwner = anchor.web3.Keypair.generate();

  before(async () => {
    await requestAirdrop(vaultOwner.publicKey);
  });

  it("is initialized", async () => {
    const tx = await vaultProgram.methods
      .initialize(depositToken)
      .accounts({
        vault: vault.publicKey,
        vaultOwner: vaultOwner.publicKey,
      })
      .signers([vault, vaultOwner])
      .rpc();
    await confirmTransaction(tx);
  });
});
