import * as anchor from "@coral-xyz/anchor";
import "dotenv/config";

import { VaultManager } from "../target/types/vault_manager";

describe("Whitelist", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.VaultManager as anchor.Program<VaultManager>;
  const programAuthority = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(
      process.env
        .PROGRAM_AUTHORITY_SECRET_KEY!.split(",")
        .map((num) => parseInt(num))
    )
  );
});
