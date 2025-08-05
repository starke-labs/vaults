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

export interface TokenWhitelist {
  tokens: Token[];
  authority: PublicKey;
  bump: number;
}

export interface ManagerWhitelist {
  managers: PublicKey[];
  bump: number;
}

export interface StarkeConfig {
  isPaused: boolean;
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
  maxAllowedAum: bigint | null;
  allowRetail: boolean;
  allowAccredited: boolean;
  allowInstitutional: boolean;
  allowQualified: boolean;
  individualMinDeposit: number; // u32, 0 = no minimum
  institutionalMinDeposit: number; // u32, 0 = no minimum
  maxDepositors: number; // u32, 0 = unlimited
  currentDepositors: number;
}

export const InvestorType = {
  Retail: { retail: {} },
  Accredited: { accredited: {} },
  Institutional: { institutional: {} },
  Qualified: { qualified: {} }
} as const;

export type InvestorType = typeof InvestorType[keyof typeof InvestorType];

export interface UserEntry {
  user: PublicKey;
  investorType: InvestorType;
}

export interface UserWhitelist {
  authority: PublicKey;
  bump: number;
  users: UserEntry[];
}

export interface VaultConfig {
  key: PublicKey;
  manager: PublicKey;
  vtokenMint: PublicKey;
  vtokenIsTransferrable: boolean;
  bump: number;
}
