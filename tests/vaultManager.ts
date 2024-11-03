import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createAccount,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import { VaultManager } from "../target/types/vault_manager";
import { confirmTransaction, requestAirdrop } from "./utils";

// Add these constants at the top after imports
const DECIMALS = 6;
const TOKEN_FACTOR = Math.pow(10, DECIMALS);

// TODO: Move to utils
// Helper function to convert tokens to raw amount
const toTokenAmount = (tokens: number) => new anchor.BN(tokens * TOKEN_FACTOR);

describe("VaultManager", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.VaultManager as Program<VaultManager>;

  // Test accounts
  const manager = anchor.web3.Keypair.generate();
  const depositor1 = anchor.web3.Keypair.generate();
  const depositor2 = anchor.web3.Keypair.generate();

  // PDAs and other accounts
  let depositToken: PublicKey;
  let depositor1TokenAccount: PublicKey;
  let depositor2TokenAccount: PublicKey;
  let vaultTokenAccount: PublicKey;
  let vault: PublicKey;
  let vaultBump: number;
  let depositor1Account: PublicKey;
  let depositor2Account: PublicKey;
  let depositor1Bump: number;
  let depositor2Bump: number;

  before(async () => {
    // Airdrop SOL to accounts
    await requestAirdrop(manager.publicKey);
    await requestAirdrop(depositor1.publicKey);
    await requestAirdrop(depositor2.publicKey);

    // Create deposit token
    depositToken = await createMint(
      provider.connection,
      manager,
      manager.publicKey,
      null,
      6
    );

    // Create token accounts
    depositor1TokenAccount = await createAccount(
      provider.connection,
      depositor1,
      depositToken,
      depositor1.publicKey
    );

    depositor2TokenAccount = await createAccount(
      provider.connection,
      depositor2,
      depositToken,
      depositor2.publicKey
    );

    // Mint tokens to depositors
    await confirmTransaction(
      await mintTo(
        provider.connection,
        manager,
        depositToken,
        depositor1TokenAccount,
        manager.publicKey,
        1000 * TOKEN_FACTOR
      )
    );

    await confirmTransaction(
      await mintTo(
        provider.connection,
        manager,
        depositToken,
        depositor2TokenAccount,
        manager.publicKey,
        1000 * TOKEN_FACTOR
      )
    );

    // Derive vault PDA
    [vault, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), manager.publicKey.toBuffer()],
      program.programId
    );

    // Derive depositor PDAs
    [depositor1Account, depositor1Bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault_balance"),
        vault.toBuffer(),
        depositor1.publicKey.toBuffer(),
      ],
      program.programId
    );

    [depositor2Account, depositor2Bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault_balance"),
        vault.toBuffer(),
        depositor2.publicKey.toBuffer(),
      ],
      program.programId
    );

    // Create vault token account if not exists
    vaultTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        manager,
        depositToken,
        vault,
        true
      )
    ).address;
  });

  describe("create vault", () => {
    it("successfully creates a vault", async () => {
      await confirmTransaction(
        await program.methods
          .createVault("Test Vault")
          .accounts({
            manager: manager.publicKey,
            depositToken: depositToken,
          })
          .signers([manager])
          .rpc()
      );
      const vaultAccount = await program.account.vault.fetch(vault);
      expect(vaultAccount.manager).to.eql(manager.publicKey);
      expect(vaultAccount.depositToken).to.eql(depositToken);
      expect(vaultAccount.name).to.equal("Test Vault");
      expect(vaultAccount.bump).to.equal(vaultBump);
    });

    it("fails to create vault with same manager", async () => {
      try {
        await program.methods
          .createVault("Another Vault")
          .accounts({
            manager: manager.publicKey,
            depositToken: depositToken,
          })
          .signers([manager])
          .rpc();
        expect.fail("Should have failed");
      } catch (error) {
        expect(error).to.be.instanceOf(Error);
      }
    });
  });

  describe("deposit", () => {
    it("successfully deposits tokens from depositor1", async () => {
      const depositAmount = toTokenAmount(1);

      const initialVaultBalance =
        await provider.connection.getTokenAccountBalance(vaultTokenAccount);

      await confirmTransaction(
        await program.methods
          .deposit(depositAmount)
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: depositor1TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc()
      );

      // Verify token transfer
      const finalVaultBalance =
        await provider.connection.getTokenAccountBalance(vaultTokenAccount);
      expect(
        Number(finalVaultBalance.value.amount) -
          Number(initialVaultBalance.value.amount)
      ).to.equal(depositAmount.toNumber());

      // Verify depositor account state
      const vaultBalance = await program.account.vaultBalance.fetch(
        depositor1Account
      );
      expect(vaultBalance.vault).to.eql(vault);
      expect(vaultBalance.user).to.eql(depositor1.publicKey);
      expect(vaultBalance.amount.toNumber()).to.equal(depositAmount.toNumber());
    });

    it("successfully makes multiple deposits from same depositor", async () => {
      const depositAmount = toTokenAmount(0.5);

      // First deposit
      await confirmTransaction(
        await program.methods
          .deposit(depositAmount)
          .accounts({
            user: depositor2.publicKey,
            userTokenAccount: depositor2TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor2])
          .rpc()
      );

      // Second deposit
      await confirmTransaction(
        await program.methods
          .deposit(depositAmount)
          .accounts({
            user: depositor2.publicKey,
            userTokenAccount: depositor2TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor2])
          .rpc()
      );

      // Verify total deposits
      const vaultBalance = await program.account.vaultBalance.fetch(
        depositor2Account
      );
      expect(vaultBalance.amount.toNumber()).to.equal(
        depositAmount.toNumber() * 2
      );
    });

    it("fails when trying to deposit with wrong token mint", async () => {
      // Create a different token mint
      const wrongToken = await createMint(
        provider.connection,
        manager,
        manager.publicKey,
        null,
        6
      );

      const wrongTokenAccount = await createAccount(
        provider.connection,
        depositor1,
        wrongToken,
        depositor1.publicKey
      );

      try {
        await program.methods
          .deposit(toTokenAmount(1))
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: wrongTokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc();
        expect.fail("Should have failed with invalid deposit token");
      } catch (error) {
        expect(error.toString()).to.include("ConstraintRaw");
      }
    });

    it("fails when trying to deposit with insufficient funds", async () => {
      const tooMuchAmount = toTokenAmount(2000);

      try {
        await program.methods
          .deposit(tooMuchAmount)
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: depositor1TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc();
        expect.fail("Should have failed with insufficient funds");
      } catch (error) {
        expect(error.toString()).to.include("insufficient funds");
      }
    });

    it("emits a deposit event with correct data", async () => {
      const depositAmount = toTokenAmount(1);

      // Create a promise that will resolve when the event is received
      const eventPromise = new Promise<any>((resolve) => {
        const listener = program.addEventListener("depositMade", (event) => {
          resolve(event);
        });

        // Clean up listener after we're done
        setTimeout(() => {
          program.removeEventListener(listener);
        }, 5000);
      });

      // Execute the deposit transaction
      await confirmTransaction(
        await program.methods
          .deposit(depositAmount)
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: depositor1TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc()
      );

      // Wait for and verify the event
      const event = await eventPromise;
      expect(event.vault.toString()).to.equal(vault.toString());
      expect(event.user.toString()).to.equal(depositor1.publicKey.toString());
      expect(event.amount.toString()).to.equal(depositAmount.toString());
      expect(event.totalDeposited.toString()).to.equal(
        depositAmount.mul(new anchor.BN(2)).toString()
      );
    });

    it("creates depositor account with correct bump and data", async () => {
      const depositAmount = toTokenAmount(0.1);

      await confirmTransaction(
        await program.methods
          .deposit(depositAmount)
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: depositor1TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc()
      );

      // Fetch and verify depositor account
      const vaultBalance = await program.account.vaultBalance.fetch(
        depositor1Account
      );

      // Verify account data
      expect(vaultBalance.vault.toString()).to.equal(vault.toString());
      expect(vaultBalance.user.toString()).to.equal(
        depositor1.publicKey.toString()
      );
      expect(vaultBalance.bump).to.equal(depositor1Bump);

      // Verify PDA derivation matches
      const [derivedPDA, derivedBump] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vault_balance"),
          vault.toBuffer(),
          depositor1.publicKey.toBuffer(),
        ],
        program.programId
      );

      expect(derivedPDA.toString()).to.equal(depositor1Account.toString());
      expect(derivedBump).to.equal(depositor1Bump);
    });
  });

  describe("withdraw", () => {
    let vaultBalance: anchor.BN;

    beforeEach(async () => {
      // Make initial deposit to test withdrawals
      await confirmTransaction(
        await program.methods
          .deposit(toTokenAmount(10))
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: depositor1TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc()
      );

      vaultBalance = (
        await program.account.vaultBalance.fetch(depositor1Account)
      ).amount;
    });

    it("successfully withdraws tokens", async () => {
      const withdrawAmount = toTokenAmount(3);

      const initialUserBalance =
        await provider.connection.getTokenAccountBalance(
          depositor1TokenAccount
        );

      await confirmTransaction(
        await program.methods
          .withdraw(withdrawAmount)
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: depositor1TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc()
      );

      // Verify token transfer
      const finalUserBalance = await provider.connection.getTokenAccountBalance(
        depositor1TokenAccount
      );
      expect(
        Number(finalUserBalance.value.amount) -
          Number(initialUserBalance.value.amount)
      ).to.equal(withdrawAmount.toNumber());

      // Verify vault balance update
      const finalVaultBalance = await program.account.vaultBalance.fetch(
        depositor1Account
      );
      expect(finalVaultBalance.amount.toNumber()).to.equal(
        vaultBalance.sub(withdrawAmount).toNumber()
      );
    });

    it("deletes vault balance account when fully withdrawn", async () => {
      // Withdraw full amount
      await confirmTransaction(
        await program.methods
          .withdraw(vaultBalance)
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: depositor1TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc()
      );

      // Verify account is deleted
      const accountInfo = await provider.connection.getAccountInfo(
        depositor1Account
      );
      expect(accountInfo).to.be.null;
    });

    it("fails when trying to withdraw more than deposited", async () => {
      const tooMuchAmount = vaultBalance.add(new anchor.BN(1));

      try {
        await program.methods
          .withdraw(tooMuchAmount)
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: depositor1TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc();
        expect.fail("Should have failed with insufficient funds");
      } catch (error) {
        expect(error.toString()).to.include("InsufficientFunds");
      }
    });

    it("emits a withdraw event with correct data", async () => {
      const withdrawAmount = toTokenAmount(2);

      const eventPromise = new Promise<any>((resolve) => {
        const listener = program.addEventListener("withdrawMade", (event) => {
          resolve(event);
        });

        setTimeout(() => {
          program.removeEventListener(listener);
        }, 5000);
      });

      await confirmTransaction(
        await program.methods
          .withdraw(withdrawAmount)
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: depositor1TokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc()
      );

      const event = await eventPromise;
      expect(event.vault.toString()).to.equal(vault.toString());
      expect(event.user.toString()).to.equal(depositor1.publicKey.toString());
      expect(event.amount.toString()).to.equal(withdrawAmount.toString());
      expect(event.remainingBalance.toString()).to.equal(
        vaultBalance.sub(withdrawAmount).toString()
      );
    });

    it("fails when trying to withdraw with wrong token account", async () => {
      const wrongToken = await createMint(
        provider.connection,
        manager,
        manager.publicKey,
        null,
        6
      );

      const wrongTokenAccount = await createAccount(
        provider.connection,
        depositor1,
        wrongToken,
        depositor1.publicKey
      );

      try {
        await program.methods
          .withdraw(toTokenAmount(1))
          .accounts({
            user: depositor1.publicKey,
            userTokenAccount: wrongTokenAccount,
            manager: manager.publicKey,
            vaultTokenAccount,
          })
          .signers([depositor1])
          .rpc();
        expect.fail("Should have failed with invalid token account");
      } catch (error) {
        expect(error.toString()).to.include("ConstraintRaw");
      }
    });
  });
});
