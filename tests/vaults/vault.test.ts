import { AnchorProvider, Idl } from "@coral-xyz/anchor";
import { createMint } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import { getVaultPda, getVaultTokenMintPda } from "@starke/sdk/pdas";

import { VaultsSDK } from "../../sdk";
import { CreateVaultAccounts, CreateVaultParams } from "../../sdk/types";
import idl from "../../target/idl/vaults.json";
import {
  getAuthorityKeypair,
  getProvider,
  requestAirdropIfNecessary,
} from "../utils.new";
import {
  DEFAULT_MINT_DECIMALS,
  USDC_PRICE_FEED_ID,
} from "../utils.new/constants";

describe("Vault Tests", () => {
  let sdk: VaultsSDK;
  let provider: AnchorProvider;
  let tester: Keypair;
  let depositTokenMint: PublicKey;

  before(async () => {
    // Setup provider and tester
    tester = Keypair.generate();
    provider = getProvider(tester);

    // Request SOL for tester if needed
    await requestAirdropIfNecessary(provider.connection, tester.publicKey);

    // Initialize SDK
    sdk = new VaultsSDK(
      provider.connection,
      tester,
      new PublicKey(idl.address),
      idl as Idl
    );

    // Create a deposit token mint
    depositTokenMint = await createMint(
      provider.connection,
      tester,
      tester.publicKey,
      null,
      DEFAULT_MINT_DECIMALS
    );

    // Get authority keypair
    const authority = getAuthorityKeypair();

    // This test must be run after the whitelist test so that the whitelist is initialized
    // Add deposit token to whitelist
    const addTokenIx = await sdk.addToken(
      { token: depositTokenMint, priceFeedId: USDC_PRICE_FEED_ID },
      { authority: authority.publicKey }
    );
    await sdk.sendTransaction([addTokenIx], [authority]);
  });

  it("should not create vault with invalid params", async () => {
    const accounts: CreateVaultAccounts = {
      manager: tester.publicKey,
      depositTokenMint,
    };

    // Create vault with invalid name
    try {
      const ix = await sdk.createVault(
        {
          name: "",
          entryFee: 100,
          exitFee: 200,
        },
        accounts
      );
      await sdk.sendTransaction([ix], [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("NameTooShort");
    }

    // Create vault with invalid entry fee
    try {
      const ix = await sdk.createVault(
        {
          name: "Test Vault",
          entryFee: -1,
          exitFee: 200,
        },
        accounts
      );
      await sdk.sendTransaction([ix], [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("RangeError");
    }

    // Create vault with invalid exit fee
    try {
      const ix = await sdk.createVault(
        {
          name: "Test Vault",
          entryFee: 100,
          exitFee: -1,
        },
        accounts
      );
      await sdk.sendTransaction([ix], [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("RangeError");
    }

    // Create vault with invalid entry fee
    try {
      const ix = await sdk.createVault(
        {
          name: "Test Vault",
          entryFee: 10001,
          exitFee: 200,
        },
        accounts
      );
      await sdk.sendTransaction([ix], [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("InvalidFee");
    }

    // Create vault with invalid exit fee
    try {
      const ix = await sdk.createVault(
        {
          name: "Test Vault",
          entryFee: 100,
          exitFee: 10001,
        },
        accounts
      );
      await sdk.sendTransaction([ix], [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("InvalidFee");
    }

    // Verify vault was not created
    try {
      await sdk.fetchVault(tester.publicKey);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("Account does not exist");
    }
  });

  it("should not create vault with non-whitelisted deposit token", async () => {
    const newTokenMint = await createMint(
      provider.connection,
      tester,
      tester.publicKey,
      null,
      DEFAULT_MINT_DECIMALS
    );

    const params: CreateVaultParams = {
      name: "Test Vault",
      entryFee: 100, // 1%
      exitFee: 200, // 2%
    };

    const accounts: CreateVaultAccounts = {
      manager: tester.publicKey,
      depositTokenMint: newTokenMint, // Non-whitelisted
    };

    // Try to create vault
    try {
      const ix = await sdk.createVault(params, accounts);
      await sdk.sendTransaction([ix], [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("TokenNotWhitelisted");
    }
  });

  it("should successfully create vault and vault token mint", async () => {
    const params: CreateVaultParams = {
      name: "Test Vault",
      entryFee: 100, // 1%
      exitFee: 200, // 2%
    };

    const accounts: CreateVaultAccounts = {
      manager: tester.publicKey,
      depositTokenMint,
    };

    const ix = await sdk.createVault(params, accounts);
    const signature = await sdk.sendTransaction([ix], [tester]);
    expect(signature).to.not.be.empty;

    // Verify vault was created with correct parameters
    const vault = await sdk.fetchVault(tester.publicKey);
    expect(vault.manager.toString()).to.equal(tester.publicKey.toString());
    expect(vault.depositTokenMint.toString()).to.equal(
      depositTokenMint.toString()
    );
    expect(vault.name).to.equal(params.name);
    expect(vault.entryFee).to.equal(params.entryFee);
    expect(vault.exitFee).to.equal(params.exitFee);

    // Verify vault token mint was created
    const [vaultPda] = getVaultPda(tester.publicKey);
    const [vaultTokenMintPda] = getVaultTokenMintPda(vaultPda);
    const vaultTokenMint = await provider.connection.getAccountInfo(
      vaultTokenMintPda
    );
    expect(vaultTokenMint).to.not.be.null;
  });

  it("should not create multiple vaults for same manager", async () => {
    const existingVaultName = "Test Vault";

    const params: CreateVaultParams = {
      name: "Test Vault 1",
      entryFee: 100,
      exitFee: 200,
    };

    const accounts: CreateVaultAccounts = {
      manager: tester.publicKey,
      depositTokenMint,
    };

    // Try to create second vault
    try {
      const ix = await sdk.createVault(params, accounts);
      await sdk.sendTransaction([ix], [tester]);
      expect.fail("Should have thrown an error");
    } catch (e) {
      expect(e.toString()).to.have.string("already in use");
    }

    // Verify only one vault exists
    const vault = await sdk.fetchVault(tester.publicKey);
    expect(vault.name).to.equal(existingVaultName);
  });
});
