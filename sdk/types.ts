import { PublicKey } from "@solana/web3.js";

export interface VaultConfig {
    name: string;
    entryFee: number;
    exitFee: number;
}

export interface TokenConfig {
    mint: PublicKey;
}

export interface WithdrawParams {
    amount: number;
    vault: PublicKey;
}

export interface DepositParams {
    amount: number;
    vault: PublicKey;
}

export interface UpdateFeesParams {
    vault: PublicKey;
    newEntryFee: number;
    newExitFee: number;
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
