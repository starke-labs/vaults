import { AnchorProvider, Idl, Program, Wallet } from "@coral-xyz/anchor";
import {
  Connection,
  PublicKey,
  Signer,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";

import { EventHandler } from "./events";
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
    wallet: Wallet,
    programId: PublicKey,
    idl: Idl
  ) {
    this.provider = new AnchorProvider(
      connection,
      wallet,
      AnchorProvider.defaultOptions()
    );
    this.program = new Program(idl, this.provider);
    this.events = new EventHandler(connection, programId);
  }

  // Instruction methods
  async createVault(params: CreateVaultParams, accounts: CreateVaultAccounts) {
    const instruction = await this.program.methods
      .createVault(params.name, params.entryFee, params.exitFee)
      .accounts({
        manager: accounts.manager,
        depositTokenMint: accounts.depositTokenMint,
      })
      .instruction();

    return instruction;
  }

  async initializeWhitelist() {
    const instruction = await this.program.methods
      .initializeWhitelist()
      .instruction();

    return instruction;
  }

  async addToken(params: AddTokenParams, accounts: AddTokenAccounts) {
    const instruction = await this.program.methods
      .addToken(params.token, params.priceFeedId)
      .accounts({
        authority: accounts.authority,
      })
      .instruction();

    return instruction;
  }

  async deposit(params: DepositParams, accounts: DepositAccounts) {
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

  async withdraw(params: WithdrawParams, accounts: WithdrawAccounts) {
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
  ) {
    const instruction = await this.program.methods
      .updateVaultFees(params.newEntryFee, params.newExitFee)
      .accounts({
        manager: accounts.manager,
      })
      .instruction();

    return instruction;
  }

  async swapOnJupiter(params: any) {
    // Implementation depends on Jupiter API integration
    const instruction = await this.program.methods
      .swapOnJupiter(params)
      .instruction();

    return instruction;
  }

  // Helper method to send and confirm transaction
  async sendAndConfirmTransaction(
    instructions: TransactionInstruction[],
    signers?: Signer[]
  ) {
    try {
      const tx = new Transaction();
      tx.add(...instructions);
      const signature = await this.provider.sendAndConfirm(tx, signers, {
        commitment: "confirmed",
      });
      return signature;
    } catch (error) {
      throw new Error(`Transaction failed: ${error}`);
    }
  }
}
