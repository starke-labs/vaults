import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, Wallet } from "@coral-xyz/anchor";
import {
  createAccount,
  createMint,
  getAssociatedTokenAddress,
  mintTo,
} from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import idl from "@starke/idl/vaults.json";
import { VaultsSDK } from "@starke/sdk";

describe("Vaults Integration Tests", () => {
  // Constants
  const DECIMALS = 6;
  const TOKEN_FACTOR = Math.pow(10, DECIMALS);
  const toTokenAmount = (tokens: number) =>
    new anchor.BN(tokens * TOKEN_FACTOR);

  // Test setup
  let sdk: VaultsSDK;
  let provider: AnchorProvider;
  let manager: Keypair;
  let depositor: Keypair;
  let tokenMint: PublicKey;
  let managerTokenAccount: PublicKey;
  let depositorTokenAccount: PublicKey;
  let vaultAddress: PublicKey;
  let vaultTokenMint: PublicKey;

  before(async () => {
    // Setup provider and accounts
    provider = AnchorProvider.env();
    anchor.setProvider(provider);

    manager = Keypair.generate();
    depositor = Keypair.generate();

    // Fund accounts
    await provider.connection.requestAirdrop(
      manager.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      depositor.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );

    // Initialize SDK
    const programId = new PublicKey(idl.metadata.address);
    sdk = new VaultsSDK(
      provider.connection,
      new Wallet(manager),
      programId,
      idl
    );

    // Create test token
    tokenMint = await createMint(
      provider.connection,
      manager,
      manager.publicKey,
      null,
      DECIMALS
    );

    // Create token accounts
    managerTokenAccount = await createAccount(
      provider.connection,
      manager,
      tokenMint,
      manager.publicKey
    );

    depositorTokenAccount = await createAccount(
      provider.connection,
      depositor,
      tokenMint,
      depositor.publicKey
    );

    // Mint initial tokens
    await mintTo(
      provider.connection,
      manager,
      tokenMint,
      managerTokenAccount,
      manager.publicKey,
      1000 * TOKEN_FACTOR
    );

    await mintTo(
      provider.connection,
      manager,
      tokenMint,
      depositorTokenAccount,
      manager.publicKey,
      1000 * TOKEN_FACTOR
    );
  });

  describe("Vault Creation and Setup", () => {
    it("should initialize token whitelist", async () => {
      const tx = new anchor.web3.Transaction();
      tx.add(await sdk.initializeWhitelist());

      const txId = await provider.sendAndConfirm(tx);
      expect(txId).to.be.a("string");
    });

    it("should whitelist a token", async () => {
      const tx = new anchor.web3.Transaction();
      tx.add(await sdk.addToken({ mint: tokenMint }));

      const txId = await provider.sendAndConfirm(tx);
      expect(txId).to.be.a("string");
    });

    it("should create a new vault", async () => {
      const vaultConfig = {
        name: "Test Vault",
        entryFee: 100, // 1%
        exitFee: 200, // 2%
      };

      const tx = new anchor.web3.Transaction();
      tx.add(await sdk.createVault(vaultConfig));

      const txId = await provider.sendAndConfirm(tx);
      expect(txId).to.be.a("string");

      // Store vault address for subsequent tests
      // Note: You'll need to implement a way to get the vault address
      // vaultAddress = ...
    });
  });

  describe("Vault Operations", () => {
    it("should deposit tokens into vault", async () => {
      const depositAmount = toTokenAmount(100);
      const tx = new anchor.web3.Transaction();
      tx.add(
        await sdk.deposit({
          vault: vaultAddress,
          amount: depositAmount,
        })
      );

      const txId = await provider.sendAndConfirm(tx);
      expect(txId).to.be.a("string");

      // Verify deposit
      // Add balance checks here
    });

    it("should withdraw tokens from vault", async () => {
      const withdrawAmount = toTokenAmount(50);
      const tx = new anchor.web3.Transaction();
      tx.add(
        await sdk.withdraw({
          vault: vaultAddress,
          amount: withdrawAmount,
        })
      );

      const txId = await provider.sendAndConfirm(tx);
      expect(txId).to.be.a("string");

      // Verify withdrawal
      // Add balance checks here
    });

    it("should update vault fees", async () => {
      const tx = new anchor.web3.Transaction();
      tx.add(
        await sdk.updateVaultFees({
          vault: vaultAddress,
          newEntryFee: 150,
          newExitFee: 250,
        })
      );

      const txId = await provider.sendAndConfirm(tx);
      expect(txId).to.be.a("string");

      // Verify fee update
      // Add fee checks here
    });
  });

  describe("Event Handling", () => {
    it("should receive vault creation events", (done) => {
      sdk.events.subscribeVaultCreated((event) => {
        expect(event).to.not.be.null;
        // Add specific event validations
        done();
      });

      // Create a new vault to trigger the event
      const vaultConfig = {
        name: "Event Test Vault",
        entryFee: 100,
        exitFee: 200,
      };

      sdk
        .createVault(vaultConfig)
        .then((ix) => {
          const tx = new anchor.web3.Transaction().add(ix);
          return provider.sendAndConfirm(tx);
        })
        .catch(done);
    });

    // Add more event tests for deposits, withdrawals, etc.
  });
});
