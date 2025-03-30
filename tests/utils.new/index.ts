import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

export * from "./provider";
export * from "./keypairs";
export * from "./airdrop";
export * from "./constants";

export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function toTokenAmount(amount: number, decimals: number = 6): BN {
  return new BN(amount * 10 ** decimals);
}

export interface WhitelistAccount {
  // Whitelist
  whitelist: PublicKey;
  whitelistBump: number;
}

export interface VaultAccount {
  // Vault
  vault: PublicKey;
  vaultBump: number;
  // Vtoken mint
  vtokenMint: PublicKey;
  vtokenMintBump: number;
  // Deposit token mint
  depositTokenMint: PublicKey;
}
