import { AnchorProvider, BN, Wallet } from "@coral-xyz/anchor";
import {
  Commitment,
  Connection,
  Keypair,
  PublicKey,
  TransactionSignature,
} from "@solana/web3.js";
import fs from "fs";

import { DEFAULT_COMMITMENT, DEFAULT_TIMEOUT } from "./constants";

export function createConnection(
  endpoint: string = process.env.ANCHOR_PROVIDER_URL!
): Connection {
  return new Connection(endpoint, {
    commitment: DEFAULT_COMMITMENT,
    confirmTransactionInitialTimeout: DEFAULT_TIMEOUT,
  });
}

export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// export async function confirmTransaction(
//   connection: Connection,
//   signature: TransactionSignature
// ): Promise<void> {
//   const latestBlockhash = await connection.getLatestBlockhash();
//   await connection.confirmTransaction(
//     {
//       signature,
//       ...latestBlockhash,
//     },
//     connection.commitment || DEFAULT_COMMITMENT
//   );
// }

export function getProvider(keypair: Keypair): AnchorProvider {
  const provider = AnchorProvider.env();

  return new AnchorProvider(
    createConnection(),
    new Wallet(keypair),
    AnchorProvider.defaultOptions()
  );
}

export async function requestAirdrop(
  connection: Connection,
  publicKey: PublicKey,
  lamports: number = 10 * 10 ** 9
): Promise<void> {
  const signature = await connection.requestAirdrop(publicKey, lamports);
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature,
    ...latestBlockhash,
  });
}

export async function requestAirdropIfNecessary(
  connection: Connection,
  publicKey: PublicKey,
  lamports: number = 10 * 10 ** 9
): Promise<void> {
  const balance = await connection.getBalance(publicKey);
  if (balance < lamports) {
    await requestAirdrop(connection, publicKey, lamports);
  }
}

export function toTokenAmount(amount: number, decimals: number = 6): BN {
  return new BN(amount * 10 ** decimals);
}

export function getAuthorityKeypair(): Keypair {
  return Keypair.fromSecretKey(
    new Uint8Array(
      JSON.parse(fs.readFileSync("./deploy/authority.json", "utf8"))
    )
  );
}
