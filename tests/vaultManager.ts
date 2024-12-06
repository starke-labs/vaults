import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createAccount,
  createMint,
  getAssociatedTokenAddress,
  mintTo,
} from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import fs from "fs";

import { Vaults } from "../target/types/vaults";
import { confirmTransaction, requestAirdrop } from "./utils";

// Add these constants at the top after imports
const DECIMALS = 6;
const TOKEN_FACTOR = Math.pow(10, DECIMALS);
const WHITELIST_SEED = "STARKE_TOKEN_WHITELIST";
const VAULT_SEED = "STARKE_VAULT";
const VAULT_TOKEN_MINT_SEED = "STARKE_VAULT_TOKEN_MINT";

// TODO: Move to utils
// Helper function to convert tokens to raw amount
const toTokenAmount = (tokens: number) => new anchor.BN(tokens * TOKEN_FACTOR);

describe("Vaults", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Vaults as Program<Vaults>;

  // Test accounts
  const programAuthority = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(
      JSON.parse(fs.readFileSync("./deploy/authority.json", "utf8"))
    )
  );
  const manager = anchor.web3.Keypair.generate();
  const depositor1 = anchor.web3.Keypair.generate();
  const depositor2 = anchor.web3.Keypair.generate();

  // Deposit token mint and associated token accounts (ATAs)
  let depositTokenMint: PublicKey;
  let depositor1DepositTokenATA: PublicKey;
  let depositor2DepositTokenATA: PublicKey;
  let vaultDepositTokenATA: PublicKey;

  // Vault, vault token mint, and associated token accounts (ATAs)
  let vault: PublicKey;
  let vaultBump: number;
  let vaultTokenMint: PublicKey;
  let vaultTokenMintBump: number;
  let depositor1VaultTokenATA: PublicKey;
  let depositor2VaultTokenATA: PublicKey;

  // Whitelist
  let whitelist: PublicKey;
  let whitelistBump: number;

  before(async () => {
    // Airdrop SOL to accounts
    await requestAirdrop(manager.publicKey);
    await requestAirdrop(depositor1.publicKey);
    await requestAirdrop(depositor2.publicKey);

    // Create deposit token
    depositTokenMint = await createMint(
      provider.connection,
      manager,
      manager.publicKey,
      null,
      DECIMALS
    );

    // Create token accounts for deposit token
    depositor1DepositTokenATA = await createAccount(
      provider.connection,
      depositor1,
      depositTokenMint,
      depositor1.publicKey
    );

    depositor2DepositTokenATA = await createAccount(
      provider.connection,
      depositor2,
      depositTokenMint,
      depositor2.publicKey
    );

    // Mint tokens to depositors
    await confirmTransaction(
      await mintTo(
        provider.connection,
        manager,
        depositTokenMint,
        depositor1DepositTokenATA,
        manager.publicKey,
        1000 * TOKEN_FACTOR
      )
    );

    await confirmTransaction(
      await mintTo(
        provider.connection,
        manager,
        depositTokenMint,
        depositor2DepositTokenATA,
        manager.publicKey,
        1000 * TOKEN_FACTOR
      )
    );

    // Derive PDAs
    [vault, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from(VAULT_SEED), manager.publicKey.toBuffer()],
      program.programId
    );

    [vaultTokenMint, vaultTokenMintBump] = PublicKey.findProgramAddressSync(
      [Buffer.from(VAULT_TOKEN_MINT_SEED), vault.toBuffer()],
      program.programId
    );

    // Create associated token accounts for vault tokens
    vaultDepositTokenATA = await getAssociatedTokenAddress(
      depositTokenMint,
      vault,
      true
    );

    depositor1VaultTokenATA = await getAssociatedTokenAddress(
      vaultTokenMint,
      depositor1.publicKey
    );

    depositor2VaultTokenATA = await getAssociatedTokenAddress(
      vaultTokenMint,
      depositor2.publicKey
    );

    [whitelist, whitelistBump] = PublicKey.findProgramAddressSync(
      [Buffer.from(WHITELIST_SEED)],
      program.programId
    );
  });

  describe("whitelist", () => {
    it("successfully initializes whitelist", async () => {
      await confirmTransaction(
        await program.methods
          .initializeWhitelist()
          .accounts({ authority: programAuthority.publicKey })
          .signers([programAuthority])
          .rpc()
      );

      const whitelistAccount = await program.account.tokenWhitelist.fetch(
        whitelist
      );
      expect(whitelistAccount.authority.toString()).to.equal(
        programAuthority.publicKey.toString()
      );
      expect(whitelistAccount.tokens).to.have.length(0);
    });

    it("successfully adds token to whitelist", async () => {
      await confirmTransaction(
        await program.methods
          .addToken(depositTokenMint)
          .accounts({
            authority: programAuthority.publicKey,
          })
          .signers([programAuthority])
          .rpc()
      );

      const whitelistAccount = await program.account.tokenWhitelist.fetch(
        whitelist
      );
      expect(whitelistAccount.tokens).to.have.length(1);
      expect(whitelistAccount.tokens[0].toString()).to.equal(
        depositTokenMint.toString()
      );
    });

    it("fails to add same token twice", async () => {
      try {
        await program.methods
          .addToken(depositTokenMint)
          .accounts({
            authority: programAuthority.publicKey,
          })
          .signers([programAuthority])
          .rpc();
        expect.fail("Should have failed");
      } catch (err) {
        expect(err.toString()).to.include("TokenAlreadyWhitelisted");
      }
    });

    it("fails when non-program authority tries to add token", async () => {
      try {
        await program.methods
          .addToken(depositTokenMint)
          .accounts({
            authority: manager.publicKey,
          })
          .signers([manager])
          .rpc();
        expect.fail("Should have failed");
      } catch (err) {
        expect(err.toString()).to.include("UnauthorizedAccess");
      }
    });
  });

  describe("create vault", () => {
    it("successfully creates a vault", async () => {
      await confirmTransaction(
        await program.methods
          .createVault("Test Vault")
          .accounts({
            manager: manager.publicKey,
            depositTokenMint,
          })
          .signers([manager])
          .rpc()
      );
      const vaultAccount = await program.account.vault.fetch(vault);
      expect(vaultAccount.manager).to.eql(manager.publicKey);
      expect(vaultAccount.depositTokenMint).to.eql(depositTokenMint);
      expect(vaultAccount.name).to.equal("Test Vault");
      expect(vaultAccount.bump).to.equal(vaultBump);
      expect(vaultAccount.mint).to.eql(vaultTokenMint);
      expect(vaultAccount.mintBump).to.equal(vaultTokenMintBump);

      // Check if vault token mint has been created
      const vaultTokenMintAccount = await provider.connection.getAccountInfo(
        vaultTokenMint
      );
      expect(vaultTokenMintAccount).to.not.be.null;
    });

    it("fails to create vault with same manager", async () => {
      try {
        await program.methods
          .createVault("Another Vault")
          .accounts({
            manager: manager.publicKey,
            depositTokenMint,
          })
          .signers([manager])
          .rpc();
        expect.fail("Should have failed");
      } catch (error) {
        // 0x0 means you're attempting to initialize an already initialized account
        expect(error.toString()).to.include("0x0");
      }
    });

    it("fails to create vault with non-whitelisted token", async () => {
      const newManager = anchor.web3.Keypair.generate();
      await requestAirdrop(newManager.publicKey);

      const nonWhitelistedToken = await createMint(
        provider.connection,
        newManager,
        newManager.publicKey,
        null,
        6
      );

      const whitelistAccount = await program.account.tokenWhitelist.fetch(
        whitelist
      );
      expect(whitelistAccount.tokens).to.have.length(1);
      expect(whitelistAccount.tokens[0].toString()).to.equal(
        depositTokenMint.toString()
      );

      try {
        await program.methods
          .createVault("Test Vault")
          .accounts({
            manager: newManager.publicKey,
            depositTokenMint: nonWhitelistedToken,
          })
          .signers([newManager])
          .rpc();
        expect.fail("Should have failed");
      } catch (error) {
        expect(error.toString()).to.include("TokenNotWhitelisted");
      }
    });
  });

  describe("interact with vault", () => {
    afterEach(async () => {
      // Check and withdraw depositor1's balance if any
      try {
        const balance1 = await provider.connection.getTokenAccountBalance(
          depositor1VaultTokenATA
        );
        const amount1 = new anchor.BN(balance1.value.amount);
        if (amount1.gt(new anchor.BN(0))) {
          await confirmTransaction(
            await program.methods
              .withdraw(amount1)
              .accounts({
                user: depositor1.publicKey,
                manager: manager.publicKey,
                depositTokenMint,
              })
              .signers([depositor1])
              .rpc()
          );
        }
      } catch (e) {
        // Account may not exist yet, ignore error
      }

      // Check and withdraw depositor2's balance if any
      try {
        const balance2 = await provider.connection.getTokenAccountBalance(
          depositor2VaultTokenATA
        );
        const amount2 = new anchor.BN(balance2.value.amount);
        if (amount2.gt(new anchor.BN(0))) {
          await confirmTransaction(
            await program.methods
              .withdraw(amount2)
              .accounts({
                user: depositor2.publicKey,
                manager: manager.publicKey,
                depositTokenMint,
              })
              .signers([depositor2])
              .rpc()
          );
        }
      } catch (e) {
        // Account may not exist yet, ignore error
      }

      // Verify vault is empty
      const vaultTokenBalance =
        await provider.connection.getTokenAccountBalance(vaultDepositTokenATA);
      expect(vaultTokenBalance.value.amount).to.equal("0");

      // Verify vault token mint supply
      const mintInfo = await provider.connection.getTokenSupply(vaultTokenMint);
      expect(mintInfo.value.amount).to.equal("0");
    });

    // NOTE: This is not needed as total deposits are not tracked anymore, we use the vault token mint supply instead
    // describe("total deposits", () => {
    //   it("correctly calculates user share of the vault", async () => {
    //     // Depositor1 deposits 75% of total
    //     const deposit1Amount = toTokenAmount(75);
    //     await confirmTransaction(
    //       await program.methods
    //         .deposit(deposit1Amount)
    //         .accounts({
    //           user: depositor1.publicKey,
    //           manager: manager.publicKey,
    //           depositTokenMint,
    //         })
    //         .signers([depositor1])
    //         .rpc()
    //     );

    //     // Depositor2 deposits 25% of total
    //     const deposit2Amount = toTokenAmount(25);
    //     await confirmTransaction(
    //       await program.methods
    //         .deposit(deposit2Amount)
    //         .accounts({
    //           user: depositor2.publicKey,
    //           manager: manager.publicKey,
    //           depositTokenMint,
    //         })
    //         .signers([depositor2])
    //         .rpc()
    //     );

    //     // Get balances from vault token accounts
    //     const balance1 = await provider.connection.getTokenAccountBalance(
    //       depositor1VaultTokenATA
    //     );
    //     const balance2 = await provider.connection.getTokenAccountBalance(
    //       depositor2VaultTokenATA
    //     );

    //     // Get the vault token mint supply
    //     const mintInfo = await provider.connection.getTokenSupply(
    //       vaultTokenMint
    //     );
    //     const totalSupply = Number(mintInfo.value.amount);

    //     // Calculate shares
    //     const share1 = Number(balance1.value.amount) / totalSupply;
    //     const share2 = Number(balance2.value.amount) / totalSupply;

    //     // Verify shares (using approximate equality due to floating point)
    //     expect(share1).to.be.approximately(0.75, 0.0001);
    //     expect(share2).to.be.approximately(0.25, 0.0001);
    //     expect(share1 + share2).to.be.approximately(1.0, 0.0001);
    //   });

    //   it("correctly tracks total deposits in vault", async () => {
    //     // Initial deposit from depositor1
    //     const deposit1Amount = toTokenAmount(5);
    //     await confirmTransaction(
    //       await program.methods
    //         .deposit(deposit1Amount)
    //         .accounts({
    //           user: depositor1.publicKey,
    //           manager: manager.publicKey,
    //           depositTokenMint,
    //         })
    //         .signers([depositor1])
    //         .rpc()
    //     );

    //     // Check vault total deposits after first deposit
    //     let vaultTokenBalance =
    //       await provider.connection.getTokenAccountBalance(
    //         depositor1VaultTokenATA
    //       );
    //     expect(vaultTokenBalance.value.amount).to.equal(
    //       deposit1Amount.toString()
    //     );

    //     // Second deposit from depositor2
    //     const deposit2Amount = toTokenAmount(3);
    //     await confirmTransaction(
    //       await program.methods
    //         .deposit(deposit2Amount)
    //         .accounts({
    //           user: depositor2.publicKey,
    //           manager: manager.publicKey,
    //           depositTokenMint,
    //         })
    //         .signers([depositor2])
    //         .rpc()
    //     );

    //     // Check vault total deposits after second deposit
    //     vaultTokenBalance = await provider.connection.getTokenAccountBalance(
    //       vaultDepositTokenATA
    //     );
    //     expect(vaultTokenBalance.value.amount).to.equal(
    //       deposit1Amount.add(deposit2Amount).toString()
    //     );
    //   });

    //   it("correctly updates total deposits after withdrawals", async () => {
    //     // Check initial total deposits
    //     let vaultTokenBalance =
    //       await provider.connection.getTokenAccountBalance(
    //         depositor1VaultTokenATA
    //       );
    //     const initialTotalDeposits = new anchor.BN(
    //       vaultTokenBalance.value.amount
    //     );

    //     // Initial deposit
    //     const depositAmount = toTokenAmount(10);
    //     await confirmTransaction(
    //       await program.methods
    //         .deposit(depositAmount)
    //         .accounts({
    //           user: depositor1.publicKey,
    //           manager: manager.publicKey,
    //           depositTokenMint,
    //         })
    //         .signers([depositor1])
    //         .rpc()
    //     );

    //     // Partial withdrawal
    //     const withdrawAmount = toTokenAmount(4);
    //     await confirmTransaction(
    //       await program.methods
    //         .withdraw(withdrawAmount)
    //         .accounts({
    //           user: depositor1.publicKey,
    //           manager: manager.publicKey,
    //           depositTokenMint,
    //         })
    //         .signers([depositor1])
    //         .rpc()
    //     );

    //     // Check vault total deposits after withdrawal
    //     vaultTokenBalance = await provider.connection.getTokenAccountBalance(
    //       depositor1VaultTokenATA
    //     );
    //     expect(vaultTokenBalance.value.amount).to.equal(
    //       depositAmount.sub(withdrawAmount).toString()
    //     );
    //   });

    //   it("correctly mints vault tokens", async () => {
    //     const depositAmount = toTokenAmount(10);
    //     await confirmTransaction(
    //       await program.methods
    //         .deposit(depositAmount)
    //         .accounts({
    //           user: depositor1.publicKey,
    //           manager: manager.publicKey,
    //           depositTokenMint,
    //         })
    //         .signers([depositor1])
    //         .rpc()
    //     );

    //     // Verify vault token mint supply
    //     const mintInfo = await provider.connection.getTokenSupply(
    //       vaultTokenMint
    //     );
    //     expect(mintInfo.value.amount).to.equal(depositAmount.toString());

    //     // Verify depositor1's vault token balance
    //     const vaultTokenBalance =
    //       await provider.connection.getTokenAccountBalance(
    //         depositor1VaultTokenATA
    //       );
    //     expect(vaultTokenBalance.value.amount).to.equal(
    //       depositAmount.toString()
    //     );
    //   });
    // });

    describe("deposit", () => {
      it("successfully deposits tokens from depositor1", async () => {
        // Deposit 1 token into the vault
        const depositAmount = toTokenAmount(1);
        await confirmTransaction(
          await program.methods
            .deposit(depositAmount)
            .accounts({
              user: depositor1.publicKey,
              manager: manager.publicKey,
              depositTokenMint,
            })
            .signers([depositor1])
            .rpc()
        );

        // Verify that vault token account has the correct balance
        let vaultDepositTokenBalance =
          await provider.connection.getTokenAccountBalance(
            vaultDepositTokenATA
          );
        expect(vaultDepositTokenBalance.value.amount).to.equal(
          depositAmount.toString()
        );

        // Verify that depositor1's vault token account has the correct balance
        let depositorVaultTokenBalance =
          await provider.connection.getTokenAccountBalance(
            depositor1VaultTokenATA
          );
        expect(depositorVaultTokenBalance.value.amount).to.equal(
          depositAmount.toString()
        );
      });

      it("successfully makes multiple deposits from same depositor", async () => {
        const depositAmount = toTokenAmount(0.5);

        // First deposit
        await confirmTransaction(
          await program.methods
            .deposit(depositAmount)
            .accounts({
              user: depositor2.publicKey,
              manager: manager.publicKey,
              depositTokenMint,
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
              manager: manager.publicKey,
              depositTokenMint,
            })
            .signers([depositor2])
            .rpc()
        );

        // Verify total deposits by checking vault token balance of depositor2
        const finalBalance = await provider.connection.getTokenAccountBalance(
          depositor2VaultTokenATA
        );
        const expectedBalance = depositAmount.add(depositAmount);
        expect(finalBalance.value.amount).to.equal(expectedBalance.toString());
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
              manager: manager.publicKey,
              depositTokenMint: wrongToken,
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
              manager: manager.publicKey,
              depositTokenMint,
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
              manager: manager.publicKey,
              depositTokenMint,
            })
            .signers([depositor1])
            .rpc()
        );

        // Wait for and verify the event
        const event = await eventPromise;
        expect(event.vault.toString()).to.equal(vault.toString());
        expect(event.user.toString()).to.equal(depositor1.publicKey.toString());
        expect(event.amount.toString()).to.equal(depositAmount.toString());
      });
    });

    describe("withdraw", () => {
      let depositor1VaultTokenBalance: anchor.BN;

      beforeEach(async () => {
        // Make initial deposit to test withdrawals
        await confirmTransaction(
          await program.methods
            .deposit(toTokenAmount(10))
            .accounts({
              user: depositor1.publicKey,
              manager: manager.publicKey,
              depositTokenMint,
            })
            .signers([depositor1])
            .rpc()
        );

        depositor1VaultTokenBalance = new anchor.BN(
          (
            await provider.connection.getTokenAccountBalance(
              depositor1VaultTokenATA
            )
          ).value.amount
        );
      });

      it("successfully withdraws tokens", async () => {
        const withdrawAmount = toTokenAmount(3);

        const initialUserBalance =
          await provider.connection.getTokenAccountBalance(
            depositor1DepositTokenATA
          );

        await confirmTransaction(
          await program.methods
            .withdraw(withdrawAmount)
            .accounts({
              user: depositor1.publicKey,
              manager: manager.publicKey,
              depositTokenMint,
            })
            .signers([depositor1])
            .rpc()
        );

        // Verify token transfer to user's deposit token account
        const finalUserBalance =
          await provider.connection.getTokenAccountBalance(
            depositor1DepositTokenATA
          );
        expect(
          Number(finalUserBalance.value.amount) -
            Number(initialUserBalance.value.amount)
        ).to.equal(withdrawAmount.toNumber());

        // Verify vault token balance update
        const finalVaultTokenBalance =
          await provider.connection.getTokenAccountBalance(
            depositor1VaultTokenATA
          );
        expect(finalVaultTokenBalance.value.amount).to.equal(
          depositor1VaultTokenBalance.sub(withdrawAmount).toString()
        );
      });

      it("burns vault tokens when fully withdrawn", async () => {
        // Withdraw full amount
        await confirmTransaction(
          await program.methods
            .withdraw(depositor1VaultTokenBalance)
            .accounts({
              user: depositor1.publicKey,
              manager: manager.publicKey,
              depositTokenMint,
            })
            .signers([depositor1])
            .rpc()
        );

        // Verify vault token balance is zero
        const finalDepositor1VaultTokenBalance =
          await provider.connection.getTokenAccountBalance(
            depositor1VaultTokenATA
          );
        expect(finalDepositor1VaultTokenBalance.value.amount).to.equal("0");
      });

      it("fails when trying to withdraw more than deposited", async () => {
        const tooMuchAmount = depositor1VaultTokenBalance.add(new anchor.BN(1));

        try {
          await confirmTransaction(
            await program.methods
              .withdraw(tooMuchAmount)
              .accounts({
                user: depositor1.publicKey,
                manager: manager.publicKey,
                depositTokenMint,
              })
              .signers([depositor1])
              .rpc()
          );
          expect.fail("Should have failed with insufficient funds");
        } catch (error) {
          expect(error.toString()).to.include("0x1");
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
              manager: manager.publicKey,
              depositTokenMint,
            })
            .signers([depositor1])
            .rpc()
        );

        const event = await eventPromise;
        expect(event.vault.toString()).to.equal(vault.toString());
        expect(event.user.toString()).to.equal(depositor1.publicKey.toString());
        expect(event.amount.toString()).to.equal(withdrawAmount.toString());
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
              manager: manager.publicKey,
              depositTokenMint: wrongToken,
            })
            .signers([depositor1])
            .rpc();
          expect.fail("Should have failed with invalid token account");
        } catch (error) {
          expect(error.toString()).to.include("AccountNotInitialized");
        }
      });
    });
  });
});
