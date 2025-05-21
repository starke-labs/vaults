import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

import idl from "@starke/idl/transfer_hook.json";

import { VtokenConfigNotInitializedError } from "./lib/errors";
import { getVtokenConfigPda } from "./lib/pdas";
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

  async fetchVtokenConfig(vTokenMint: PublicKey): Promise<VaultConfig> {
    const [vtokenConfigPda] = getVtokenConfigPda(vTokenMint);
    try {
      // @ts-ignore
      return (await this.program.account.vtokenConfig.fetch(
        vtokenConfigPda
      )) as VaultConfig;
    } catch (e) {
      if (e.toString().includes("Account does not exist")) {
        throw new VtokenConfigNotInitializedError(vtokenConfigPda);
      }
      throw e;
    }
  }

  // TODO: Implement sdk method for instruction `set_vtoken_is_transferrable`
}
