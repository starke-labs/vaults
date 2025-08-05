import { PublicKey } from "@solana/web3.js";

// export const PROGRAM_ID = new PublicKey(
//   "STRK2VEGPAKstk6S6k5Cpin6uGtSDQkvanTaXUeaTNj"
// );
// !!TODO: Find a way to get the program id from the IDL
export const VAULTS_PROGRAM_ID = new PublicKey(
  "8mkCdpBLeEEiGTu3y5rRyAWMfjBEw3Qm8SNQmMXNhG5f"
);
export const TRANSFER_HOOK_PROGRAM_ID = new PublicKey(
  "Gk7syLzEbk46Ez6Fr9pApPPhTJMDavKxiN9JHAtfhZCz"
);
export const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

export const AUTHORITY_PROGRAM_ID = new PublicKey(
  "STRK1me6eFLDYGKYqbn2oyHsaxiCHe8GDWQnnSGiScS"
);

// Seeds
export const TOKEN_WHITELIST_SEED = "STARKE_TOKEN_WHITELIST";
export const MANAGER_WHITELIST_SEED = "STARKE_MANAGER_WHITELIST";
export const USER_WHITELIST_SEED = "STARKE_USER_WHITELIST";
export const STARKE_CONFIG_SEED = "STARKE_CONFIG";
export const VAULT_SEED = "STARKE_VAULT";
export const VTOKEN_MINT_SEED = "STARKE_VTOKEN_MINT";
export const VTOKEN_CONFIG_SEED = "STARKE_VTOKEN_CONFIG";
export const METADATA_SEED = "metadata";
export const EXTRA_ACCOUNT_METAS_SEED = "extra-account-metas";
// PDAs
export function getTokenWhitelistPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(TOKEN_WHITELIST_SEED)],
    VAULTS_PROGRAM_ID
  );
}

export function getManagerWhitelistPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(MANAGER_WHITELIST_SEED)],
    VAULTS_PROGRAM_ID
  );
}

export function getUserWhitelistPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(USER_WHITELIST_SEED)],
    VAULTS_PROGRAM_ID
  );
}

export function getStarkeConfigPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(STARKE_CONFIG_SEED)],
    VAULTS_PROGRAM_ID
  );
}

export function getVaultPda(manager: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), manager.toBuffer()],
    VAULTS_PROGRAM_ID
  );
}

export function getVtokenMintPda(vault: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(VTOKEN_MINT_SEED), vault.toBuffer()],
    VAULTS_PROGRAM_ID
  );
}

export function getVtokenMetadataPda(
  vtokenMint: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_SEED),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      vtokenMint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );
}

export function getVtokenConfigPda(vTokenMint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(VTOKEN_CONFIG_SEED), vTokenMint.toBuffer()],
    TRANSFER_HOOK_PROGRAM_ID
  );
}

export function getExtraAccountMetasPda(
  vTokenMint: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(EXTRA_ACCOUNT_METAS_SEED), vTokenMint.toBuffer()],
    TRANSFER_HOOK_PROGRAM_ID
  );
}
