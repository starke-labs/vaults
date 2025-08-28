import { VaultsSdk } from "@starke/sdk";
import { USDC } from "@starke/sdk/whitelist";

import { createConnection, getManagerKeypair } from "./utils.new";

const manager = getManagerKeypair();

const RPC_URL =
  "https://mainnet.helius-rpc.com/?api-key=722d0f89-704b-4963-a20f-cdddda7c8798";

const vaults = new VaultsSdk(createConnection(RPC_URL), manager);

(async () => {
  // Write your script here

  // Deposit USDC to the vault

  // Get the vault
  const vault = await vaults.fetchVault(manager.publicKey);
  console.log(vault);

  // Get the vault's vtoken mint
  const vTokenMint = vault.mint;
  console.log(vTokenMint);

  // Create a vault
  // const vault = await vaults.createVault(
  //   "test vault",
  //   "TV",
  //   "https://test.com",
  //   manager.publicKey,
  //   USDC.mint,
  //   false,
  //   null,
  //   true,
  //   true,
  //   true,
  //   true
  // );
})();
