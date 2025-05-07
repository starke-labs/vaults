import { createMint } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
  getTesterKeypair,
  requestAirdropIfNecessary,
} from "tests/utils.new";

import { VaultsSdk } from "@starke/sdk";
import { VaultConfigNotInitializedError } from "@starke/sdk/lib/errors";
import { getVaultPda, getVtokenMintPda } from "@starke/sdk/lib/pdas";
import { TransferHookSdk } from "@starke/sdk/transferHook";

describe("Transfer Hook", () => {
  let tester: Keypair;
  let manager: Keypair;
  let authority: Keypair;

  let transferHook: TransferHookSdk;
  let vaults: VaultsSdk;
  let vaultsAuthority: VaultsSdk;

  let depositTokenMint: PublicKey;

  before(async () => {
    tester = getTesterKeypair();
    manager = getManagerKeypair();
    authority = getAuthorityKeypair();
    transferHook = new TransferHookSdk(createConnection(), tester);

    const connection = createConnection();
    vaults = new VaultsSdk(createConnection(), manager);
    vaultsAuthority = new VaultsSdk(createConnection(), authority);

    await requestAirdropIfNecessary(connection, manager.publicKey);
    await requestAirdropIfNecessary(connection, authority.publicKey);
    depositTokenMint = await createMint(
      connection,
      manager,
      manager.publicKey,
      manager.publicKey,
      6
    );
  });

  it("should not be able to fetch the vault config before it is initialized", async () => {
    const [vault] = getVaultPda(manager.publicKey);
    const [vtokenMint] = getVtokenMintPda(vault);

    try {
      const vaultConfig = await transferHook.fetchVaultConfig(vtokenMint);
    } catch (e) {
      expect(e).to.be.instanceOf(VaultConfigNotInitializedError);
    }
  });

  it("should create a whitelist and add the deposit token mint to the whitelist", async () => {
    const txHash = await vaultsAuthority.initializeStarke([]);
    expect(txHash).to.not.be.undefined;

    const txHash2 = await vaultsAuthority.addTokenToWhitelist({
      mint: depositTokenMint,
      priceFeedId: "dummy_price_feed_id",
      priceUpdate: Keypair.generate().publicKey,
    });
    expect(txHash2).to.not.be.undefined;
  });

  it("should create a vault", async () => {
    const txHash = await vaults.createVault(
      "Test Vault",
      "TEST",
      "https://test.com",
      // TODO: Hardcode entry/exit fee to 0 in the sdk method
      0,
      0,
      manager.publicKey,
      depositTokenMint
    );
    expect(txHash).to.not.be.undefined;

    const vault = await vaults.fetchVault(manager.publicKey);
    expect(vault.name).to.equal("Test Vault");
    expect(vault.depositTokenMint.toBase58()).to.equal(
      depositTokenMint.toBase58()
    );
  });
});
