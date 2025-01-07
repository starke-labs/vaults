import { AnchorProvider, Idl, Wallet, setProvider } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import fs from "fs";

import idl from "@starke/idl/vaults.json";
import { VaultsSDK } from "@starke/sdk";
import {
  AddTokenAccounts,
  AddTokenParams,
  CreateVaultAccounts,
  CreateVaultParams,
  DepositAccounts,
  DepositParams,
  UpdateFeesAccounts,
  UpdateFeesParams,
  WithdrawAccounts,
  WithdrawParams,
} from "@starke/sdk/types";

import { getAuthorityKeypair, getProvider, toTokenAmount } from "../utils.new";

describe("VaultsSDK Unit Tests", () => {
  const priceFeedId =
    "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";

  let sdk: VaultsSDK;
  let provider: AnchorProvider;
  let authority: Keypair;
  let manager: Keypair;
  let tester: Keypair;
  let depositTokenMint: PublicKey;
  let priceUpdate: PublicKey;

  before(() => {
    // Setup keypairs
    authority = getAuthorityKeypair();
    tester = Keypair.generate();
    manager = Keypair.generate();
    depositTokenMint = Keypair.generate().publicKey;
    // Use a mock priceUpdate account instead of actual Pyth priceUpdateV2 account
    priceUpdate = Keypair.generate().publicKey;

    // Setup provider
    provider = getProvider(tester);

    // Initialize SDK
    const programId = new PublicKey(idl.address);
    sdk = new VaultsSDK(provider.connection, tester, programId, idl as Idl);
  });

  describe("Instruction Generation", () => {
    describe("createVault", () => {
      it("should generate valid create vault instruction", async () => {
        const params: CreateVaultParams = {
          name: "Test Vault",
          entryFee: 100,
          exitFee: 200,
        };

        const accounts: CreateVaultAccounts = {
          manager: manager.publicKey,
          depositTokenMint: depositTokenMint,
        };

        const ix = await sdk.createVault(params, accounts);
        expect(ix).to.not.be.null;
        expect(ix.programId.toString()).to.equal(idl.address);
        expect(ix.keys.length).to.be.greaterThan(0);
      });
    });

    describe("initializeWhitelist", () => {
      it("should generate valid initialize whitelist instruction", async () => {
        const ix = await sdk.initializeWhitelist();

        expect(ix).to.not.be.null;
        expect(ix.programId.toString()).to.equal(idl.address);
        expect(ix.keys.length).to.be.greaterThan(0);
      });
    });

    describe("addToken", () => {
      it("should generate valid add token instruction", async () => {
        const params: AddTokenParams = {
          token: depositTokenMint,
          priceFeedId,
        };

        const accounts: AddTokenAccounts = {
          authority: authority.publicKey,
        };

        const ix = await sdk.addToken(params, accounts);
        expect(ix).to.not.be.null;
        expect(ix.programId.toString()).to.equal(idl.address);
        expect(ix.keys.length).to.be.greaterThan(0);
      });
    });

    describe("deposit", () => {
      it("should generate valid deposit instruction", async () => {
        const params: DepositParams = {
          amount: toTokenAmount(10),
        };

        const accounts: DepositAccounts = {
          user: tester.publicKey,
          manager: manager.publicKey,
          depositTokenMint,
          priceUpdate,
        };

        const ix = await sdk.deposit(params, accounts);
        expect(ix).to.not.be.null;
        expect(ix.programId.toString()).to.equal(idl.address);
        expect(ix.keys.length).to.be.greaterThan(0);
      });
    });

    describe("withdraw", () => {
      it("should generate valid withdraw instruction", async () => {
        const params: WithdrawParams = {
          amount: toTokenAmount(10),
        };

        const accounts: WithdrawAccounts = {
          user: tester.publicKey,
          manager: manager.publicKey,
          depositTokenMint,
          priceUpdate,
        };

        const ix = await sdk.withdraw(params, accounts);
        expect(ix).to.not.be.null;
        expect(ix.programId.toString()).to.equal(idl.address);
        expect(ix.keys.length).to.be.greaterThan(0);
      });
    });

    describe("updateVaultFees", () => {
      it("should generate valid update fees instruction", async () => {
        const params: UpdateFeesParams = {
          newEntryFee: 150,
          newExitFee: 250,
        };

        const accounts: UpdateFeesAccounts = {
          manager: manager.publicKey,
        };

        const ix = await sdk.updateVaultFees(params, accounts);
        expect(ix).to.not.be.null;
        expect(ix.programId.toString()).to.equal(idl.address);
        expect(ix.keys.length).to.be.greaterThan(0);
      });
    });
  });

  // TODO: Add tests for event handlers
  // describe("Event Handlers", () => {
  //   it("should properly initialize event handler", () => {
  //     expect(sdk.events).to.not.be.null;
  //     expect(typeof sdk.events.subscribeVaultCreated).to.equal("function");
  //     expect(typeof sdk.events.subscribeDepositMade).to.equal("function");
  //     expect(typeof sdk.events.subscribeWithdrawMade).to.equal("function");
  //     expect(typeof sdk.events.subscribeTokenWhitelisted).to.equal("function");
  //   });
  // });
});
