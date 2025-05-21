import { BN } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  createTransferCheckedWithTransferHookInstruction,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { expect } from "chai";
import {
  VTOKEN_DECIMALS,
  getAuthorityKeypair,
  getManager2Keypair,
  getManagerKeypair,
  getTesterKeypair,
} from "tests/utils.new";
import { createConnection } from "tests/utils.new/provider";

import { VaultsSdk } from "@starke/sdk";
import { getVaultPda, getVtokenMintPda } from "@starke/sdk/lib/pdas";

describe("Vtoken Transferrability", () => {
  const connection = createConnection();
  const authority = getAuthorityKeypair();
  const manager = getManagerKeypair();
  const tester = getTesterKeypair();
  const tester2 = getManager2Keypair();

  const vaults = new VaultsSdk(connection, tester);

  const vaultPda = getVaultPda(manager.publicKey)[0];
  const vtokenMint = getVtokenMintPda(vaultPda)[0];

  const testerATA = getAssociatedTokenAddressSync(
    vtokenMint,
    tester.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID
  );
  let tester2ATA = getAssociatedTokenAddressSync(
    vtokenMint,
    tester2.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID
  );

  it("should be able to deposit and receive some vtokens", async () => {
    const [vaultPda] = getVaultPda(manager.publicKey);

    await vaults.deposit(
      new BN(1 * 10 ** 6),
      tester.publicKey,
      manager.publicKey,
      [authority]
    );

    const vtokenBalance = await vaults.getVtokenBalance(
      vaultPda,
      tester.publicKey
    );
    expect(vtokenBalance.gt(new BN(0))).to.be.true;
  });

  it("should not be able to transfer vtokens", async () => {
    const vtokenBalance = await vaults.getVtokenBalance(
      vaultPda,
      tester.publicKey
    );

    const instruction = await createTransferCheckedWithTransferHookInstruction(
      connection,
      testerATA,
      vtokenMint,
      tester2ATA,
      tester.publicKey,
      BigInt(vtokenBalance.toString()),
      VTOKEN_DECIMALS,
      [tester.publicKey],
      "confirmed",
      TOKEN_2022_PROGRAM_ID
    );
    const tx = new Transaction().add(instruction);
    const txSig = await sendAndConfirmTransaction(connection, tx, [tester]);
  });
});
