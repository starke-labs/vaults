import { PublicKey } from "@solana/web3.js";

export interface WhitelistedToken {
  mint: PublicKey;
  priceFeedId: string;
  priceUpdate: PublicKey;
}

export const USDC: WhitelistedToken = {
  mint: new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
  priceFeedId:
    "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a",
  // NOTE: This is the sponsored price feed account for USDC
  priceUpdate: new PublicKey("Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX"),
};

export const USDT: WhitelistedToken = {
  mint: new PublicKey("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
  priceFeedId:
    "0x2b89b9dc8fdf9f34709a5b106b472f0f39bb6ca9ce04b0fd7f2e971688e2e53b",
  // NOTE: This is the sponsored price feed account for USDT
  priceUpdate: new PublicKey("HT2PLQBcG5EiCcNSaMHAjSgd9F98ecpATbk4Sk5oYuM"),
};

export const JUP: WhitelistedToken = {
  mint: new PublicKey("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"),
  priceFeedId:
    "0x0a0408d619e9380abad35060f9192039ed5042fa6f82301d0e48bb52be830996",
  // NOTE: This is the sponsored price feed account for JUP
  priceUpdate: new PublicKey("7dbob1psH1iZBS7qPsm3Kwbf5DzSXK8Jyg31CTgTnxH5"),
};

export const PYTH: WhitelistedToken = {
  mint: new PublicKey("HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3"),
  priceFeedId:
    "0x0bbf28e9a841a1cc788f6a361b17ca072d0ea3098a1e5df1c3922d06719579ff",
  // NOTE: This is the sponsored price feed account for PYTH
  priceUpdate: new PublicKey("8vjchtMuJNY4oFQdTi8yCe6mhCaNBFaUbktT482TpLPS"),
};
