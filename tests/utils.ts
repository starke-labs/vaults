import * as anchor from "@coral-xyz/anchor";

const connection = anchor.getProvider().connection;

export async function confirmTransaction(
  signature: anchor.web3.TransactionSignature
) {
  const latestBlockHash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    blockhash: latestBlockHash.blockhash,
    lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    signature,
  });
}

export async function requestAirdrop(publicKey: anchor.web3.PublicKey) {
  const tx = await connection.requestAirdrop(
    publicKey,
    100 * anchor.web3.LAMPORTS_PER_SOL
  );
  await confirmTransaction(tx);
}
