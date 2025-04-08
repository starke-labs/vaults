import { AnchorProvider, BN, Idl, Program, Wallet } from "@coral-xyz/anchor";
import { DefaultApi, Instruction, createJupiterApiClient } from "@jup-ag/api";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import {
  AddressLookupTableAccount,
  ComputeBudgetProgram,
  Connection,
  Keypair,
  PublicKey,
  Signer,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";

import idl from "@starke/idl/vaults.json";

import { EventHandler } from "./events";
import {
  AccountNotInitializedError,
  InsufficientBalanceError,
  InvalidTokenError,
  SignatureVerificationFailedError,
  TokenAlreadyInWhitelistError,
  TokenNotWhitelistedError,
  VaultAlreadyCreatedError,
  VaultNotFoundError,
  WhitelistAlreadyInitializedError,
  WhitelistNotInitializedError,
} from "./lib/errors";
import {
  AUTHORITY_PROGRAM_ID,
  getVaultPda,
  getVtokenMetadataPda,
  getVtokenMintPda,
  getWhitelistPda,
} from "./lib/pdas";
import { Token, Vault, Whitelist } from "./lib/types";
import {
  AddTokenAccounts,
  AddTokenParams,
  CreateVaultAccounts,
  CreateVaultParams,
  DepositAccounts,
  DepositParams,
  RemoveTokenAccounts,
  RemoveTokenParams,
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

  constructor(connection: Connection, keypair: Keypair) {
    this.provider = new AnchorProvider(
      connection,
      new Wallet(keypair),
      AnchorProvider.defaultOptions()
    );
    this.program = new Program(idl, this.provider);
    this.events = new EventHandler(connection, this.program.programId);

    // Jup API client
    this.jup = createJupiterApiClient();
  }

  // *** New methods ***
  async fetchWhitelist(): Promise<Whitelist> {
    try {
      // @ts-ignore
      return (await this.program.account.tokenWhitelist.fetch(
        getWhitelistPda()[0]
      )) as Whitelist;
    } catch (e) {
      if (e.toString().includes("Account does not exist")) {
        throw new WhitelistNotInitializedError();
      }
      throw e;
    }
  }

  async fetchWhitelistedTokens(mint: PublicKey): Promise<Token> {
    const whitelist = await this.fetchWhitelist();
    const token = whitelist.tokens.find(
      (token) => token.mint.toBase58() === mint.toBase58()
    );
    if (!token) {
      throw new TokenNotWhitelistedError(mint);
    }
    return token;
  }

  async initializeWhitelist(signers: (Keypair | Signer)[]): Promise<string> {
    // Check if whitelist is already initialized
    try {
      await this.fetchWhitelist();
      throw new WhitelistAlreadyInitializedError(getWhitelistPda()[0]);
    } catch (e) {
      if (!(e instanceof WhitelistNotInitializedError)) {
        throw e;
      }
    }

    const tx = await this.program.methods
      .initializeWhitelist()
      .accounts({
        authority: AUTHORITY_PROGRAM_ID,
      })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Missing signature")) {
        throw new SignatureVerificationFailedError(getWhitelistPda()[0]);
      }
      throw e;
    }
  }

  async addTokenToWhitelist(
    supportedToken: Token,
    signers: (Keypair | Signer)[] = []
  ): Promise<string> {
    const whitelist = await this.fetchWhitelist();
    const tokenInWhitelist = whitelist.tokens.find(
      (token) => token.mint.toBase58() === supportedToken.mint.toBase58()
    );
    if (tokenInWhitelist) {
      throw new TokenAlreadyInWhitelistError(supportedToken.mint);
    }

    const tx = await this.program.methods
      .addToken(
        supportedToken.mint,
        supportedToken.priceFeedId,
        supportedToken.priceUpdate
      )
      .accounts({ authority: whitelist.authority })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(whitelist.authority);
      }
      throw e;
    }
  }

  async removeTokenFromWhitelist(
    mint: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<string> {
    const whitelist = await this.fetchWhitelist();
    const tokenInWhitelist = whitelist.tokens.find(
      (t) => t.mint.toBase58() === mint.toBase58()
    );
    if (!tokenInWhitelist) {
      throw new TokenNotWhitelistedError(mint);
    }

    const tx = await this.program.methods
      .removeToken(mint)
      .accounts({ authority: whitelist.authority })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(whitelist.authority);
      }
      throw e;
    }
  }

  async getTokenProgram(mint: PublicKey): Promise<PublicKey> {
    // TODO: Should we check this at program level?
    const tokenProgram = (await this.provider.connection.getAccountInfo(mint))
      ?.owner;
    if (
      !tokenProgram ||
      ![TOKEN_PROGRAM_ID.toBase58(), TOKEN_2022_PROGRAM_ID.toBase58()].includes(
        tokenProgram.toBase58()
      )
    ) {
      throw new InvalidTokenError(mint);
    }
    return tokenProgram;
  }

  async createVault(
    name: string,
    symbol: string,
    uri: string,
    entryFee: number,
    exitFee: number,
    manager: PublicKey,
    depositTokenMint: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<string> {
    const [vault] = getVaultPda(manager);
    const [vtokenMint] = getVtokenMintPda(vault);
    const [metadata] = getVtokenMetadataPda(vtokenMint);
    const tokenProgram = await this.getTokenProgram(depositTokenMint);

    const tx = await this.program.methods
      .createVault(name, symbol, uri, entryFee, exitFee)
      .accounts({
        manager,
        depositTokenMint,
        metadata,
        tokenProgram,
      })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (
        e.toString().includes("unauthorized") ||
        e.toString().includes("Signature verification failed")
      ) {
        throw new SignatureVerificationFailedError(manager);
      } else if (e.toString().includes("already in use")) {
        throw new VaultAlreadyCreatedError(vault);
      } else if (
        e
          .toString()
          .includes(
            "Attempt to debit an account but found no record of a prior credit"
          )
      ) {
        throw new InsufficientBalanceError(manager);
      } else if (e.toString().includes("Token is not whitelisted")) {
        throw new TokenNotWhitelistedError(depositTokenMint);
      }
      throw e;
    }
  }

  async fetchVault(manager: PublicKey): Promise<Vault> {
    try {
      // @ts-ignore
      return (await this.program.account.vault.fetch(
        getVaultPda(manager)[0]
      )) as Vault;
    } catch (e) {
      console.log(e);
      if (e.toString().includes("Account does not exist")) {
        throw new VaultNotFoundError(manager);
      }
    }
  }

  async deposit(
    amount: BN,
    depositor: PublicKey,
    manager: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<string> {
    // Populate remaining accounts
    const tokenAccounts =
      await this.provider.connection.getParsedTokenAccountsByOwner(
        getVaultPda(manager)[0],
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );
    const token2022Accounts =
      await this.provider.connection.getParsedTokenAccountsByOwner(
        getVaultPda(manager)[0],
        {
          programId: TOKEN_2022_PROGRAM_ID,
        }
      );
    const allTokenAccounts = [
      ...tokenAccounts.value,
      ...token2022Accounts.value,
    ];
    const whitelistedTokens = (await this.fetchWhitelist()).tokens;
    const remainingAccounts: AccountMeta[] = [];

    for (const tokenAccount of allTokenAccounts) {
      const tokenMint = new PublicKey(
        tokenAccount.account.data.parsed.info.mint
      );
      const token = whitelistedTokens.find(
        (token) => token.mint.toBase58() === tokenMint.toBase58()
      );

      remainingAccounts.push(
        {
          pubkey: tokenMint,
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: tokenAccount.pubkey,
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: token.priceUpdate,
          isWritable: false,
          isSigner: false,
        }
      );
    }

    // Deposit token
    const vault = await this.fetchVault(manager);
    const depositTokenFromWhitelist = whitelistedTokens.find(
      (token) => token.mint.toBase58() === vault.depositTokenMint.toBase58()
    );
    if (!depositTokenFromWhitelist) {
      throw new TokenNotWhitelistedError(vault.depositTokenMint);
    }
    const tokenProgram = await this.getTokenProgram(vault.depositTokenMint);

    // Tx
    const tx = await this.program.methods
      .deposit(amount)
      .accounts({
        user: depositor,
        manager,
        authority: AUTHORITY_PROGRAM_ID,
        depositTokenMint: vault.depositTokenMint,
        depositTokenPriceUpdate: depositTokenFromWhitelist.priceUpdate,
        tokenProgram,
      })
      .remainingAccounts(remainingAccounts)
      .signers(signers)
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        if (e.toString().includes(depositor.toBase58())) {
          throw new SignatureVerificationFailedError(depositor);
        } else if (e.toString().includes(AUTHORITY_PROGRAM_ID.toBase58())) {
          throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
        }
      } else if (e.toString().includes("Attempt to debit an account")) {
        throw new InsufficientBalanceError(depositor);
      } else if (
        e.toString().includes("AccountNotInitialized") &&
        e.toString().includes("user_deposit_token_account")
      ) {
        // TODO: Send proper message through this error
        throw new AccountNotInitializedError("user_deposit_token_account");
      } else if (e.toString().includes("Error: insufficient funds")) {
        throw new InsufficientBalanceError(depositor);
      }
      throw e;
    }
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

  // Transaction methods
  // TODO: This should be removed once the SDK is updated to use the new methods
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
