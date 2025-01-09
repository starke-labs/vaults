import { AnchorProvider } from "@coral-xyz/anchor";
import { Keypair, Transaction } from "@solana/web3.js";

/**
 * Sleep for the specified number of milliseconds
 */
export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Configuration options for transaction retry mechanism
 */
export interface TransactionRetryConfig {
  /** Maximum number of retry attempts */
  maxRetries?: number;
  /** Base delay between retries in milliseconds */
  baseDelayMs?: number;
  /** Maximum delay between retries in milliseconds */
  maxDelayMs?: number;
  /** Whether to skip preflight */
  skipPreflight?: boolean;
  /** Commitment level for preflight */
  preflightCommitment?: "processed" | "confirmed" | "finalized";
  /** Commitment level for confirmation */
  commitment?: "processed" | "confirmed" | "finalized";
}

/**
 * Default configuration for transaction retries
 */
export const DEFAULT_RETRY_CONFIG: Required<TransactionRetryConfig> = {
  maxRetries: 3,
  baseDelayMs: 1000,
  maxDelayMs: 10000,
  skipPreflight: false,
  preflightCommitment: "confirmed",
  commitment: "confirmed",
};

/**
 * Send and confirm a transaction with automatic retries and exponential backoff
 */
export async function sendAndConfirmWithRetry(
  provider: AnchorProvider,
  tx: Transaction,
  signers: Keypair[],
  config: TransactionRetryConfig = {}
): Promise<string> {
  const finalConfig = { ...DEFAULT_RETRY_CONFIG, ...config };
  let lastError;

  for (let attempt = 1; attempt <= finalConfig.maxRetries; attempt++) {
    try {
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

      // console.log(`Transaction successful on attempt ${attempt}: ${signature}`);
      return signature;
    } catch (error) {
      lastError = error;
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
        // console.log(`Retrying in ${delay}ms...`);
        await sleep(delay);
      }
    }
  }

  throw new Error(
    `Transaction failed after ${finalConfig.maxRetries} attempts. Last error: ${lastError}`
  );
}
