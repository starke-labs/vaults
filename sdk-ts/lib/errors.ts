import { PublicKey } from "@solana/web3.js";

export class VaultsError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "VaultsError";
  }
}

export class StarkeNotInitializedError extends VaultsError {
  constructor() {
    super("Starke not initialized");
    this.name = "StarkeNotInitializedError";
  }
}

export class StarkeAlreadyInitializedError extends VaultsError {
  constructor() {
    super(`Starke already initialized`);
    this.name = "StarkeAlreadyInitializedError";
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

export class ManagerAlreadyInWhitelistError extends VaultsError {
  constructor(manager: PublicKey) {
    super(`Manager ${manager.toBase58()} is already in whitelist`);
    this.name = "ManagerAlreadyInWhitelistError";
  }
}

export class ManagerNotWhitelistedError extends VaultsError {
  constructor(manager: PublicKey) {
    super(`Manager ${manager.toBase58()} is not whitelisted`);
    this.name = "ManagerNotWhitelistedError";
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

export class VaultNotFoundError extends VaultsError {
  constructor(manager: PublicKey) {
    super(`No vault found for manager ${manager.toBase58()}`);
    this.name = "VaultNotFoundError";
  }
}

export class AccountNotInitializedError extends VaultsError {
  constructor(accountName: string) {
    super(`Account ${accountName} not initialized`);
    this.name = "AccountNotInitializedError";
  }
}

export class VtokenConfigNotInitializedError extends VaultsError {
  constructor(vtokenConfig: PublicKey) {
    super(`Vtoken config ${vtokenConfig.toBase58()} not initialized`);
    this.name = "VtokenConfigNotInitializedError";
  }
}
