import { PublicKey } from "@solana/web3.js";

export class VaultsError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "VaultsError";
  }
}

export class WhitelistNotInitializedError extends VaultsError {
  constructor() {
    super("Whitelist not initialized");
    this.name = "WhitelistNotInitializedError";
  }
}

export class WhitelistAlreadyInitializedError extends VaultsError {
  constructor(whitelist: PublicKey) {
    super(`Whitelist ${whitelist.toBase58()} already initialized`);
    this.name = "WhitelistAlreadyInitializedError";
  }
}

export class TokenAlreadyInWhitelistError extends VaultsError {
  constructor(token: PublicKey) {
    super(`Token ${token.toBase58()} is already in whitelist`);
    this.name = "TokenAlreadyInWhitelistError";
  }
}

export class SignatureVerificationFailedError extends VaultsError {
  constructor(signer: PublicKey) {
    super(`Signature verification failed for ${signer.toBase58()}`);
    this.name = "SignatureVerificationFailedError";
  }
}

export class TokenNotWhitelistedError extends VaultsError {
  constructor(mint: PublicKey) {
    super(`Token ${mint.toBase58()} is not whitelisted`);
    this.name = "TokenNotWhitelistedError";
  }
}

export class InvalidTokenError extends VaultsError {
  constructor(mint: PublicKey) {
    super(`Token ${mint.toBase58()} is not a valid SPL token`);
    this.name = "InvalidTokenError";
  }
}

export class VaultAlreadyCreatedError extends VaultsError {
  constructor(vault: PublicKey) {
    super(`Vault ${vault.toBase58()} already created`);
    this.name = "VaultAlreadyCreatedError";
  }
}

export class InsufficientBalanceError extends VaultsError {
  constructor(account: PublicKey) {
    super(`Insufficient balance for ${account.toBase58()}`);
    this.name = "InsufficientBalanceError";
  }
}

// TODO: Think about error handling
function mapError(error: Error, kwargs: Record<string, any>): VaultsError {
  const e = error.toString();
  // TODO: Add more cases
  return new VaultsError(e);
}
