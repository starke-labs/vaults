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
  getTesterKeypair,
  requestAirdropIfNecessary,
} from "tests/utils.new";

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
    // Check if the extra account metas was created
    const [extraAccountMetas] = PublicKey.findProgramAddressSync(
      [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
      program.programId
    );

    // Check if the extra account metas account is empty
    let extraAccountMetasAccount = await connection.getAccountInfo(
      extraAccountMetas
    );
    expect(extraAccountMetasAccount).to.equal(null);

    const addVaultConfigInstruction = await program.methods
      .initializeExtraAccountMetas(true)
      .accounts({
        manager: tester.publicKey,
        mint: mint.publicKey,
      })
      .instruction();

    const tx = new Transaction().add(addVaultConfigInstruction);
    try {
      const txSig = await sendAndConfirmTransaction(connection, tx, [tester], {
        skipPreflight: false,
      });
    } catch (error) {
      console.error(error);
      throw error;
    }

    // Get the vault config account
    const [vaultConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("STARKE_VAULT_CONFIG"), mint.publicKey.toBuffer()],
      // [Buffer.from("STARKE_VAULT_CONFIG")],
      program.programId
    );

    // Check if the extra account meta list was created
    const vaultConfigAccount = await program.account.vaultConfig.fetch(
      vaultConfig
    );
    expect(vaultConfigAccount.vtokenIsTransferrable).to.equal(true);
    expect(vaultConfigAccount.vtokenMint.equals(mint.publicKey)).to.equal(true);

    // Check if the extra account metas account is not empty
    extraAccountMetasAccount = await connection.getAccountInfo(
      extraAccountMetas
    );
    expect(extraAccountMetasAccount).to.not.equal(null);
  });

  it("should set the vtoken is transferrable to false", async () => {
    // Get the vault config account
    let [vaultConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("STARKE_VAULT_CONFIG"), mint.publicKey.toBuffer()],
      // [Buffer.from("STARKE_VAULT_CONFIG")],
      program.programId
    );

    // Get the vault config account
    let vaultConfigAccount = await program.account.vaultConfig.fetch(
      vaultConfig
    );
    expect(vaultConfigAccount.vtokenIsTransferrable).to.equal(true);

    const setVtokenIsTransferrableInstruction = await program.methods
      .setVtokenIsTransferrable(false)
      .accounts({
        manager: tester.publicKey,
        mint: mint.publicKey,
      })
      .instruction();

    const tx = new Transaction().add(setVtokenIsTransferrableInstruction);
    const txSig = await sendAndConfirmTransaction(connection, tx, [tester]);

    // Get the vault config account
    [vaultConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("STARKE_VAULT_CONFIG"), mint.publicKey.toBuffer()],
      // [Buffer.from("STARKE_VAULT_CONFIG")],
      program.programId
    );

    vaultConfigAccount = await program.account.vaultConfig.fetch(vaultConfig);
    expect(vaultConfigAccount.vtokenIsTransferrable).to.equal(false);
  });

  it("should create associated token accounts and mint tokens", async () => {
    // 100 tokens
    const amount = 100 * 10 ** decimals;

    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        tester.publicKey,
        sourceTokenAccount,
        tester.publicKey,
        mint.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createAssociatedTokenAccountInstruction(
        tester.publicKey,
        receiverTokenAccount,
        receiver.publicKey,
        mint.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createMintToInstruction(
        mint.publicKey,
        sourceTokenAccount,
        tester.publicKey,
        amount,
        [],
        TOKEN_2022_PROGRAM_ID
      )
    );
    const txSig = await sendAndConfirmTransaction(connection, transaction, [
      tester,
    ]);

    // Check if the source token account has the correct amount
    const sourceTokenAccountBalance = await connection.getTokenAccountBalance(
      sourceTokenAccount
    );
    expect(sourceTokenAccountBalance.value.amount).to.equal(amount.toString());
  });

  it("should transfer tokens from the source token account to the receiver token account", async () => {
    const amount = 1 * 10 ** decimals;

    const mintInfo = await getMint(
      connection,
      mint.publicKey,
      "confirmed",
      TOKEN_2022_PROGRAM_ID
    );
    const transferHook = getTransferHook(mintInfo);
    expect(transferHook.programId.equals(program.programId)).to.equal(true);

    // const extraAccountMetaAddress = getExtraAccountMetaAddress(
    //   mint.publicKey,
    //   transferHook.programId
    // );
    // const validateStateAccount = await connection.getAccountInfo(
    //   extraAccountMetaAddress,
    //   "confirmed"
    // );
    // const extraAccountMetaAccount = getExtraAccountMetas(validateStateAccount);
    // console.log("extraAccountMetaAccount", extraAccountMetaAccount);

    const xyz = await createTransferCheckedWithTransferHookInstruction(
      connection,
      sourceTokenAccount,
      mint.publicKey,
      receiverTokenAccount,
      tester.publicKey,
      BigInt(amount),
      decimals,
      [tester.publicKey],
      "confirmed",
      TOKEN_2022_PROGRAM_ID
    );

    const tx = new Transaction().add(xyz);
    const txSig = await sendAndConfirmTransaction(connection, tx, [tester], {
      skipPreflight: false,
    });
  });
});
