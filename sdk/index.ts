import { Program, AnchorProvider, Idl, Wallet } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import { EventHandler } from "./events";
import {
    VaultConfig,
    TokenConfig,
    WithdrawParams,
    DepositParams,
    UpdateFeesParams,
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
    async createVault(config: VaultConfig) {
        const instruction = await this.program.methods
            .createVault(config.name, config.entryFee, config.exitFee)
            .instruction();
        
        return instruction;
    }

    async initializeWhitelist() {
        const instruction = await this.program.methods
            .initializeWhitelist()
            .instruction();
        
        return instruction;
    }

    async addToken(config: TokenConfig) {
        const instruction = await this.program.methods
            .addToken(config.mint)
            .instruction();
        
        return instruction;
    }

    async deposit(params: DepositParams) {
        const instruction = await this.program.methods
            .deposit(params.amount)
            .accounts({
                vault: params.vault,
            })
            .instruction();
        
        return instruction;
    }

    async withdraw(params: WithdrawParams) {
        const instruction = await this.program.methods
            .withdraw(params.amount)
            .accounts({
                vault: params.vault,
            })
            .instruction();
        
        return instruction;
    }

    async updateVaultFees(params: UpdateFeesParams) {
        const instruction = await this.program.methods
            .updateVaultFees(params.newEntryFee, params.newExitFee)
            .accounts({
                vault: params.vault,
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
    async sendAndConfirmTransaction(instructions: Transaction) {
        try {
            const signature = await this.provider.sendAndConfirm(instructions);
            return signature;
        } catch (error) {
            throw new Error(`Transaction failed: ${error}`);
        }
    }
}
