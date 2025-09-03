import {
  ConnectionResult,
  IBackpackSolanaSigner,
  ISolanaEvents,
  ISolanaSigner,
  SignedMessage,
} from "@dynamic-labs/solana-core";
import {
  Connection,
  Keypair,
  PublicKey,
  SendOptions,
  Signer,
  Transaction,
  TransactionSignature,
  VersionedTransaction,
} from "@solana/web3.js";
import EventEmitter from "eventemitter3";

/**
 * Adapter that wraps a standard Keypair to implement the Dynamic.xyz ISolanaSigner interface
 * This allows existing Keypair-based code to work with Dynamic.xyz wallet interface
 */
export class KeypairSignerAdapter
  extends EventEmitter<ISolanaEvents>
  implements ISolanaSigner
{
  protected keypair: Keypair;
  protected connection: Connection;
  private _isConnected: boolean = false;

  // ExtensionLocators properties - all false for Keypair adapter
  isBraveWallet = false;
  isGlow = false;
  isPhantom = false;
  isSolflare = false;
  isExodus = false;
  isBackpack = false;
  isMagicEden = false;

  constructor(keypair: Keypair, connection: Connection) {
    super();
    this.keypair = keypair;
    this.connection = connection;
  }

  get publicKey() {
    return {
      toBytes: () => this.keypair.publicKey.toBytes(),
    };
  }

  get isConnected(): boolean {
    return this._isConnected;
  }

  get providers(): ISolanaSigner[] {
    return [this]; // Self-reference for compatibility
  }

  async connect(args?: { onlyIfTrusted: boolean }): Promise<ConnectionResult> {
    this._isConnected = true;
    const result = {
      publicKey: { toString: () => this.keypair.publicKey.toString() },
    };
    this.emit("connect", this.keypair.publicKey.toString());
    return result;
  }

  async disconnect(): Promise<void> {
    this._isConnected = false;
    this.emit("disconnect");
  }

  async signTransaction<T extends Transaction | VersionedTransaction>(
    transaction: T
  ): Promise<T> {
    if (!this._isConnected) {
      throw new Error("Wallet not connected");
    }

    if (transaction instanceof VersionedTransaction) {
      transaction.sign([this.keypair]);
    } else {
      // Cast to Transaction for proper typing
      (transaction as Transaction).sign(this.keypair);
    }

    return transaction;
  }

  async signAllTransactions<T extends Transaction | VersionedTransaction>(
    transactions: T[]
  ): Promise<T[]> {
    if (!this._isConnected) {
      throw new Error("Wallet not connected");
    }

    return Promise.all(transactions.map((tx) => this.signTransaction(tx)));
  }

  async signAndSendTransaction<T extends Transaction | VersionedTransaction>(
    transaction: T,
    options?: SendOptions
  ): Promise<{ signature: TransactionSignature }> {
    if (!this._isConnected) {
      throw new Error("Wallet not connected");
    }

    const signedTx = await this.signTransaction(transaction);

    let signature: TransactionSignature;
    if (signedTx instanceof VersionedTransaction) {
      signature = await this.connection.sendTransaction(signedTx, options);
    } else {
      signature = await this.connection.sendTransaction(signedTx, [], options);
    }

    // Wait for confirmation
    await this.connection.confirmTransaction(
      signature,
      options?.preflightCommitment || "confirmed"
    );

    return { signature };
  }

  async signMessage(
    message: Uint8Array,
    encoding?: string
  ): Promise<SignedMessage> {
    if (!this._isConnected) {
      throw new Error("Wallet not connected");
    }

    // Import tweetnacl dynamically to avoid type issues
    const { sign } = await import("tweetnacl");
    const signature = sign.detached(message, this.keypair.secretKey);

    return {
      signature,
    };
  }
}

/**
 * Extended adapter that also implements IBackpackSolanaSigner interface
 */
export class KeypairBackpackSignerAdapter
  extends KeypairSignerAdapter
  implements IBackpackSolanaSigner
{
  async send(
    transaction: Transaction,
    signers?: Signer[],
    options?: SendOptions,
    connection?: Connection,
    publicKey?: PublicKey
  ): Promise<TransactionSignature> {
    const conn = connection || this.connection;
    const allSigners = [this.keypair, ...(signers || [])];

    // Sign the transaction
    transaction.sign(...allSigners);

    // Send and confirm
    const signature = await conn.sendTransaction(transaction, [], options);
    await conn.confirmTransaction(
      signature,
      options?.preflightCommitment || "confirmed"
    );

    return signature;
  }
}

/**
 * Utility function to create the appropriate signer adapter based on input type
 */
export function createSolanaSigner(
  signerOrKeypair: Keypair | ISolanaSigner,
  connection?: Connection,
  supportsBackpack: boolean = false
): ISolanaSigner | IBackpackSolanaSigner {
  // If it's already a signer interface, return as-is
  if ("signTransaction" in signerOrKeypair && "connect" in signerOrKeypair) {
    return signerOrKeypair as ISolanaSigner;
  }

  // If it's a Keypair, wrap it in an adapter
  if (signerOrKeypair instanceof Keypair) {
    if (!connection) {
      throw new Error("Connection required when using Keypair");
    }

    return supportsBackpack
      ? new KeypairBackpackSignerAdapter(signerOrKeypair, connection)
      : new KeypairSignerAdapter(signerOrKeypair, connection);
  }

  throw new Error("Invalid signer type provided");
}

/**
 * Type guard to check if a signer supports Backpack interface
 */
export function isBackpackSigner(
  signer: ISolanaSigner
): signer is IBackpackSolanaSigner {
  return "send" in signer && typeof (signer as any).send === "function";
}

/**
 * Type guard to check if input is a Keypair
 */
export function isKeypair(signer: any): signer is Keypair {
  return signer instanceof Keypair;
}

/**
 * Type guard to check if input implements ISolanaSigner
 */
export function isSolanaSigner(signer: any): signer is ISolanaSigner {
  return (
    signer &&
    typeof signer.signTransaction === "function" &&
    typeof signer.connect === "function" &&
    typeof signer.disconnect === "function"
  );
}
