import { Keypair } from "@solana/web3.js";

import { VaultsSDK } from "@starke/sdk";

import { createConnection, getTesterKeypair } from "../utils.new";

describe("Create Vault", () => {
  let vaults: VaultsSDK;
  let manager: Keypair;

  before(async () => {
    // Get keypairs
    manager = getTesterKeypair();

    // Initialize SDK
    vaults = new VaultsSDK(createConnection(), manager);
  });

  it("should create a vault", async () => {
    console.log("Creating vault...");
  });
});
