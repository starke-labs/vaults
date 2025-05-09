import { Program } from "@coral-xyz/anchor";
import * as anchor from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  ExtensionType,
  TOKEN_2022_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createInitializeTransferHookInstruction,
  createMintToInstruction,
  createTransferCheckedWithTransferHookInstruction,
  getAssociatedTokenAddressSync,
  getExtraAccountMetaAddress,
  getExtraAccountMetas,
  getMint,
  getMintLen,
  getTransferHook,
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
  getManagerKeypair,
  getTesterKeypair, // requestAirdropIfNecessary,
} from "tests/utils.new";

describe("Transfer Hook", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TransferHook as Program<TransferHook>;

  const tester = getTesterKeypair();
  const receiver = getManagerKeypair();
  const mint1: Keypair = Keypair.generate();
  const mint2: Keypair = Keypair.generate();
  const decimals = 9;

  // Sender
  const sourceTokenAccount1 = getAssociatedTokenAddressSync(
    mint1.publicKey,
    tester.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  const sourceTokenAccount2 = getAssociatedTokenAddressSync(
    mint2.publicKey,
    tester.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // Receiver
  // const receiver = Keypair.generate();
  const receiverTokenAccount1 = getAssociatedTokenAddressSync(
    mint1.publicKey,
    receiver.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  const receiverTokenAccount2 = getAssociatedTokenAddressSync(
    mint2.publicKey,
    receiver.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // Connection
  const connection = createConnection();

  before(async () => {
    // await requestAirdropIfNecessary(connection, tester.publicKey);
  });

  it("should create mint1 account with transfer hook extension", async () => {
    const extensions = [ExtensionType.TransferHook];
    const mintLen = getMintLen(extensions);
    const lamports = await connection.getMinimumBalanceForRentExemption(
      mintLen
    );

    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: tester.publicKey,
        newAccountPubkey: mint1.publicKey,
        space: mintLen,
        lamports: lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeTransferHookInstruction(
        mint1.publicKey,
        tester.publicKey,
        program.programId,
        TOKEN_2022_PROGRAM_ID
      ),
      createInitializeMintInstruction(
        mint1.publicKey,
        decimals,
        tester.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID
      )
    );

    const txSig = await sendAndConfirmTransaction(connection, tx, [
      tester,
      mint1,
    ]);
    console.log("txSig", txSig);
  });

  it("should create mint2 account with transfer hook extension", async () => {
    const extensions = [ExtensionType.TransferHook];
    const mintLen = getMintLen(extensions);
    const lamports = await connection.getMinimumBalanceForRentExemption(
      mintLen
    );

    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: tester.publicKey,
        newAccountPubkey: mint2.publicKey,
        space: mintLen,
        lamports: lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeTransferHookInstruction(
        mint2.publicKey,
        tester.publicKey,
        program.programId,
        TOKEN_2022_PROGRAM_ID
      ),
      createInitializeMintInstruction(
        mint2.publicKey,
        decimals,
        tester.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID
      )
    );

    const txSig = await sendAndConfirmTransaction(connection, tx, [
      tester,
      mint2,
    ]);
    console.log("txSig", txSig);
  });

  it("should add a vault config for mint1 with true", async () => {
    // // Check if the extra account metas was created
    // const [extraAccountMetas] = PublicKey.findProgramAddressSync(
    //   [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
    //   program.programId
    // );

    // // Check if the extra account metas account is empty
    // let extraAccountMetasAccount = await connection.getAccountInfo(
    //   extraAccountMetas
    // );
    // expect(extraAccountMetasAccount).to.equal(null);

    const addVaultConfigInstruction = await program.methods
      .initializeExtraAccountMetas(true)
      .accounts({
        manager: tester.publicKey,
        mint: mint1.publicKey,
      })
      .instruction();

    const tx = new Transaction().add(addVaultConfigInstruction);
    try {
      const txSig = await sendAndConfirmTransaction(connection, tx, [tester], {
        skipPreflight: false,
      });
      console.log("txSig", txSig);
    } catch (error) {
      console.error(error);
      throw error;
    }

    // // Get the vault config account
    // const [vaultConfig] = PublicKey.findProgramAddressSync(
    //   [Buffer.from("STARKE_VAULT_CONFIG"), mint.publicKey.toBuffer()],
    //   program.programId
    // );

    // // Check if the extra account meta list was created
    // const vaultConfigAccount = await program.account.vaultConfig.fetch(
    //   vaultConfig
    // );
    // expect(vaultConfigAccount.vtokenIsTransferrable).to.equal(true);
    // expect(vaultConfigAccount.vtokenMint.equals(mint.publicKey)).to.equal(true);

    // // Check if the extra account metas account is not empty
    // extraAccountMetasAccount = await connection.getAccountInfo(
    //   extraAccountMetas
    // );
    // expect(extraAccountMetasAccount).to.not.equal(null);
  });

  it("should add a vault config for mint2 with false", async () => {
    const addVaultConfigInstruction = await program.methods
      .initializeExtraAccountMetas(false)
      .accounts({
        manager: tester.publicKey,
        mint: mint2.publicKey,
      })
      .instruction();

    const tx = new Transaction().add(addVaultConfigInstruction);
    try {
      const txSig = await sendAndConfirmTransaction(connection, tx, [tester], {
        skipPreflight: false,
      });
      console.log("txSig", txSig);
    } catch (error) {
      console.error(error);
      throw error;
    }
  });

  // it("should set the vtoken is transferrable to false", async () => {
  //   // // Get the vault config account
  //   // let [vaultConfig] = PublicKey.findProgramAddressSync(
  //   //   [Buffer.from("STARKE_VAULT_CONFIG"), mint.publicKey.toBuffer()],
  //   //   // [Buffer.from("STARKE_VAULT_CONFIG")],
  //   //   program.programId
  //   // );

  //   // // Get the vault config account
  //   // let vaultConfigAccount = await program.account.vaultConfig.fetch(
  //   //   vaultConfig
  //   // );
  //   // expect(vaultConfigAccount.vtokenIsTransferrable).to.equal(true);

  //   const setVtokenIsTransferrableInstruction = await program.methods
  //     .setVtokenIsTransferrable(false)
  //     .accounts({
  //       manager: tester.publicKey,
  //       mint: mint1.publicKey,
  //     })
  //     .instruction();

  //   const tx = new Transaction().add(setVtokenIsTransferrableInstruction);
  //   const txSig = await sendAndConfirmTransaction(connection, tx, [tester]);
  //   console.log("txSig", txSig);

  //   // // Get the vault config account
  //   // [vaultConfig] = PublicKey.findProgramAddressSync(
  //   //   [Buffer.from("STARKE_VAULT_CONFIG"), mint.publicKey.toBuffer()],
  //   //   // [Buffer.from("STARKE_VAULT_CONFIG")],
  //   //   program.programId
  //   // );

  //   // vaultConfigAccount = await program.account.vaultConfig.fetch(vaultConfig);
  //   // expect(vaultConfigAccount.vtokenIsTransferrable).to.equal(false);
  // });

  it("should create associated token accounts and mint tokens for mint1", async () => {
    // 100 tokens
    const amount = 100 * 10 ** decimals;

    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        tester.publicKey,
        sourceTokenAccount1,
        tester.publicKey,
        mint1.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createAssociatedTokenAccountInstruction(
        tester.publicKey,
        receiverTokenAccount1,
        receiver.publicKey,
        mint1.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createMintToInstruction(
        mint1.publicKey,
        sourceTokenAccount1,
        tester.publicKey,
        amount,
        [],
        TOKEN_2022_PROGRAM_ID
      )
    );
    const txSig = await sendAndConfirmTransaction(connection, transaction, [
      tester,
    ]);
    console.log("txSig", txSig);

    // // Check if the source token account has the correct amount
    // const sourceTokenAccountBalance = await connection.getTokenAccountBalance(
    //   sourceTokenAccount
    // );
    // expect(sourceTokenAccountBalance.value.amount).to.equal(amount.toString());
  });

  it("should create associated token accounts and mint tokens for mint2", async () => {
    // 100 tokens
    const amount = 100 * 10 ** decimals;

    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        tester.publicKey,
        sourceTokenAccount2,
        tester.publicKey,
        mint2.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createAssociatedTokenAccountInstruction(
        tester.publicKey,
        receiverTokenAccount2,
        receiver.publicKey,
        mint2.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createMintToInstruction(
        mint2.publicKey,
        sourceTokenAccount2,
        tester.publicKey,
        amount,
        [],
        TOKEN_2022_PROGRAM_ID
      )
    );
    const txSig = await sendAndConfirmTransaction(connection, transaction, [
      tester,
    ]);
    console.log("txSig", txSig);
  });

  it("should transfer tokens from the source token account to the receiver token account for mint1", async () => {
    const amount = 1 * 10 ** decimals;

    const transferCheckedInstruction =
      await createTransferCheckedWithTransferHookInstruction(
        connection,
        sourceTokenAccount1,
        mint1.publicKey,
        receiverTokenAccount1,
        tester.publicKey,
        BigInt(amount),
        decimals,
        [tester.publicKey],
        "confirmed",
        TOKEN_2022_PROGRAM_ID
      );

    const tx = new Transaction().add(transferCheckedInstruction);
    const txSig = await sendAndConfirmTransaction(connection, tx, [tester], {
      skipPreflight: false,
    });
    console.log("txSig", txSig);
  });

  it("should not transfer mint2 tokens from the source token account to the receiver token account", async () => {
    const amount = 1 * 10 ** decimals;

    // const mintInfo = await getMint(
    //   connection,
    //   mint.publicKey,
    //   "confirmed",
    //   TOKEN_2022_PROGRAM_ID
    // );
    // const transferHook = getTransferHook(mintInfo);
    // expect(transferHook.programId.equals(program.programId)).to.equal(true);

    const transferCheckedInstruction =
      await createTransferCheckedWithTransferHookInstruction(
        connection,
        sourceTokenAccount2,
        mint2.publicKey,
        receiverTokenAccount2,
        tester.publicKey,
        BigInt(amount),
        decimals,
        [tester.publicKey],
        "confirmed",
        TOKEN_2022_PROGRAM_ID
      );

    const tx = new Transaction().add(transferCheckedInstruction);

    try {
      const txSig = await sendAndConfirmTransaction(connection, tx, [tester], {
        skipPreflight: false,
      });
      expect.fail("Transaction should have failed");
    } catch (error) {
      console.log("error", error.toString());
      expect(error.toString()).to.include("TokenNonTransferrable");
    }
  });

  it("should set the vtoken is transferrable to false for mint1", async () => {
    const setVtokenIsTransferrableInstruction = await program.methods
      .setVtokenIsTransferrable(false)
      .accounts({
        manager: tester.publicKey,
        mint: mint1.publicKey,
      })
      .instruction();

    const tx = new Transaction().add(setVtokenIsTransferrableInstruction);
    const txSig = await sendAndConfirmTransaction(connection, tx, [tester]);
    console.log("txSig", txSig);
  });

  it("should not transfer tokens from the source token account to the receiver token account for mint1", async () => {
    const amount = 1 * 10 ** decimals;

    const transferCheckedInstruction =
      await createTransferCheckedWithTransferHookInstruction(
        connection,
        sourceTokenAccount1,
        mint1.publicKey,
        receiverTokenAccount1,
        tester.publicKey,
        BigInt(amount),
        decimals,
        [tester.publicKey],
        "confirmed",
        TOKEN_2022_PROGRAM_ID
      );

    const tx = new Transaction().add(transferCheckedInstruction);
    try {
      const txSig = await sendAndConfirmTransaction(connection, tx, [tester], {
        skipPreflight: false,
      });
      expect.fail("Transaction should have failed");
    } catch (error) {
      console.log("error", error.toString());
      expect(error.toString()).to.include("TokenNonTransferrable");
    }
  });

  it("should set the vtoken is transferrable to true for mint2", async () => {
    const setVtokenIsTransferrableInstruction = await program.methods
      .setVtokenIsTransferrable(true)
      .accounts({
        manager: tester.publicKey,
        mint: mint2.publicKey,
      })
      .instruction();

    const tx = new Transaction().add(setVtokenIsTransferrableInstruction);
    const txSig = await sendAndConfirmTransaction(connection, tx, [tester]);
    console.log("txSig", txSig);
  });

  it("should transfer tokens from the source token account to the receiver token account for mint2", async () => {
    const amount = 1 * 10 ** decimals;

    const transferCheckedInstruction =
      await createTransferCheckedWithTransferHookInstruction(
        connection,
        sourceTokenAccount2,
        mint2.publicKey,
        receiverTokenAccount2,
        tester.publicKey,
        BigInt(amount),
        decimals,
        [tester.publicKey],
        "confirmed",
        TOKEN_2022_PROGRAM_ID
      );

    const tx = new Transaction().add(transferCheckedInstruction);
    const txSig = await sendAndConfirmTransaction(connection, tx, [tester], {
      skipPreflight: false,
    });
    console.log("txSig", txSig);
  });
});
