import { BN } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";

// Create Vault
export interface CreateVaultParams {
  name: string;
  entryFee: number;
  exitFee: number;
}

export interface CreateVaultAccounts {
  manager: PublicKey;
  depositTokenMint: PublicKey;
}

// Add Token
export interface AddTokenParams {
  token: PublicKey;
  priceFeedId: string;
}

export interface AddTokenAccounts {
  authority: PublicKey;
}

// Deposit
export interface DepositParams {
  amount: BN;
}

export interface DepositAccounts {
  user: PublicKey;
  manager: PublicKey;
  depositTokenMint: PublicKey;
}

// Withdraw
export interface WithdrawParams {
  amount: BN;
}

export interface WithdrawAccounts {
  user: PublicKey;
  manager: PublicKey;
  depositTokenMint: PublicKey;
}

// Update Fees
export interface UpdateFeesParams {
  newEntryFee: number;
  newExitFee: number;
}

export interface UpdateFeesAccounts {
  manager: PublicKey;
}

// Event Types
export interface VaultCreatedEvent {
  vault: PublicKey;
  manager: PublicKey;
  depositToken: PublicKey;
  vaultTokenMint: PublicKey;
  name: string;
  timestamp: number;
  entryFee: number;
  exitFee: number;
}

export interface VaultFeesUpdateRequestedEvent {
  vault: PublicKey;
  manager: PublicKey;
  pendingEntryFee: number;
  pendingExitFee: number;
  timestamp: number;
}

export interface VaultFeesUpdatedEvent {
  vault: PublicKey;
  manager: PublicKey;
  newEntryFee: number;
  newExitFee: number;
  timestamp: number;
}

export interface DepositMadeEvent {
  vault: PublicKey;
  user: PublicKey;
  amount: number;
  timestamp: number;
}

export interface WithdrawMadeEvent {
  vault: PublicKey;
  user: PublicKey;
  amount: number;
  timestamp: number;
}

export interface TokenWhitelistedEvent {
  token: PublicKey;
}
