import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { getManager2Keypair, getManagerKeypair } from "tests/utils.new";
import { createConnection, getAuthorityKeypair } from "tests/utils.new";

import { VaultsSdk } from "@starke/sdk";
import {
  StarkeAlreadyPausedError,
  StarkeAlreadyResumedError,
  StarkePausedError,
} from "@starke/sdk/lib/errors";
import { USDC } from "@starke/sdk/whitelist";

describe("Pause Starke", () => {
  const connection = createConnection();
  const authority = getAuthorityKeypair();
  const manager2 = getManager2Keypair();

  const vaults = new VaultsSdk(connection, authority);
  const manager2Vaults = new VaultsSdk(connection, manager2);

  const nonWhitelistedToken = new PublicKey(
    "EPCz5LK372vmvCkZH3HgSuGNKACJJwwxsofW6fypCPZL"
  );

  it("should be able to pause starke", async () => {
    const starkeConfig = await vaults.fetchStarkeConfig();
    expect(starkeConfig.isPaused).to.be.false;

    await vaults.pauseStarke();

    const starkeConfig2 = await vaults.fetchStarkeConfig();
    expect(starkeConfig2.isPaused).to.be.true;
  });

  it("should not be able to pause starke if it is already paused", async () => {
    try {
      await vaults.pauseStarke();
      expect.fail(
        "Should not be able to pause the starke if it is already paused"
      );
    } catch (error) {
      expect(error).instanceOf(StarkeAlreadyPausedError);
    }
  });

  it("should be able to add a token to whitelist even if starke is paused", async () => {
    await vaults.addTokenToWhitelist({
      mint: nonWhitelistedToken,
      priceFeedId: "1",
      priceUpdate: Keypair.generate().publicKey,
    });
  });

  it("should be able to add a manager to whitelist even if starke is paused", async () => {
    await vaults.addManagerToWhitelist(manager2.publicKey);
  });

  it("should not be able to add a vault if starke is paused", async () => {
    try {
      await manager2Vaults.createVault(
        "test",
        "test",
        "test",
        manager2.publicKey,
        nonWhitelistedToken,
        false,
        null,
        true,
        true,
        true,
        true,
        1000000,
        10000000,
        0,
        [manager2]
      );
      expect.fail("Should not be able to add a vault if the starke is paused");
    } catch (error) {
      expect(error).instanceOf(StarkePausedError);
    }
  });

  // TODO: Add test for deposit, withdraw and swap

  it("should be able to remove a manager from whitelist even if starke is paused", async () => {
    await vaults.removeManagerFromWhitelist(manager2.publicKey);
  });

  it("should be able to remove a token from whitelist even if starke is paused", async () => {
    await vaults.removeTokenFromWhitelist(nonWhitelistedToken);
  });

  it("should be able to resume starke", async () => {
    await vaults.resumeStarke();

    const starkeConfig2 = await vaults.fetchStarkeConfig();
    expect(starkeConfig2.isPaused).to.be.false;
  });

  it("should not be able to resume starke if it is not paused", async () => {
    try {
      await vaults.resumeStarke();
      expect.fail(
        "Should not be able to resume the starke if it is not paused"
      );
    } catch (error) {
      expect(error).instanceOf(StarkeAlreadyResumedError);
    }
  });
});
