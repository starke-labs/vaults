import * as anchor from "@coral-xyz/anchor";
import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
} from "@solana/spl-token";
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

  const ata = await createAssociatedTokenAccount(
    connection,
    authority,
    mint,
    accountOwner,
    undefined,
    undefined,
    undefined,
    true
  );

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

  // Vault
  let vault: anchor.web3.PublicKey;

  // Whitelist
  let whitelist: anchor.web3.PublicKey;

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

    [vault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("STARKE_VAULT"), manager.publicKey.toBuffer()],
      program.programId
    );

    [whitelist] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("STARKE_TOKEN_WHITELIST")],
      program.programId
    );

    ({ mint: mintA, ata: vaultATA_A } = await tokenSetup(
      provider.connection,
      manager,
      vault,
      1000
    ));

    ({ mint: mintB, ata: vaultATA_B } = await tokenSetup(
      provider.connection,
      manager,
      vault,
      1500
    ));

    ({ mint: mintC, ata: vaultATA_C } = await tokenSetup(
      provider.connection,
      manager,
      vault,
      2000
    ));
  });

  it("successfully creates a whitelist", async () => {
    const signature = await program.methods
      .initializeWhitelist()
      .accounts({ authority: programAuthority.publicKey })
      .signers([programAuthority])
      .rpc();

    await confirmTransaction(signature);
  });

  it("successfully whitelists the mints", async () => {
    const signatureA = await program.methods
      .addToken(mintA, "0xA")
      .accounts({
        authority: programAuthority.publicKey,
        whitelist: whitelist,
      })
      .signers([programAuthority])
      .rpc();

    await confirmTransaction(signatureA);

    const signatureB = await program.methods
      .addToken(mintB, "0xB")
      .accounts({
        authority: programAuthority.publicKey,
        whitelist: whitelist,
      })
      .signers([programAuthority])
      .rpc();

    await confirmTransaction(signatureB);

    const signatureC = await program.methods
      .addToken(mintC, "0xC")
      .accounts({
        authority: programAuthority.publicKey,
        whitelist: whitelist,
      })
      .signers([programAuthority])
      .rpc();

    await confirmTransaction(signatureC);
  });

  it("successfully creates a vault", async () => {
    const signature = await program.methods
      .createVault("Test Vault", 100, 100)
      .accounts({
        manager: manager.publicKey,
        depositTokenMint: mintA,
      })
      .signers([manager])
      .rpc();

    await confirmTransaction(signature);
  });

  it("successfully tests remaining accounts", async () => {
    const signature = await program.methods
      .testRemainingAccounts()
      .accounts({
        signer: manager.publicKey,
        manager: manager.publicKey,
      })
      .remainingAccounts([
        {
          pubkey: mintA,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: vaultATA_A,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: mintB,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: vaultATA_B,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: mintC,
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
