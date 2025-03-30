import { PublicKey } from "@solana/web3.js";

export interface Token {
  mint: PublicKey;
  priceFeedId: string;
  priceUpdate: PublicKey;
}

export interface Whitelist {
  tokens: Token[];
  authority: PublicKey;
  bump: number;
}
