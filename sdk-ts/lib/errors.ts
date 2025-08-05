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

export class StarkeAlreadyPausedError extends VaultsError {
  constructor() {
    super("Starke is already paused");
    this.name = "StarkeAlreadyPausedError";
  }
}

export class StarkeAlreadyResumedError extends VaultsError {
  constructor() {
    super("Starke is already resumed");
    this.name = "StarkeAlreadyResumedError";
  }
}

export class StarkePausedError extends VaultsError {
  constructor() {
    super("Starke is paused");
    this.name = "StarkePausedError";
  }
}

export class DepositBelowMinimumError extends VaultsError {
  constructor(amount: string, minimum: string) {
    super(`Deposit amount ${amount} is below minimum ${minimum}`);
    this.name = "DepositBelowMinimumError";
  }
}

export class MaxAumExceededError extends VaultsError {
  constructor(currentAum: string, maxAum: string) {
    super(`Maximum AUM limit exceeded. Current: ${currentAum}, Max: ${maxAum}`);
    this.name = "MaxAumExceededError";
  }
}

export class InvalidAmountError extends VaultsError {
  constructor() {
    super("Invalid amount: must be greater than 0");
    this.name = "InvalidAmountError";
  }
}


export class UserNotWhitelistedError extends VaultsError {
  constructor(user: PublicKey) {
    super(`User ${user.toBase58()} is not whitelisted`);
    this.name = "UserNotWhitelistedError";
  }
}

export class InvestorTypeNotAllowedError extends VaultsError {
  constructor(investorType: string) {
    super(`Investor type ${investorType} is not allowed for this vault`);
    this.name = "InvestorTypeNotAllowedError";
  }
}

export class MaxDepositorsExceededError extends VaultsError {
  constructor() {
    super("Maximum number of depositors exceeded for this vault");
    this.name = "MaxDepositorsExceededError";
  }
}
