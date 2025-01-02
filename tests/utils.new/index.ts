import { BN } from "@coral-xyz/anchor";
import { Connection, PublicKey, TransactionSignature } from "@solana/web3.js";

export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function confirmTransaction(
  connection: Connection,
  signature: TransactionSignature
): Promise<void> {
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature,
    ...latestBlockhash,
  });
}

export async function requestAirdrop(
  connection: Connection,
  publicKey: PublicKey,
  lamports: number = 100 * 10 ** 9
): Promise<void> {
  const signature = await connection.requestAirdrop(publicKey, lamports);
  await confirmTransaction(connection, signature);
}

export function toTokenAmount(amount: number, decimals: number = 6): BN {
  return new BN(amount * 10 ** decimals);
}
