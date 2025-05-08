import { Program, Wallet } from "@coral-xyz/anchor";
import * as anchor from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  ExtensionType,
  TOKEN_2022_PROGRAM_ID,
  createInitializeMint2Instruction,
  createInitializeMintInstruction,
  createInitializeTransferHookInstruction,
  createMint,
  getAssociatedTokenAddressSync,
  getMintLen,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { sendAndConfirmTransaction } from "@solana/web3.js";
import { expect } from "chai";
import { TransferHook } from "target/types/transfer_hook";
import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
  getTesterKeypair,
  requestAirdropIfNecessary,
} from "tests/utils.new";

import { VaultsSdk } from "@starke/sdk";
import { VaultConfigNotInitializedError } from "@starke/sdk/lib/errors";
import { getVaultPda, getVtokenMintPda } from "@starke/sdk/lib/pdas";
import { TransferHookSdk } from "@starke/sdk/transferHook";

describe("Transfer Hook", () => {
  const program = anchor.workspace.TransferHook as Program<TransferHook>;

  const tester = getTesterKeypair();

  const mint: Keypair = Keypair.generate();
  const decimals = 9;

  // Sender
  const sourceTokenAccount = getAssociatedTokenAddressSync(
    mint.publicKey,
    tester.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // Receiver
  const receiver = Keypair.generate();
  const receiverTokenAccount = getAssociatedTokenAddressSync(
    mint.publicKey,
    receiver.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // Connection
  const connection = createConnection();

  before(async () => {
    await requestAirdropIfNecessary(connection, tester.publicKey);
  });

  it("should create a mint account with transfer hook extension", async () => {
    const extensions = [ExtensionType.TransferHook];
    const mintLen = getMintLen(extensions);
    const lamports = await connection.getMinimumBalanceForRentExemption(
      mintLen
    );

    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: tester.publicKey,
        newAccountPubkey: mint.publicKey,
        space: mintLen,
        lamports: lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeTransferHookInstruction(
        mint.publicKey,
        tester.publicKey,
        program.programId,
        TOKEN_2022_PROGRAM_ID
      ),
      createInitializeMintInstruction(
        mint.publicKey,
        decimals,
        tester.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID
      )
    );

    const txSig = await sendAndConfirmTransaction(connection, tx, [
      tester,
      mint,
    ]);
  });

  it("should add a vault config", async () => {
    const [extraAccountMetaList] = PublicKey.findProgramAddressSync(
      [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
      program.programId
    );

    const addVaultConfigInstruction = await program.methods
      .addVaultConfig(true)
      .accounts({
        manager: tester.publicKey,
        mint: mint.publicKey,
      })
      .instruction();

    const tx = new Transaction().add(addVaultConfigInstruction);
    try {
      const txSig = await sendAndConfirmTransaction(connection, tx, [tester], {
        skipPreflight: true,
      });
    } catch (error) {
      console.error(error);
      throw error;
    }

    // Check if the extra account meta list was created
  });
});
