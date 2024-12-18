import * as anchor from "@coral-xyz/anchor";

const connection = anchor.getProvider().connection;

export async function confirmTransaction(
  signature: anchor.web3.TransactionSignature
) {
  const latestBlockHash = await connection.getLatestBlockhash();
  await connection.confirmTransaction(
    {
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature,
    },
    "confirmed"
  );
}

export async function requestAirdrop(publicKey: anchor.web3.PublicKey) {
  const tx = await connection.requestAirdrop(
    publicKey,
    10 * anchor.web3.LAMPORTS_PER_SOL
  );
  await confirmTransaction(tx);
}

export async function getTransactionLogs(
  signature: anchor.web3.TransactionSignature
) {
  const txDetails = await connection.getTransaction(signature, {
    maxSupportedTransactionVersion: 0,
    commitment: "confirmed",
  });
  return txDetails?.meta?.logMessages || [];
}
