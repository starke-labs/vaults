import {
  ConnectionResult,
  ISolanaEvents,
  ISolanaSigner,
  SignedMessage,
} from "@dynamic-labs/solana-core";
import { PublicKey } from "@solana/web3.js";
import EventEmitter from "eventemitter3";

/**
 * Dynamic.xyz Wallet Adapter - Reference Implementation
 *
 * This is a reference implementation that shows how to integrate Dynamic.xyz wallets
 * with the Enhanced Vaults SDK. The SDK itself does not depend on Dynamic.xyz packages.
 *
 * To use this adapter, your application needs to install Dynamic.xyz packages:
 * ```bash
 * npm install @dynamic-labs/sdk-react-core @dynamic-labs/solana
 * ```
 *
 * Usage example:
 * ```typescript
 * import { useUserWallets } from '@dynamic-labs/sdk-react-core';
 *
 * const userWallets = useUserWallets();
 * const solanaWallet = userWallets.find(wallet => wallet.chain === 'SOL');
 *
 * if (solanaWallet?.connector) {
 *   const adapter = new DynamicWalletAdapter(solanaWallet.connector);
 *   const sdk = new EnhancedVaultsSdk(connection, adapter);
 * }
 * ```
 *
 * Note: This adapter expects the connector to have Dynamic.xyz's standard interface.
 * You may need to adapt this code based on the specific Dynamic.xyz version you're using.
 */
export class DynamicWalletAdapter
  extends EventEmitter<ISolanaEvents>
  implements ISolanaSigner
{
  private connector: any; // Dynamic wallet connector
  private _isConnected: boolean = false;
  private _publicKey?: PublicKey;

  // ExtensionLocators properties - set based on connector type
  isBraveWallet = false;
  isGlow = false;
  isPhantom = false;
  isSolflare = false;
  isExodus = false;
  isBackpack = false;
  isMagicEden = false;

  constructor(connector: any) {
    super();
    this.connector = connector;
    this._setupEventListeners();

    // Set extension locator based on connector type if available
    if (connector.walletName) {
      const walletName = connector.walletName.toLowerCase();
      this.isPhantom = walletName.includes("phantom");
      this.isSolflare = walletName.includes("solflare");
      this.isBackpack = walletName.includes("backpack");
      this.isGlow = walletName.includes("glow");
      this.isBraveWallet = walletName.includes("brave");
      this.isExodus = walletName.includes("exodus");
      this.isMagicEden = walletName.includes("magiceden");
    }
  }

  private _setupEventListeners() {
    // Listen to Dynamic wallet events and forward them
    if (this.connector.on) {
      this.connector.on("connect", (publicKey: string) => {
        this._publicKey = new PublicKey(publicKey);
        this._isConnected = true;
        this.emit("connect", publicKey);
      });

      this.connector.on("disconnect", () => {
        this._isConnected = false;
        this._publicKey = undefined;
        this.emit("disconnect");
      });

      this.connector.on("accountChanged", (publicKey: string) => {
        this._publicKey = new PublicKey(publicKey);
        this.emit("accountChanged", publicKey);
      });
    }
  }

  get publicKey() {
    return this._publicKey
      ? {
          toBytes: () => this._publicKey!.toBytes(),
        }
      : undefined;
  }

  get isConnected(): boolean {
    return this._isConnected && !!this._publicKey;
  }

  get providers(): ISolanaSigner[] {
    return [this]; // Self-reference for compatibility
  }

  connect = async (args?: {
    onlyIfTrusted: boolean;
  }): Promise<ConnectionResult> => {
    try {
      // Use Dynamic's connect method
      const result = await this.connector.connect?.(args);

      if (result?.publicKey) {
        this._publicKey = new PublicKey(result.publicKey.toString());
        this._isConnected = true;
        this.emit("connect", this._publicKey.toString());
        return {
          publicKey: { toString: () => this._publicKey!.toString() },
          address: this._publicKey.toString(),
        };
      }

      throw new Error("Failed to connect to Dynamic wallet");
    } catch (error) {
      this._isConnected = false;
      throw error;
    }
  };

  async disconnect(): Promise<void> {
    try {
      await this.connector.disconnect?.();
      this._isConnected = false;
      this._publicKey = undefined;
      this.emit("disconnect");
    } catch (error) {
      console.error("Error disconnecting from Dynamic wallet:", error);
      throw error;
    }
  }

  async signTransaction<T extends any>(transaction: T): Promise<T> {
    if (!this.isConnected) {
      throw new Error("Wallet not connected");
    }

    try {
      // Use Dynamic's signTransaction method
      const signedTransaction = await this.connector.signTransaction?.(
        transaction
      );
      return signedTransaction || transaction;
    } catch (error) {
      console.error("Error signing transaction:", error);
      throw error;
    }
  }

  async signAllTransactions<T extends any>(transactions: T[]): Promise<T[]> {
    if (!this.isConnected) {
      throw new Error("Wallet not connected");
    }

    try {
      // Use Dynamic's signAllTransactions method if available
      if (this.connector.signAllTransactions) {
        return await this.connector.signAllTransactions(transactions);
      }

      // Fallback to signing one by one
      const signedTransactions: T[] = [];
      for (const transaction of transactions) {
        const signed = await this.signTransaction(transaction);
        signedTransactions.push(signed);
      }
      return signedTransactions;
    } catch (error) {
      console.error("Error signing transactions:", error);
      throw error;
    }
  }

  async signAndSendTransaction<T extends any>(
    transaction: T,
    options?: any
  ): Promise<{ signature: string }> {
    if (!this.isConnected) {
      throw new Error("Wallet not connected");
    }

    try {
      // Use Dynamic's signAndSendTransaction method if available
      if (this.connector.signAndSendTransaction) {
        const result = await this.connector.signAndSendTransaction(
          transaction,
          options
        );
        return { signature: result.signature || result };
      }

      // Fallback: sign then send manually
      const signedTx = await this.signTransaction(transaction);
      // Note: You would need to implement sending logic here based on your connection
      throw new Error(
        "signAndSendTransaction not implemented in Dynamic connector"
      );
    } catch (error) {
      console.error("Error signing and sending transaction:", error);
      throw error;
    }
  }

  async signMessage(
    message: Uint8Array,
    encoding?: string
  ): Promise<SignedMessage> {
    if (!this.isConnected) {
      throw new Error("Wallet not connected");
    }

    try {
      // Use Dynamic's signMessage method
      const signature = await this.connector.signMessage?.(message, encoding);

      if (!signature) {
        throw new Error("Failed to sign message");
      }

      return {
        signature:
          typeof signature === "string"
            ? new TextEncoder().encode(signature)
            : signature,
      };
    } catch (error) {
      console.error("Error signing message:", error);
      throw error;
    }
  }
}

/**
 * Factory function to create a Dynamic wallet adapter from Dynamic.xyz SDK
 *
 * Note: This is a convenience function. Your application must have Dynamic.xyz packages installed.
 *
 * @param userWallets - Array of user wallets from useUserWallets()
 * @param chainType - Optional chain type to filter for ('SOL' for Solana)
 * @returns DynamicWalletAdapter instance or null if no suitable wallet found
 */
export function createDynamicWalletAdapter(
  userWallets: any[],
  chainType: string = "SOL"
): DynamicWalletAdapter | null {
  const solanaWallet = userWallets.find(
    (wallet) => wallet.chain === chainType && wallet.connector
  );

  if (!solanaWallet?.connector) {
    return null;
  }

  return new DynamicWalletAdapter(solanaWallet.connector);
}

/**
 * React hook helper for using Dynamic wallet with Enhanced Vaults SDK
 *
 * Note: This would typically be in a separate React-specific file
 *
 * Usage:
 * ```typescript
 * const walletAdapter = useDynamicWalletAdapter();
 * const sdk = useMemo(() => {
 *   if (walletAdapter) {
 *     return new EnhancedVaultsSdk(connection, walletAdapter);
 *   }
 *   return null;
 * }, [walletAdapter, connection]);
 * ```
 */
export function useDynamicWalletAdapter(): DynamicWalletAdapter | null {
  // This would use Dynamic's useUserWallets hook
  // Commented out since we don't have React context here
  /*
  const userWallets = useUserWallets();
  return useMemo(() => {
    return createDynamicWalletAdapter(userWallets);
  }, [userWallets]);
  */

  // Placeholder return - in actual usage, implement the above
  return null;
}
