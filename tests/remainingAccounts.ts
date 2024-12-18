import * as anchor from "@coral-xyz/anchor";
import { createAccount, createMint, mintTo } from "@solana/spl-token";
import fs from "fs";

import { Vaults } from "../target/types/vaults";
import {
  confirmTransaction,
  getTransactionLogs,
  requestAirdrop,
} from "./utils";

const DECIMALS = 6;
const TOKEN_FACTOR = Math.pow(10, DECIMALS);

async function tokenSetup(
  connection: anchor.web3.Connection,
  authority: anchor.web3.Keypair,
  accountOwner: anchor.web3.PublicKey,
  amount: number
) {
  const mint = await createMint(
    connection,
    authority,
    authority.publicKey,
    null,
    DECIMALS
  );

  const ata = await createAccount(connection, authority, mint, accountOwner);

  await mintTo(
    connection,
    authority,
    mint,
    ata,
    authority,
    amount * TOKEN_FACTOR
  );

  return { mint, ata };
}

describe("Remaining Accounts", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Vaults as anchor.Program<Vaults>;

  // Test accounts
  const programAuthority = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(
      JSON.parse(fs.readFileSync("./deploy/authority.json", "utf8"))
    )
  );
  const manager = anchor.web3.Keypair.generate();

  // Token mints
  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;
  let mintC: anchor.web3.PublicKey;

  // Vault accounts
  let vaultATA_A: anchor.web3.PublicKey;
  let vaultATA_B: anchor.web3.PublicKey;
  let vaultATA_C: anchor.web3.PublicKey;

  before(async () => {
    await requestAirdrop(manager.publicKey);

    ({ mint: mintA, ata: vaultATA_A } = await tokenSetup(
      provider.connection,
      manager,
      manager.publicKey,
      1000
    ));

    ({ mint: mintB, ata: vaultATA_B } = await tokenSetup(
      provider.connection,
      manager,
      manager.publicKey,
      1500
    ));

    ({ mint: mintC, ata: vaultATA_C } = await tokenSetup(
      provider.connection,
      manager,
      manager.publicKey,
      2000
    ));

    console.log(
      "vaultATA_A",
      await provider.connection.getTokenAccountBalance(vaultATA_A)
    );

    console.log(
      "vaultATA_B",
      await provider.connection.getTokenAccountBalance(vaultATA_B)
    );

    console.log(
      "vaultATA_C",
      await provider.connection.getTokenAccountBalance(vaultATA_C)
    );
  });

  it("successfully tests remaining accounts", async () => {
    const signature = await program.methods
      .testRemainingAccounts()
      .accounts({ signer: manager.publicKey })
      .remainingAccounts([
        {
          pubkey: vaultATA_A,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: vaultATA_B,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: vaultATA_C,
          isSigner: false,
          isWritable: false,
        },
      ])
      .signers([manager])
      .rpc();

    await confirmTransaction(signature);
    console.log(await getTransactionLogs(signature));
  });
});
