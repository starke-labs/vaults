import { AnchorProvider, Idl, Program, Wallet } from "@coral-xyz/anchor";
import {
  ConfirmOptions,
  Connection,
  Keypair,
  PublicKey,
  Signer,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";

import { EventHandler } from "./events";
import { getVaultPda, getVaultTokenMintPda, getWhitelistPda } from "./pdas";
import {
  AddTokenAccounts,
  AddTokenParams,
  CreateVaultAccounts,
  CreateVaultParams,
  DepositAccounts,
  DepositParams,
  UpdateFeesAccounts,
  UpdateFeesParams,
  WithdrawAccounts,
  WithdrawParams,
} from "./types";

export class VaultsSDK {
  private program: Program;
  private provider: AnchorProvider;
  public events: EventHandler;

  constructor(
    connection: Connection,
    keypair: Keypair,
    programId: PublicKey,
    idl: Idl
  ) {
    this.provider = new AnchorProvider(
      connection,
      new Wallet(keypair),
      AnchorProvider.defaultOptions()
    );
    this.program = new Program(idl, this.provider);
    this.events = new EventHandler(connection, programId);
  }

  // Instruction methods
  async createVault(
    params: CreateVaultParams,
    accounts: CreateVaultAccounts
  ): Promise<TransactionInstruction> {
    const instruction = await this.program.methods
      .createVault(params.name, params.entryFee, params.exitFee)
      .accounts({
        manager: accounts.manager,
        depositTokenMint: accounts.depositTokenMint,
      })
      .instruction();

    return instruction;
  }

  async initializeWhitelist(): Promise<TransactionInstruction> {
    const instruction = await this.program.methods
      .initializeWhitelist()
      .accounts({})
      .instruction();

    return instruction;
  }

  async addToken(
    params: AddTokenParams,
    accounts: AddTokenAccounts
  ): Promise<TransactionInstruction> {
    const instruction = await this.program.methods
      .addToken(params.token, params.priceFeedId)
      .accounts({
        authority: accounts.authority,
      })
      .instruction();

    return instruction;
  }

  async deposit(
    params: DepositParams,
    accounts: DepositAccounts
  ): Promise<TransactionInstruction> {
    const instruction = await this.program.methods
      .deposit(params.amount)
      .accounts({
        user: accounts.user,
        manager: accounts.manager,
        depositTokenMint: accounts.depositTokenMint,
        priceUpdate: accounts.priceUpdate,
      })
      .instruction();

    return instruction;
  }

  async withdraw(
    params: WithdrawParams,
    accounts: WithdrawAccounts
  ): Promise<TransactionInstruction> {
    const instruction = await this.program.methods
      .withdraw(params.amount)
      .accounts({
        user: accounts.user,
        manager: accounts.manager,
        depositTokenMint: accounts.depositTokenMint,
        priceUpdate: accounts.priceUpdate,
      })
      .instruction();

    return instruction;
  }

  async updateVaultFees(
    params: UpdateFeesParams,
    accounts: UpdateFeesAccounts
  ): Promise<TransactionInstruction> {
    const instruction = await this.program.methods
      .updateVaultFees(params.newEntryFee, params.newExitFee)
      .accounts({
        manager: accounts.manager,
      })
      .instruction();

    return instruction;
  }

  async swapOnJupiter(params: any): Promise<TransactionInstruction> {
    // Implementation depends on Jupiter API integration
    const instruction = await this.program.methods
      .swapOnJupiter(params)
      .instruction();

    return instruction;
  }

  // Fetch state methods
  async fetchWhitelist() {
    // @ts-ignore
    return await this.program.account.tokenWhitelist.fetch(
      getWhitelistPda()[0]
    );
  }

  async fetchVault(manager: PublicKey) {
    // @ts-ignore
    return await this.program.account.vault.fetch(getVaultPda(manager)[0]);
  }

  // Transaction methods
  async sendTransaction(
    instructions: TransactionInstruction[],
    signers: Signer[] = [],
    options: ConfirmOptions = {}
  ): Promise<string> {
    const tx = new Transaction();
    tx.add(...instructions);
    const signature = await this.provider.sendAndConfirm(tx, signers, options);
    return signature;
  }
}
