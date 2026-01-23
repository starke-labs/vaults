import { BN } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import {
  getAssociatedTokenAddress,
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

import { VaultsSdk } from "@starke/sdk";
import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
  getTesterKeypair,
  toTokenAmount,
} from "../utils.new";
import { getVaultDepositFeeConfigPda, getVaultPda } from "@starke/sdk/lib/pdas";

describe("Deposit Fee", () => {
  let depositor: Keypair;
  let vaults: VaultsSdk;
  let authorityVaults: VaultsSdk;
  let manager: PublicKey;
  let vault: any;
  let authority: Keypair;

  const FEE_RATE = 50; // 0.5% in basis points (50/10000)
  const DEPOSIT_AMOUNT = 1000; // tokens

  before(async () => {
    depositor = getTesterKeypair();
    authority = getAuthorityKeypair();
    manager = getManagerKeypair().publicKey;

    vaults = new VaultsSdk(createConnection(), depositor);
    authorityVaults = new VaultsSdk(createConnection(), authority);

    vault = await vaults.fetchVault(manager);
  });

  describe("Fee Configuration", () => {
    it("should set deposit fee config", async () => {
      const [vaultPda] = getVaultPda(manager);
      const [depositFeeConfigPda] = getVaultDepositFeeConfigPda(vaultPda);

      // Use program directly since SDK doesn't have this method yet
      const program = (authorityVaults as any).program;
      const platformFeeRecipient = authority.publicKey;

      try {
        const tx = await program.methods
          .setVaultDepositFee(true, FEE_RATE, platformFeeRecipient)
          .accounts({
            authority: authority.publicKey,
            manager: manager,
            vault: vaultPda,
            depositFeeConfig: depositFeeConfigPda,
            starkeConfig: vault.starkeConfig,
            systemProgram: PublicKey.default,
          })
          .signers([authority])
          .rpc();

        console.log("Set vault deposit fee tx:", tx);

        // Verify the config was set
        const feeConfig = await program.account.vaultDepositFeeConfig.fetch(
          depositFeeConfigPda
        );
        expect(feeConfig.vault.toBase58()).to.equal(vaultPda.toBase58());
        expect(feeConfig.enabled).to.be.true;
        expect(feeConfig.feeRate.toNumber()).to.equal(FEE_RATE);
      } catch (error) {
        // Config might already exist, try to update
        if (
          error.toString().includes("already in use") ||
          error.toString().includes("0x0")
        ) {
          const tx = await program.methods
            .setVaultDepositFee(true, FEE_RATE, platformFeeRecipient)
            .accounts({
              authority: authority.publicKey,
              manager: manager,
              vault: vaultPda,
              depositFeeConfig: depositFeeConfigPda,
              starkeConfig: vault.starkeConfig,
              systemProgram: PublicKey.default,
            })
            .signers([authority])
            .rpc();
          console.log("Updated vault deposit fee tx:", tx);
        } else {
          throw error;
        }
      }
    });

    it("should enable deposit fee", async () => {
      const [vaultPda] = getVaultPda(manager);
      const [depositFeeConfigPda] = getVaultDepositFeeConfigPda(vaultPda);
      const program = (authorityVaults as any).program;

      const tx = await program.methods
        .enableVaultDepositFee()
        .accounts({
          authority: authority.publicKey,
          manager: manager,
          vault: vaultPda,
          depositFeeConfig: depositFeeConfigPda,
          starkeConfig: vault.starkeConfig,
        })
        .signers([authority])
        .rpc();

      console.log("Enable deposit fee tx:", tx);

      const feeConfig = await program.account.vaultDepositFeeConfig.fetch(
        depositFeeConfigPda
      );
      expect(feeConfig.enabled).to.be.true;
    });

    it("should disable deposit fee", async () => {
      const [vaultPda] = getVaultPda(manager);
      const [depositFeeConfigPda] = getVaultDepositFeeConfigPda(vaultPda);
      const program = (authorityVaults as any).program;

      const tx = await program.methods
        .disableVaultDepositFee()
        .accounts({
          authority: authority.publicKey,
          manager: manager,
          vault: vaultPda,
          depositFeeConfig: depositFeeConfigPda,
          starkeConfig: vault.starkeConfig,
        })
        .signers([authority])
        .rpc();

      console.log("Disable deposit fee tx:", tx);

      const feeConfig = await program.account.vaultDepositFeeConfig.fetch(
        depositFeeConfigPda
      );
      expect(feeConfig.enabled).to.be.false;
    });
  });

  describe("Deposit with Fee", () => {
    beforeEach(async () => {
      // Ensure fee is enabled before each deposit test
      const [vaultPda] = getVaultPda(manager);
      const [depositFeeConfigPda] = getVaultDepositFeeConfigPda(vaultPda);
      const program = (authorityVaults as any).program;

      try {
        await program.methods
          .enableVaultDepositFee()
          .accounts({
            authority: authority.publicKey,
            manager: manager,
            vault: vaultPda,
            depositFeeConfig: depositFeeConfigPda,
            starkeConfig: vault.starkeConfig,
          })
          .signers([authority])
          .rpc();
      } catch (error) {
        // Fee might already be enabled or config doesn't exist yet
        console.log("Fee enable skipped:", error.toString());
      }
    });

    it("should deposit with fee enabled and transfer fee to platform", async () => {
      const [vaultPda] = getVaultPda(manager);
      const depositTokenMint = vault.depositTokenMint;
      const platformFeeRecipient = authority.publicKey;

      // Get token accounts
      const depositorDepositTokenAccount = await getAssociatedTokenAddress(
        depositTokenMint,
        depositor.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );

      const platformFeeRecipientTokenAccount = await getAssociatedTokenAddress(
        depositTokenMint,
        platformFeeRecipient,
        false,
        TOKEN_PROGRAM_ID
      );

      const vaultDepositTokenAccount = await getAssociatedTokenAddress(
        depositTokenMint,
        vaultPda,
        true,
        TOKEN_PROGRAM_ID
      );

      // Get initial balances
      const depositorBalanceBefore = await getAccount(
        createConnection(),
        depositorDepositTokenAccount
      );
      const platformBalanceBefore = await getAccount(
        createConnection(),
        platformFeeRecipientTokenAccount
      ).catch(() => null);
      const vaultBalanceBefore = await getAccount(
        createConnection(),
        vaultDepositTokenAccount
      ).catch(() => null);

      const depositAmount = toTokenAmount(DEPOSIT_AMOUNT);
      const expectedFee = depositAmount.mul(new BN(FEE_RATE)).div(new BN(10000));
      const expectedNetDeposit = depositAmount.sub(expectedFee);

      console.log("Deposit amount:", depositAmount.toString());
      console.log("Expected fee:", expectedFee.toString());
      console.log("Expected net deposit:", expectedNetDeposit.toString());

      // Perform deposit using SDK (which handles fee config automatically)
      try {
        await vaults.deposit(depositAmount, depositor.publicKey, manager, [
          authority,
        ]);
      } catch (error) {
        // If price is too old, skip this test
        if (
          error.toString().includes("PriceTooOld") ||
          error.toString().includes("price feed update's age exceeds")
        ) {
          console.log("Skipping deposit fee test due to stale price feed data");
          return;
        }
        throw error;
      }

      // Check balances after deposit
      const depositorBalanceAfter = await getAccount(
        createConnection(),
        depositorDepositTokenAccount
      );
      const platformBalanceAfter = await getAccount(
        createConnection(),
        platformFeeRecipientTokenAccount
      ).catch(() => null);
      const vaultBalanceAfter = await getAccount(
        createConnection(),
        vaultDepositTokenAccount
      );

      // Verify fee was transferred to platform
      if (platformBalanceAfter) {
        const platformBalanceChange =
          platformBalanceAfter.amount -
          (platformBalanceBefore?.amount || BigInt(0));
        expect(platformBalanceChange.toString()).to.equal(
          expectedFee.toString()
        );
      }

      // Verify net deposit was transferred to vault
      const vaultBalanceChange =
        vaultBalanceAfter.amount - (vaultBalanceBefore?.amount || BigInt(0));
      expect(vaultBalanceChange.toString()).to.equal(
        expectedNetDeposit.toString()
      );

      // Verify depositor balance decreased by full amount
      const depositorBalanceChange =
        depositorBalanceBefore.amount - depositorBalanceAfter.amount;
      expect(depositorBalanceChange.toString()).to.equal(
        depositAmount.toString()
      );
    });

    it("should deposit without fee when fee is disabled", async () => {
      // Disable fee first
      const [vaultPda] = getVaultPda(manager);
      const [depositFeeConfigPda] = getVaultDepositFeeConfigPda(vaultPda);
      const program = (authorityVaults as any).program;

      await program.methods
        .disableVaultDepositFee()
        .accounts({
          authority: authority.publicKey,
          manager: manager,
          vault: vaultPda,
          depositFeeConfig: depositFeeConfigPda,
          starkeConfig: vault.starkeConfig,
        })
        .signers([authority])
        .rpc();

      const depositTokenMint = vault.depositTokenMint;
      const platformFeeRecipient = authority.publicKey;

      // Get token accounts
      const platformFeeRecipientTokenAccount = await getAssociatedTokenAddress(
        depositTokenMint,
        platformFeeRecipient,
        false,
        TOKEN_PROGRAM_ID
      );

      const vaultDepositTokenAccount = await getAssociatedTokenAddress(
        depositTokenMint,
        vaultPda,
        true,
        TOKEN_PROGRAM_ID
      );

      // Get initial balances
      const platformBalanceBefore = await getAccount(
        createConnection(),
        platformFeeRecipientTokenAccount
      ).catch(() => null);
      const vaultBalanceBefore = await getAccount(
        createConnection(),
        vaultDepositTokenAccount
      ).catch(() => null);

      const depositAmount = toTokenAmount(DEPOSIT_AMOUNT);

      // Perform deposit using SDK
      try {
        await vaults.deposit(depositAmount, depositor.publicKey, manager, [
          authority,
        ]);
      } catch (error) {
        // If price is too old, skip this test
        if (
          error.toString().includes("PriceTooOld") ||
          error.toString().includes("price feed update's age exceeds")
        ) {
          console.log(
            "Skipping deposit without fee test due to stale price feed data"
          );
          return;
        }
        throw error;
      }

      // Check balances after deposit
      const platformBalanceAfter = await getAccount(
        createConnection(),
        platformFeeRecipientTokenAccount
      ).catch(() => null);
      const vaultBalanceAfter = await getAccount(
        createConnection(),
        vaultDepositTokenAccount
      );

      // Verify no fee was transferred to platform
      const platformBalanceChange =
        (platformBalanceAfter?.amount || BigInt(0)) -
        (platformBalanceBefore?.amount || BigInt(0));
      expect(platformBalanceChange.toString()).to.equal("0");

      // Verify full deposit was transferred to vault (no fee deducted)
      const vaultBalanceChange =
        vaultBalanceAfter.amount - (vaultBalanceBefore?.amount || BigInt(0));
      expect(vaultBalanceChange.toString()).to.equal(depositAmount.toString());
    });
  });
});
