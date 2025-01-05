import { PublicKey } from "@solana/web3.js";

export const PROGRAM_ID = new PublicKey(
  "STRK2VEGPAKstk6S6k5Cpin6uGtSDQkvanTaXUeaTNj"
);

// Seeds
export const WHITELIST_SEED = "STARKE_TOKEN_WHITELIST";

// PDAs
export const [WHITELIST_PDA] = PublicKey.findProgramAddressSync(
  [Buffer.from(WHITELIST_SEED)],
  PROGRAM_ID
);
