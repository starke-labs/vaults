import { AnchorProvider } from "@coral-xyz/anchor";
import { Signer, TransactionSignature } from "@solana/web3.js";
import { Keypair, Transaction } from "@solana/web3.js";

export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export interface TransactionRetryConfig {
  maxRetries?: number;
  baseDelayMs?: number;
  maxDelayMs?: number;
  skipPreflight?: boolean;
  preflightCommitment?: "processed" | "confirmed" | "finalized";
  commitment?: "processed" | "confirmed" | "finalized";
}

export const DEFAULT_RETRY_CONFIG: Required<TransactionRetryConfig> = {
  maxRetries: 3,
  baseDelayMs: 1000,
  maxDelayMs: 10000,
  skipPreflight: false,
  preflightCommitment: "confirmed",
  commitment: "confirmed",
};

export async function sendAndConfirmWithRetry(
  provider: AnchorProvider,
  tx: Transaction,
  signers: (Keypair | Signer)[],
  config: TransactionRetryConfig = {}
): Promise<TransactionSignature> {
  const finalConfig = { ...DEFAULT_RETRY_CONFIG, ...config };
  let lastError: Error;

  for (let attempt = 1; attempt <= finalConfig.maxRetries; attempt++) {
    try {
      // TODO: Setup debug logging
      // DEBUG
      // console.log(`Transaction attempt ${attempt}/${finalConfig.maxRetries}`);

      // Get latest blockhash before each attempt
      const latestBlockhash = await provider.connection.getLatestBlockhash(
        finalConfig.commitment
      );
      tx.recentBlockhash = latestBlockhash.blockhash;

      // Sign and send transaction
      const signature = await provider.sendAndConfirm(tx, signers, {
        skipPreflight: finalConfig.skipPreflight,
        preflightCommitment: finalConfig.preflightCommitment,
        commitment: finalConfig.commitment,
        maxRetries: 3, // Internal retries for network issues
      });

      const SOLSCAN_TX_URL = "https://solscan.io/tx/";
      console.log(`Attempt ${attempt}: ${SOLSCAN_TX_URL}${signature}`);
      return signature;
    } catch (error) {
      lastError = error as Error;
      // DEBUG
      // console.error(
      //   `Attempt ${attempt} failed:`,
      //   error instanceof Error ? error.message : String(error)
      // );

      if (attempt < finalConfig.maxRetries) {
        // Calculate delay with exponential backoff
        const delay = Math.min(
          finalConfig.baseDelayMs * Math.pow(2, attempt - 1),
          finalConfig.maxDelayMs
        );
        await sleep(delay);
      }
    }
  }

  throw lastError;
}
