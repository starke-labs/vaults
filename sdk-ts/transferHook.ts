import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

import idl from "@starke/idl/transfer_hook.json";

import { VaultConfigNotInitializedError } from "./lib/errors";
import { getVaultConfigPda } from "./lib/pdas";
import { VaultConfig } from "./lib/types";

export class TransferHookSdk {
  private program: Program;
  private provider: AnchorProvider;

  constructor(connection: Connection, keypair: Keypair) {
    this.provider = new AnchorProvider(
      connection,
      new Wallet(keypair),
      AnchorProvider.defaultOptions()
    );
    this.program = new Program(idl, this.provider);
  }

  async fetchVaultConfig(vTokenMint: PublicKey): Promise<VaultConfig> {
    const [vaultConfigPda] = getVaultConfigPda(vTokenMint);
    try {
      // @ts-ignore
      return (await this.program.account.vaultConfig.fetch(
        vaultConfigPda
      )) as VaultConfig;
    } catch (e) {
      if (e.toString().includes("Account does not exist")) {
        throw new VaultConfigNotInitializedError(vaultConfigPda);
      }
      throw e;
    }
  }
}
