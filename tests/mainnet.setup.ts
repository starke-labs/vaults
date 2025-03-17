import { AnchorProvider, BN, Idl, Wallet } from "@coral-xyz/anchor";
import { HermesClient } from "@pythnetwork/hermes-client";
import {
  InstructionWithEphemeralSigners,
  PythSolanaReceiver,
} from "@pythnetwork/pyth-solana-receiver";
import { sendTransactions } from "@pythnetwork/solana-utils";
import {
  createAssociatedTokenAccount,
  createMint,
  getAssociatedTokenAddress,
  mintTo,
} from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { AddressLookupTableAccount } from "@solana/web3.js";
import { expect } from "chai";

import idl from "@starke/idl/vaults.json";
import { VaultsSDK } from "@starke/sdk";
import { getVaultPda, getVaultTokenMintPda } from "@starke/sdk/pdas";
import {
  AddTokenAccounts,
  AddTokenParams,
  CreateVaultAccounts,
  CreateVaultParams,
  DepositAccounts,
  DepositParams,
  SwapOnJupiterAccounts,
  SwapOnJupiterParams,
  WithdrawAccounts,
  WithdrawParams,
} from "@starke/sdk/types";

import {
  getAuthorityKeypair,
  getProvider,
  getTesterKeypair,
  requestAirdropIfNecessary,
} from "./utils.new";
import { JUP_PRICE_FEED_ID, USDC_PRICE_FEED_ID } from "./utils.new/constants";

const SHARD_ID = 0;

describe("Setup Vaults", () => {
  let USDC = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
  const USDC_DECIMALS = 6;

  let JUP = new PublicKey("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN");
  const JUP_DECIMALS = 6;

  let authority: Keypair;
  let tester: Keypair;
  let provider: AnchorProvider;
  let sdk: VaultsSDK;
  let hermesClient: HermesClient;
  let pythSolReceiver: PythSolanaReceiver;
  let testerATA: PublicKey;

  before(async () => {
    // Setup provider and authority
    authority = getAuthorityKeypair();
    tester = getTesterKeypair();
    provider = getProvider(tester);

    // Initialize SDK
    sdk = new VaultsSDK(
      provider.connection,
      tester,
      new PublicKey(idl.address),
      idl as Idl
    );

    // Initialize price service
    hermesClient = new HermesClient("https://hermes.pyth.network", {});

    // NOTE: works with rpc-websockets@7.5.1, use `yarn add rpc-websockets@7.5.1 --exact`
    pythSolReceiver = new PythSolanaReceiver({
      connection: provider.connection,
      wallet: new Wallet(tester),
    });

    // Only required for Localnet
    if (provider.connection.rpcEndpoint.includes("localnet")) {
      await requestAirdropIfNecessary(provider.connection, tester.publicKey);
      USDC = await createMint(
        provider.connection,
        tester,
        authority.publicKey,
        null,
        USDC_DECIMALS
      );
      testerATA = await createAssociatedTokenAccount(
        provider.connection,
        tester,
        USDC,
        tester.publicKey
      );
      await mintTo(
        provider.connection,
        tester,
        USDC,
        testerATA,
        authority,
        1000 * 10 ** USDC_DECIMALS
      );
    }
  });

  // it("test", async () => {
  //   const priceIds = [
  //     // You can find the ids of prices at https://pyth.network/developers/price-feed-ids
  //     "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43", // BTC/USD price id
  //     "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace", // ETH/USD price id
  //   ];

  //   // // Get price feeds
  //   // // You can also fetch price feeds for other assets by specifying the asset name and asset class.
  //   // const priceFeeds = await connection.getPriceFeeds({
  //   //   query: "btc",
  //   //   filter: "crypto",
  //   // });
  //   // console.log(priceFeeds);

  //   // Latest price updates
  //   const response = await hermesClient.getLatestPriceUpdates(priceIds, {
  //     encoding: "base64",
  //   });
  //   let priceUpdateData = response.binary.data;
  //   console.log("priceUpdateData", priceUpdateData);

  //   const txBuilder = pythSolReceiver.newTransactionBuilder({
  //     closeUpdateAccounts: false,
  //   });

  //   // TODO: Move to constants
  //   const SHARD_ID = 0;
  //   txBuilder.addUpdatePriceFeed(priceUpdateData, SHARD_ID);

  //   const txs = await txBuilder.buildVersionedTransactions({});

  //   console.log("txs", txs);

  //   const signatures = await sendTransactions(
  //     txs,
  //     provider.connection,
  //     new Wallet(tester)
  //   );
  //   console.log(signatures);

  //   console.log(
  //     "priceUpdateAccount",
  //     priceIds[0],
  //     txBuilder.getPriceUpdateAccount(priceIds[0]).toString()
  //   );

  //   // const tx = await txBuilder.build();
  //   // const signature = await provider.sendAndConfirm(tx);
  //   // console.log(signature);
  // });

  it("should successfully initialize token whitelist or verify it is already initialized", async () => {
    try {
      const ix = await sdk.initializeWhitelist();
      const signature = await sdk.sendTransaction([ix], [authority]);
      expect(signature).to.not.be.empty;
    } catch (e) {
      expect(e.toString()).to.have.string("already in use");
    }

    // Verify whitelist was initialized
    const whitelist = await sdk.fetchWhitelist();
    console.log("whitelist", whitelist);
    expect(whitelist.authority.toString()).to.equal(
      authority.publicKey.toString()
    );
    expect(whitelist.programAuthority.toString()).to.equal(
      authority.publicKey.toString()
    );
  });

  it("should successfully add USDC to whitelist or verify it is already added", async () => {
    const response = await hermesClient.getLatestPriceUpdates(
      [USDC_PRICE_FEED_ID],
      {
        encoding: "base64",
      }
    );
    let priceUpdateData = response.binary.data;

    const txBuilder = pythSolReceiver.newTransactionBuilder({
      closeUpdateAccounts: false,
    });
    txBuilder.addUpdatePriceFeed(priceUpdateData, SHARD_ID);

    const priceUpdate = txBuilder.getPriceUpdateAccount(USDC_PRICE_FEED_ID);

    const params: AddTokenParams = {
      priceFeedId: USDC_PRICE_FEED_ID,
      token: USDC,
    };

    const accounts: AddTokenAccounts = {
      authority: authority.publicKey,
    };

    txBuilder.addInstruction({
      instruction: await sdk.addToken(params, accounts),
      signers: [authority],
    });

    try {
      const txs = await txBuilder.buildVersionedTransactions({});
      const signatures = await sendTransactions(
        txs,
        provider.connection,
        new Wallet(tester)
      );
      expect(signatures).to.not.be.empty;
    } catch (e) {
      expect(e.toString()).to.have.string("TokenAlreadyWhitelisted");
    }

  //   // Verify token was added
  //   const whitelist = await sdk.fetchWhitelist();
  //   expect(whitelist.tokens[0].mint.toString()).to.equal(USDC.toString());
  //   expect(whitelist.tokens[0].priceFeedId).to.equal(USDC_PRICE_FEED_ID);
  //   expect(whitelist.tokens[0].priceUpdate.toString()).to.equal(
  //     priceUpdate.toString()
  //   );
  // });

  // it("should successfully add JUP to whitelist or verify it is already added", async () => {
  //   const params: AddTokenParams = {
  //     token: JUP,
  //     priceFeedId: JUP_PRICE_FEED_ID,
  //   };

  //   const accounts: AddTokenAccounts = {
  //     authority: authority.publicKey,
  //   };

  //   try {
  //     const ix = await sdk.addToken(params, accounts);
  //     const signature = await sdk.sendTransaction([ix], [authority]);
  //     expect(signature).to.not.be.empty;
  //   } catch (e) {
  //     expect(e.toString()).to.have.string("TokenAlreadyWhitelisted");
  //   }

  //   // Verify token was added
  //   const whitelist = await sdk.fetchWhitelist();
  //   expect(whitelist.tokens[1].mint.toString()).to.equal(JUP.toString());
  //   expect(whitelist.tokens[1].priceFeedId).to.equal(JUP_PRICE_FEED_ID);
  // });

  // it("should successfully create vault and vault token mint or verify it is already created", async () => {
  //   const params: CreateVaultParams = {
  //     name: "Test Vault",
  //     entryFee: 100, // 1%
  //     exitFee: 200, // 2%
  //   };

  //   const accounts: CreateVaultAccounts = {
  //     manager: tester.publicKey,
  //     depositTokenMint: USDC,
  //   };

  //   try {
  //     const ix = await sdk.createVault(params, accounts);
  //     const signature = await sdk.sendTransaction([ix], [tester]);
  //     expect(signature).to.not.be.empty;
  //   } catch (e) {
  //     expect(e.toString()).to.have.string("already in use");
  //   }

  //   // Verify vault was created with correct parameters
  //   const vault = await sdk.fetchVault(tester.publicKey);
  //   expect(vault.manager.toString()).to.equal(tester.publicKey.toString());
  //   expect(vault.depositTokenMint.toString()).to.equal(USDC.toString());
  //   expect(vault.name).to.equal(params.name);
  //   expect(vault.entryFee).to.equal(params.entryFee);
  //   expect(vault.exitFee).to.equal(params.exitFee);

  //   // Verify vault token mint was created
  //   const [vaultPda] = getVaultPda(tester.publicKey);
  //   const [vaultTokenMintPda] = getVaultTokenMintPda(vaultPda);

  //   console.log("Vault:", vaultPda.toString());
  //   console.log("Vault token mint:", vaultTokenMintPda.toString());

  //   const vaultTokenMint = await provider.connection.getAccountInfo(
  //     vaultTokenMintPda
  //   );
  //   expect(vaultTokenMint).to.not.be.null;
  // });

  // it("should successfully deposit into vault", async () => {
  //   const params: DepositParams = {
  //     amount: new BN(2).mul(new BN(10).pow(new BN(USDC_DECIMALS))), // 2 USDC
  //   };
  //   const accounts: DepositAccounts = {
  //     user: tester.publicKey,
  //     manager: tester.publicKey,
  //     depositTokenMint: USDC,
  //   };

  //   // Get latest price updates for all tokens in whitelist
  //   const whitelist = await sdk.fetchWhitelist();
  //   const priceFeeds = whitelist.tokens.map((token) => token.priceFeedId);
  //   const priceUpdateData = (
  //     await hermesClient.getLatestPriceUpdates(priceFeeds, {
  //       encoding: "base64",
  //     })
  //   ).binary.data;

  //   const {
  //     postInstructions: postIxs,
  //     closeInstructions: closeIxs,
  //     priceFeedIdToPriceUpdateAccount,
  //   } = await pythSolReceiver.buildPostPriceUpdateInstructions(priceUpdateData);

  //   const depositIx: InstructionWithEphemeralSigners = {
  //     instruction: await sdk.deposit(
  //       params,
  //       accounts,
  //       (priceFeedId: string) => priceFeedIdToPriceUpdateAccount[priceFeedId]
  //     ),
  //     signers: [tester],
  //     computeUnits: 100000,
  //   };

  //   const transactions = await pythSolReceiver.batchIntoVersionedTransactions(
  //     [...postIxs, depositIx, ...closeIxs],
  //     {
  //       computeUnitPriceMicroLamports: 100000,
  //       tightComputeBudget: true,
  //     }
  //   );

  //   for (let tx of transactions) {
  //     console.log("Transaction:", tx);
  //     try {
  //       const signature = await pythSolReceiver.provider.sendAndConfirm(
  //         tx.tx,
  //         tx.signers,
  //         {
  //           // skipPreflight: true,
  //         }
  //       );
  //       console.log("Signature:", signature);
  //     } catch (e) {
  //       console.log("Error:", e);
  //     }
  //   }
  // });

  // it("should successfully swap vault funds on jupiter", async () => {
  //   const params: SwapOnJupiterParams = {
  //     amount: new BN(1).mul(new BN(10).pow(new BN(USDC_DECIMALS))),
  //   };

  //   const accounts: SwapOnJupiterAccounts = {
  //     manager: tester.publicKey,
  //     inputMint: USDC,
  //     outputMint: JUP,
  //   };

  //   try {
  //     const signature = await sdk.swapOnJupiter(params, accounts, [tester]);
  //     console.log("Signature:", signature);
  //     expect(signature).to.not.be.empty;
  //   } catch (e) {
  //     console.log("Error:", e);
  //     throw e;
  //   }
  // });

  // it("should successfully withdraw from vault", async () => {
  //   const [vaultPda] = getVaultPda(tester.publicKey);
  //   const testerVaultTokenAccount = await getAssociatedTokenAddress(
  //     getVaultTokenMintPda(vaultPda)[0],
  //     tester.publicKey
  //   );
  //   const balance = await provider.connection.getTokenAccountBalance(
  //     testerVaultTokenAccount
  //   );
  //   console.log("Balance:", balance.value.uiAmount, balance.value.amount);

  //   const params: WithdrawParams = {
  //     amount: new BN(balance.value.amount),
  //   };
  //   const accounts: WithdrawAccounts = {
  //     user: tester.publicKey,
  //     manager: tester.publicKey,
  //   };

  //   try {
  //     const ix = await sdk.withdraw(params, accounts);
  //     const signature = await sdk.sendTransaction([ix], [tester]);
  //     console.log("Signature:", signature);
  //     expect(signature).to.not.be.empty;
  //   } catch (e) {
  //     console.log("Error:", e);
  //   }
  // });
});
