import { BN } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";

async function requestAirdrop(
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
