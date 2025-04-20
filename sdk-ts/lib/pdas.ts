import { PublicKey } from "@solana/web3.js";

// export const PROGRAM_ID = new PublicKey(
//   "STRK2VEGPAKstk6S6k5Cpin6uGtSDQkvanTaXUeaTNj"
// );
// TODO: Find a way to get the program id from the IDL
export const VAULTS_PROGRAM_ID = new PublicKey(
  "HULUoT5PHWhoj9ELE1XHFo8jJqHj89uGwWNq3Skrja28"
);
export const TRANSFER_HOOK_PROGRAM_ID = new PublicKey(
  "3Mbtr8yzqLUuBZVSefrVtAPmgNLFutEXeRWJNATsKU5z"
);
export const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

export const AUTHORITY_PROGRAM_ID = new PublicKey(
  "STRK1me6eFLDYGKYqbn2oyHsaxiCHe8GDWQnnSGiScS"
);

// Seeds
export const WHITELIST_SEED = "STARKE_TOKEN_WHITELIST";
export const VAULT_SEED = "STARKE_VAULT";
export const VTOKEN_MINT_SEED = "STARKE_VTOKEN_MINT";
export const VAULT_CONFIG_SEED = "STARKE_VAULT_CONFIG";
export const METADATA_SEED = "metadata";

// PDAs
export function getWhitelistPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(WHITELIST_SEED)],
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

export function getVaultConfigPda(vTokenMint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_CONFIG_SEED), vTokenMint.toBuffer()],
    TRANSFER_HOOK_PROGRAM_ID
  );
}
