import { PublicKey } from "@solana/web3.js";

export class WhitelistNotInitializedError extends Error {
  constructor() {
    super("Whitelist not initialized");
    this.name = "WhitelistNotInitializedError";
  }
}

export class WhitelistAlreadyInitializedError extends Error {
  constructor(whitelist: PublicKey) {
    super(`Whitelist ${whitelist.toBase58()} already initialized`);
    this.name = "WhitelistAlreadyInitializedError";
  }
}

export class TokenAlreadyInWhitelistError extends Error {
  constructor(token: PublicKey) {
    super(`Token ${token.toBase58()} is already in whitelist`);
    this.name = "TokenAlreadyInWhitelistError";
  }
}

export class SignatureVerificationFailedError extends Error {
  constructor(authority: PublicKey) {
    super(
      `Missing authority ${authority.toBase58()} to add token to whitelist`
    );
    this.name = "SignatureVerificationFailedError";
  }
}

export class TokenNotWhitelistedError extends Error {
  constructor(mint: PublicKey) {
    super(`Token ${mint.toBase58()} is not whitelisted`);
    this.name = "TokenNotWhitelistedError";
  }
}
