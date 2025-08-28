import { Connection, PublicKey, SendOptions, Signer, Transaction, TransactionSignature, VersionedTransaction } from "@solana/web3.js";
import { EventEmitter, ISolanaEvents } from "./wallet-events";

// Dynamic.xyz compatible types
export type ExtensionLocators = {
  // Add any extension-specific locator properties if needed
};

export type ConnectionResult = {
  publicKey: PublicKey;
};

export type SignedMessage = {
  signature: Uint8Array;
  message: Uint8Array;
};

// Core Solana Signer interface from Dynamic.xyz
export interface ISolanaSigner extends EventEmitter<ISolanaEvents> {
  publicKey?: {
    toBytes(): Uint8Array;
  };
  isConnected: boolean;
  providers: ISolanaSigner[];
  
  signTransaction<T extends Transaction | VersionedTransaction>(transaction: T): Promise<T>;
  signAllTransactions<T extends Transaction | VersionedTransaction>(transactions: T[]): Promise<T[]>;
  signAndSendTransaction<T extends Transaction | VersionedTransaction>(
    transaction: T, 
    options?: SendOptions
  ): Promise<{ signature: TransactionSignature }>;
  signMessage(message: Uint8Array, encoding?: string): Promise<SignedMessage>;
  
  connect(args?: { onlyIfTrusted: boolean }): Promise<ConnectionResult>;
  disconnect(): Promise<void>;
}

// Backpack-specific extension
export interface IBackpackSolanaSigner extends ISolanaSigner {
  send: (
    transaction: Transaction,
    signers?: Signer[],
    options?: SendOptions,
    connection?: Connection,
    publicKey?: PublicKey
  ) => Promise<TransactionSignature>;
}

// Union type for all supported signers
export type ISolana = ISolanaSigner | IBackpackSolanaSigner;