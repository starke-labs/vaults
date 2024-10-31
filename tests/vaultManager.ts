import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
} from "@solana/spl-token";
import { expect } from "chai";

import { VaultManager } from "../target/types/vault_manager";
import { requestAirdrop, confirmTransaction } from "./utils";

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
        1000000000
      )
    );

    await confirmTransaction(
      await mintTo(
        provider.connection,
        manager,
        depositToken,
        depositor2TokenAccount,
        manager.publicKey,
        1000000000
      )
    );

    // Derive vault PDA
    [vault, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), manager.publicKey.toBuffer()],
      program.programId
    );
  });

  describe("create_vault", () => {
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
    let depositor1Account: PublicKey;
    let depositor2Account: PublicKey;
    let depositor1Bump: number;
    let depositor2Bump: number;

    before(async () => {
      // Derive depositor PDAs
      [depositor1Account, depositor1Bump] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("depositor"),
          vault.toBuffer(),
          depositor1.publicKey.toBuffer(),
        ],
        program.programId
      );

      [depositor2Account, depositor2Bump] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("depositor"),
          vault.toBuffer(),
          depositor2.publicKey.toBuffer(),
        ],
        program.programId
      );

      // Create vault token account if not exists
      vaultTokenAccount = await createAccount(
        provider.connection,
        manager,
        depositToken,
        manager.publicKey
      );
    });

    // TODO: Error when running `anchor test`
    //       `Error: Reached maximum depth for account resolution`
    //       Need to fix this
    it("successfully deposits tokens from depositor1", async () => {
      const depositAmount = new anchor.BN(1000000); // 1 token with 6 decimals

      const initialVaultBalance =
        await provider.connection.getTokenAccountBalance(vaultTokenAccount);

      await confirmTransaction(
        await program.methods
          .deposit(depositAmount)
          .accounts({
            depositor: depositor1.publicKey,
            depositorTokenAccount: depositor1TokenAccount,
            vaultTokenAccount,
            depositToken,
          })
          .signers([depositor1])
          .rpc()
      );

      // TODO: Need to check if the following tests are working as expected
      // // Verify token transfer
      // const finalVaultBalance =
      //   await provider.connection.getTokenAccountBalance(vaultTokenAccount);
      // expect(
      //   Number(finalVaultBalance.value.amount) -
      //     Number(initialVaultBalance.value.amount)
      // ).to.equal(depositAmount.toNumber());

      // // Verify depositor account state
      // const depositorAccount = await program.account.depositor.fetch(
      //   depositor1Account
      // );
      // expect(depositorAccount.vault).to.eql(vault);
      // expect(depositorAccount.depositor).to.eql(depositor1.publicKey);
      // expect(depositorAccount.amount.toNumber()).to.equal(
      //   depositAmount.toNumber()
      // );
    });

    // it("successfully makes multiple deposits from same depositor", async () => {
    //   const depositAmount = new anchor.BN(500000); // 0.5 tokens

    //   // First deposit
    //   await program.methods
    //     .deposit(depositAmount)
    //     .accounts({
    //       depositor: depositor2.publicKey,
    //       depositorTokenAccount: depositor2TokenAccount,
    //       vaultTokenAccount,
    //       depositorTokenAccount: depositor2Account,
    //       depositToken,
    //       systemProgram: SystemProgram.programId,
    //       clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    //     })
    //     .signers([depositor2])
    //     .rpc();

    //   // Second deposit
    //   await program.methods
    //     .deposit(depositAmount)
    //     .accounts({
    //       depositor: depositor2.publicKey,
    //       depositorTokenAccount: depositor2TokenAccount,
    //       vaultTokenAccount,
    //       depositorTokenAccount: depositor2Account,
    //       depositToken,
    //       systemProgram: SystemProgram.programId,
    //       clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    //     })
    //     .signers([depositor2])
    //     .rpc();

    //   // Verify total deposits
    //   const depositorAccount = await program.account.depositor.fetch(
    //     depositor2Account
    //   );
    //   expect(depositorAccount.amount.toNumber()).to.equal(
    //     depositAmount.toNumber() * 2
    //   );
    // });

    // it("fails when trying to deposit with wrong token mint", async () => {
    //   // Create a different token mint
    //   const wrongToken = await createMint(
    //     provider.connection,
    //     manager,
    //     manager.publicKey,
    //     null,
    //     6
    //   );

    //   const wrongTokenAccount = await createAccount(
    //     provider.connection,
    //     depositor1,
    //     wrongToken,
    //     depositor1.publicKey
    //   );

    //   try {
    //     await program.methods
    //       .deposit(new anchor.BN(1000000))
    //       .accounts({
    //         depositor: depositor1.publicKey,
    //         depositorTokenAccount: wrongTokenAccount,
    //         vault,
    //         vaultTokenAccount,
    //         depositorAccount: depositor1Account,
    //         tokenProgram: TOKEN_PROGRAM_ID,
    //         systemProgram: SystemProgram.programId,
    //         clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    //       })
    //       .signers([depositor1])
    //       .rpc();
    //     expect.fail("Should have failed with invalid deposit token");
    //   } catch (error) {
    //     expect(error.toString()).to.include("InvalidDepositToken");
    //   }
    // });

    // it("fails when trying to deposit with insufficient funds", async () => {
    //   const tooMuchAmount = new anchor.BN(2000000000); // More than minted

    //   try {
    //     await program.methods
    //       .deposit(tooMuchAmount)
    //       .accounts({
    //         depositor: depositor1.publicKey,
    //         depositorTokenAccount: depositor1TokenAccount,
    //         vault,
    //         vaultTokenAccount,
    //         depositorAccount: depositor1Account,
    //         tokenProgram: TOKEN_PROGRAM_ID,
    //         systemProgram: SystemProgram.programId,
    //         clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    //       })
    //       .signers([depositor1])
    //       .rpc();
    //     expect.fail("Should have failed with insufficient funds");
    //   } catch (error) {
    //     expect(error.toString()).to.include("insufficient funds");
    //   }
    // });
  });
});
