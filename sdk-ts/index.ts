import { AnchorProvider, Idl, Program, Wallet } from "@coral-xyz/anchor";
import { DefaultApi, Instruction, createJupiterApiClient } from "@jup-ag/api";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import {
  AddressLookupTableAccount,
  ComputeBudgetProgram,
  ConfirmOptions,
  Connection,
  Keypair,
  PublicKey,
  Signer,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";

import { EventHandler } from "./events";
import { getVaultPda, getWhitelistPda } from "./pdas";
import {
  AddTokenAccounts,
  AddTokenParams,
  CreateVaultAccounts,
  CreateVaultParams,
  DepositAccounts,
  DepositParams,
  SwapOnJupiterAccounts,
  SwapOnJupiterParams,
  UpdateFeesAccounts,
  UpdateFeesParams,
  WithdrawAccounts,
  WithdrawParams,
} from "./types";
import {
  DEFAULT_RETRY_CONFIG,
  TransactionRetryConfig,
  sendAndConfirmWithRetry,
} from "./utils";

// TODO: Move this to somewhere else
interface Token {
  mint: PublicKey;
  priceFeedId: string;
}

interface AccountMeta {
  pubkey: PublicKey;
  isWritable: boolean;
  isSigner: boolean;
}

export class VaultsSDK {
  private program: Program;
  private provider: AnchorProvider;
  public events: EventHandler;

  // Jup API client
  private jup: DefaultApi;

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

    // Jup API client
    this.jup = createJupiterApiClient();
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
    accounts: DepositAccounts,
    getPriceUpdateAccount: (priceFeedId: string) => PublicKey
  ): Promise<TransactionInstruction> {
    const tokenAccounts =
      await this.provider.connection.getParsedTokenAccountsByOwner(
        getVaultPda(accounts.manager)[0],
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    // TODO: Enable 2022 tokens
    // const token2022Accounts =
    //   await this.provider.connection.getParsedTokenAccountsByOwner(
    //     getVaultPda(accounts.manager)[0],
    //     {
    //       programId: TOKEN_2022_PROGRAM_ID,
    //     }
    //   );
    const allTokenAccounts = [
      ...tokenAccounts.value,
      // ...token2022Accounts.value,
    ];
    const whitelistedTokens = (await this.fetchWhitelist()).tokens as Token[];
    const remainingAccounts = allTokenAccounts.reduce((prev, account) => {
      const tokenMint = new PublicKey(account.account.data.parsed.info.mint);
      const token = whitelistedTokens.find(
        (token) => token.mint.toBase58() === tokenMint.toBase58()
      );
      if (!token) {
        return prev;
      }
      return [
        ...prev,
        // Token mint
        {
          pubkey: tokenMint,
          isWritable: false,
          isSigner: false,
        },
        // Token account
        {
          pubkey: account.pubkey,
          isWritable: false,
          isSigner: false,
        },
        // Price update account
        {
          pubkey: getPriceUpdateAccount(token.priceFeedId),
          isWritable: false,
          isSigner: false,
        },
      ];
    }, []);

    const depositTokenPriceFeedId = whitelistedTokens.find(
      (token) => token.mint.toBase58() === accounts.depositTokenMint.toBase58()
    )?.priceFeedId;
    if (!depositTokenPriceFeedId) {
      throw new Error("Deposit token not whitelisted");
    }

    const instruction = await this.program.methods
      .deposit(params.amount)
      .accounts({
        user: accounts.user,
        manager: accounts.manager,
        depositTokenMint: accounts.depositTokenMint,
        depositTokenPriceUpdate: getPriceUpdateAccount(depositTokenPriceFeedId),
      })
      .remainingAccounts(remainingAccounts)
      .instruction();

    return instruction;
  }

  async withdraw(
    params: WithdrawParams,
    accounts: WithdrawAccounts
  ): Promise<TransactionInstruction> {
    const tokenAccounts =
      await this.provider.connection.getParsedTokenAccountsByOwner(
        getVaultPda(accounts.manager)[0],
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    // TODO: Enable 2022 tokens
    // const token2022Accounts =
    //   await this.provider.connection.getParsedTokenAccountsByOwner(
    //     getVaultPda(accounts.manager)[0],
    //     {
    //       programId: TOKEN_2022_PROGRAM_ID,
    //     }
    //   );
    const allTokenAccounts = [
      ...tokenAccounts.value,
      // ...token2022Accounts.value,
    ];

    const whitelistedTokens = (await this.fetchWhitelist()).tokens as Token[];

    const remainingAccounts: AccountMeta[] = [];

    for (const vaultTokenAccount of allTokenAccounts) {
      const tokenMint = new PublicKey(
        vaultTokenAccount.account.data.parsed.info.mint
      );

      const token = whitelistedTokens.find(
        (token) => token.mint.toBase58() === tokenMint.toBase58()
      );
      if (!token) {
        continue;
      }

      const userTokenAccount = await getAssociatedTokenAddress(
        tokenMint,
        accounts.user
      );

      // Token mint
      remainingAccounts.push({
        pubkey: tokenMint,
        isWritable: false,
        isSigner: false,
      });

      // Vault token account
      remainingAccounts.push({
        pubkey: vaultTokenAccount.pubkey,
        isWritable: true,
        isSigner: false,
      });

      // User token account
      remainingAccounts.push({
        pubkey: userTokenAccount,
        isWritable: true,
        isSigner: false,
      });
    }

    const instruction = await this.program.methods
      .withdraw(params.amount)
      .accounts({
        user: accounts.user,
        manager: accounts.manager,
      })
      .remainingAccounts(remainingAccounts)
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

  async swapOnJupiter(
    params: SwapOnJupiterParams,
    accounts: SwapOnJupiterAccounts,
    signers: Signer[]
  ): Promise<string> {
    const quoteResponse = await this.jup.quoteGet({
      inputMint: accounts.inputMint.toBase58(),
      outputMint: accounts.outputMint.toBase58(),
      amount: params.amount.toNumber(),
    });

    const vault = getVaultPda(accounts.manager)[0];

    const swapIxsResponse = await this.jup.swapInstructionsPost({
      swapRequest: {
        userPublicKey: vault.toBase58(),
        quoteResponse,
      },
    });

    function constructIxPayloadToTxIx(ixPayload: Instruction) {
      return new TransactionInstruction({
        programId: new PublicKey(ixPayload.programId),
        keys: ixPayload.accounts.map((account) => ({
          pubkey: new PublicKey(account.pubkey),
          isSigner: false,
          isWritable: account.isWritable,
        })),
        data: Buffer.from(ixPayload.data, "base64"),
      });
    }

    const swapIx = constructIxPayloadToTxIx(swapIxsResponse.swapInstruction);

    const instruction = await this.program.methods
      .swapOnJupiter(swapIx.data)
      .accounts({
        manager: accounts.manager,
        inputTokenMint: accounts.inputMint,
        outputTokenMint: accounts.outputMint,
      })
      .remainingAccounts(swapIx.keys)
      .instruction();

    async function getAddressLookupTables(
      connection: Connection,
      addressLookupTableAddresses: string[]
    ): Promise<AddressLookupTableAccount[]> {
      const keys = addressLookupTableAddresses.map(
        (address) => new PublicKey(address)
      );
      const addressLookupTableAccountInfos =
        await connection.getMultipleAccountsInfo(keys);

      return addressLookupTableAccountInfos.map((info, index) => {
        const key = keys[index];
        if (info) {
          return new AddressLookupTableAccount({
            key,
            state: AddressLookupTableAccount.deserialize(info.data),
          });
        }
      });
    }

    const modifyComputeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 1_400_000,
    });

    const addPriorityFeeIx = ComputeBudgetProgram.setComputeUnitPrice({
      // Does not work with 50k micro lamports, works with 100k
      microLamports: 100_000,
    });

    let recentBlockhash = (await this.provider.connection.getLatestBlockhash())
      .blockhash;
    const messageV0 = new TransactionMessage({
      payerKey: accounts.manager,
      recentBlockhash,
      instructions: [modifyComputeUnitsIx, addPriorityFeeIx, instruction],
    }).compileToV0Message(
      await getAddressLookupTables(
        this.provider.connection,
        swapIxsResponse.addressLookupTableAddresses
      )
    );
    const tx = new VersionedTransaction(messageV0);
    tx.sign(signers);
    return await this.provider.sendAndConfirm(tx, signers, {
      ...DEFAULT_RETRY_CONFIG,
      skipPreflight: true,
      maxRetries: 3, // Internal retries for network issues
    });
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
    signers: (Keypair | Signer)[] = [],
    retryConfig?: TransactionRetryConfig
  ): Promise<string> {
    const transaction = new Transaction();
    transaction.add(...instructions);
    return sendAndConfirmWithRetry(
      this.provider,
      transaction,
      signers,
      retryConfig
    );
  }
}
