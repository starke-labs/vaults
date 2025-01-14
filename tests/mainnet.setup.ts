import { AnchorProvider, BN, Idl, Wallet } from "@coral-xyz/anchor";
import { PriceServiceConnection } from "@pythnetwork/price-service-client";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
} from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import idl from "@starke/idl/vaults.json";
import { VaultsSDK } from "@starke/sdk";
import { getVaultPda, getVaultTokenMintPda } from "@starke/sdk/pdas";
import {
  AddTokenAccounts,
  AddTokenParams,
  CreateVaultAccounts,
  CreateVaultParams,
  DepositParams,
} from "@starke/sdk/types";

import {
  getAuthorityKeypair,
  getProvider,
  getTesterKeypair,
  requestAirdropIfNecessary,
} from "./utils.new";

describe("Setup Vaults", () => {
  let USDC = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
  const USDC_USD_PYTH_FEED_ID =
    "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";
  const USDC_DECIMALS = 6;

  let authority: Keypair;
  let tester: Keypair;
  let provider: AnchorProvider;
  let sdk: VaultsSDK;
  let priceService: PriceServiceConnection;
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
    priceService = new PriceServiceConnection("https://hermes.pyth.network", {
      priceFeedRequestConfig: {
        binary: true,
      },
    });

    // NOTE: works with rpc-websockets@7.5.1, use `yarn add rpc-websockets@7.5.1 --exact`
    pythSolReceiver = new PythSolanaReceiver({
      connection: provider.connection,
      wallet: new Wallet(tester),
    });

    // Only required for Localnet, comment out for mainnet
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
  });

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
    expect(whitelist.authority.toString()).to.equal(
      authority.publicKey.toString()
    );
    expect(whitelist.programAuthority.toString()).to.equal(
      authority.publicKey.toString()
    );
  });

  it("should successfully add token to whitelist or verify it is already added", async () => {
    const params: AddTokenParams = {
      token: USDC,
      priceFeedId: USDC_USD_PYTH_FEED_ID,
    };

    const accounts: AddTokenAccounts = {
      authority: authority.publicKey,
    };

    try {
      const ix = await sdk.addToken(params, accounts);
      const signature = await sdk.sendTransaction([ix], [authority]);
      expect(signature).to.not.be.empty;
    } catch (e) {
      expect(e.toString()).to.have.string("TokenAlreadyWhitelisted");
    }

    // Verify token was added
    const whitelist = await sdk.fetchWhitelist();
    expect(whitelist.tokens[0].mint.toString()).to.equal(USDC.toString());
    expect(whitelist.tokens[0].priceFeedId).to.equal(USDC_USD_PYTH_FEED_ID);
  });

  it("should successfully create vault and vault token mint or verify it is already created", async () => {
    const params: CreateVaultParams = {
      name: "Test Vault",
      entryFee: 100, // 1%
      exitFee: 200, // 2%
    };

    const accounts: CreateVaultAccounts = {
      manager: tester.publicKey,
      depositTokenMint: USDC,
    };

    // Get latest price updates for all tokens in whitelist
    const whitelist = await sdk.fetchWhitelist();
    const priceFeeds = whitelist.tokens.map((token) => token.priceFeedId);
    const priceUpdateData = (
      await hermesClient.getLatestPriceUpdates(priceFeeds, {
        encoding: "base64",
      })
    ).binary.data;

    const {
      postInstructions: postIxs,
      closeInstructions: closeIxs,
      priceFeedIdToPriceUpdateAccount,
    } = await pythSolReceiver.buildPostPriceUpdateInstructions(priceUpdateData);

    const depositIx: InstructionWithEphemeralSigners = {
      instruction: await sdk.deposit(
        params,
        accounts,
        (priceFeedId: string) => priceFeedIdToPriceUpdateAccount[priceFeedId]
      ),
      signers: [tester],
      computeUnits: 100000,
    };

    const transactions = await pythSolReceiver.batchIntoVersionedTransactions(
      [...postIxs, depositIx, ...closeIxs],
      {
        computeUnitPriceMicroLamports: 100000,
        tightComputeBudget: true,
      }
    );

    for (let tx of transactions) {
      console.log("Transaction:", tx);
      try {
        const signature = await pythSolReceiver.provider.sendAndConfirm(
          tx.tx,
          tx.signers,
          {
            // skipPreflight: true,
          }
        );
        console.log("Signature:", signature);
      } catch (e) {
        console.log("Error:", e);
      }
    }
  });
});
