import { VaultsSdk } from "@starke/sdk";
import { USDC } from "@starke/sdk/whitelist";

import {
  createConnection,
  getAuthorityKeypair,
  getManagerKeypair,
  getTesterKeypair,
  toTokenAmount,
} from "./utils.new";

const manager = getManagerKeypair();
const tester = getTesterKeypair();
const authority = getAuthorityKeypair();

const RPC_URL =
  "https://mainnet.helius-rpc.com/?api-key=722d0f89-704b-4963-a20f-cdddda7c8798";

const vaults = {
  manager: new VaultsSdk(createConnection(RPC_URL), manager),
  tester: new VaultsSdk(createConnection(RPC_URL), tester),
  authority: new VaultsSdk(createConnection(RPC_URL), authority),
};

(async () => {
  // Write your script here
  // // Get the vault
  // const vault = await vaults.manager.fetchVault(manager.publicKey);
  // console.log(vault);
  // // Get the vault's vtoken mint
  // const vTokenMint = vault.mint;
  // console.log(vTokenMint);
  // // Create a vault
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

  // Deposit USDC to the vault
  const deposit = await vaults.tester.deposit(
    toTokenAmount(1),
    tester.publicKey,
    manager.publicKey,
    [authority]
  );
  console.log(deposit);
})();
