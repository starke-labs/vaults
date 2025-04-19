import { PublicKey } from "@solana/web3.js";

export interface AccountMeta {
  pubkey: PublicKey;
  isWritable: boolean;
  isSigner: boolean;
}

export interface Token {
  mint: PublicKey;
  priceFeedId: string;
  priceUpdate: PublicKey;
}

export interface Whitelist {
  tokens: Token[];
  authority: PublicKey;
  bump: number;
}

export interface Vault {
  manager: PublicKey;
  depositTokenMint: PublicKey;
  name: string;
  bump: number;
  mint: PublicKey;
  mintBump: number;
  entryFee: number;
  exitFee: number;
  pendingEntryFee: number | null;
  pendingExitFee: number | null;
  feeUpdateTimestamp: number;
}

export interface VaultConfig {
  key: PublicKey;
  manager: PublicKey;
  vtokenMint: PublicKey;
  vtokenIsTransferrable: boolean;
  bump: number;
}
