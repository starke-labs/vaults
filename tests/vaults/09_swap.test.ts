import { BN } from "@coral-xyz/anchor";
import { Connection, Keypair } from "@solana/web3.js";

import { VaultsSdk } from "@starke/sdk";
import { USDC, USDT } from "@starke/sdk/whitelist";

import { createConnection, getManagerKeypair } from "../utils.new";

describe("Swap", () => {
  let connection: Connection;
  let vaults: VaultsSdk;
  let manager: Keypair;

  before(async () => {
    connection = createConnection();
    manager = getManagerKeypair();
    vaults = new VaultsSdk(connection, manager);
  });

  it("should test something", async () => {
    console.log(manager.publicKey.toBase58());

    const tx = await vaults.swapOnJupiter(
      USDC.mint,
      USDT.mint,
      new BN(1 * 10 ** 6),
      manager.publicKey
    );
    console.log(tx);
  });
});
