import { Instruction } from "@jup-ag/api";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

export function constructSwapInstruction(
  jupiterSwapInstruction: Instruction
): TransactionInstruction {
  return new TransactionInstruction({
    programId: new PublicKey(jupiterSwapInstruction.programId),
    keys: jupiterSwapInstruction.accounts.map((account) => ({
      pubkey: new PublicKey(account.pubkey),
      isSigner: false,
      isWritable: account.isWritable,
    })),
    data: Buffer.from(jupiterSwapInstruction.data, "base64"),
  });
}
