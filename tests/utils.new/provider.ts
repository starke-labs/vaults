import { AnchorProvider, Wallet } from "@coral-xyz/anchor";
import { Connection, Keypair } from "@solana/web3.js";

import { DEFAULT_COMMITMENT, DEFAULT_TIMEOUT } from "./constants";

export function createConnection(
  endpoint: string = process.env.ANCHOR_PROVIDER_URL!
): Connection {
  return new Connection(endpoint, {
    commitment: DEFAULT_COMMITMENT,
    confirmTransactionInitialTimeout: DEFAULT_TIMEOUT,
  });
}

export function getProvider(keypair: Keypair): AnchorProvider {
  return new AnchorProvider(
    createConnection(),
    new Wallet(keypair),
    AnchorProvider.defaultOptions()
  );
}
