import { PublicKey } from "@solana/web3.js";

export const PROGRAM_ID = new PublicKey(
  "STRK2VEGPAKstk6S6k5Cpin6uGtSDQkvanTaXUeaTNj"
);

// Seeds
export const WHITELIST_SEED = "STARKE_TOKEN_WHITELIST";
export const VAULT_SEED = "STARKE_VAULT";
export const VAULT_TOKEN_MINT_SEED = "STARKE_VAULT_TOKEN_MINT";

// PDAs
export function getWhitelistPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(WHITELIST_SEED)],
    PROGRAM_ID
  );
}

export function getVaultPda(manager: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), manager.toBuffer()],
    PROGRAM_ID
  );
}

export function getVaultTokenMintPda(vault: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_TOKEN_MINT_SEED), vault.toBuffer()],
    PROGRAM_ID
  );
}
