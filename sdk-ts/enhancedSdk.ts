import { AnchorProvider, BN, Program, Wallet } from "@coral-xyz/anchor";
import {
  IBackpackSolanaSigner,
  ISolanaSigner,
} from "@dynamic-labs/solana-core";
import { DefaultApi, createJupiterApiClient } from "@jup-ag/api";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import {
  ComputeBudgetProgram,
  Connection,
  Keypair,
  PublicKey,
  Signer,
  Transaction,
  TransactionMessage,
  TransactionSignature,
  VersionedTransaction,
} from "@solana/web3.js";

import idl from "@starke/idl/vaults.json";

import { EventHandler } from "./events";
import {
  AccountNotInitializedError,
  DepositBelowMinimumError,
  InsufficientBalanceError,
  InvalidAmountError,
  InvalidTokenError,
  InvestorTypeNotAllowedError,
  ManagerAlreadyInWhitelistError,
  ManagerNotWhitelistedError,
  MaxAumExceededError,
  MaxDepositorsExceededError,
  SignatureVerificationFailedError,
  StarkeAlreadyInitializedError,
  StarkeAlreadyPausedError,
  StarkeAlreadyResumedError,
  StarkeNotInitializedError,
  StarkePausedError,
  TokenAlreadyInWhitelistError,
  TokenNotWhitelistedError,
  UserNotWhitelistedError,
  VaultAlreadyCreatedError,
  VaultNotFoundError,
} from "./lib/errors";
import { constructSwapInstruction } from "./lib/jupiter";
import {
  AUTHORITY_PROGRAM_ID,
  getManagerWhitelistPda,
  getStarkeConfigPda,
  getTokenWhitelistPda,
  getUserWhitelistPda,
  getVaultPda,
  getVtokenConfigPda,
  getVtokenMetadataPda,
  getVtokenMintPda,
} from "./lib/pdas";
import {
  KeypairSignerAdapter,
  createSolanaSigner,
  isKeypair,
  isSolanaSigner,
} from "./lib/signerAdapter";
import { getAddressLookupTables } from "./lib/solana";
import {
  DEFAULT_RETRY_CONFIG,
  sendAndConfirmWithRetry,
} from "./lib/transaction";
import {
  AccountMeta,
  InvestorType,
  ManagerWhitelist,
  StarkeConfig,
  Token,
  TokenWhitelist,
  UserWhitelist,
  Vault,
} from "./lib/types";

// Wallet interface that works with both Keypair and wallet signers
interface WalletInterface {
  publicKey: PublicKey;
  signTransaction<T extends VersionedTransaction | Transaction>(
    transaction: T
  ): Promise<T>;
  signAllTransactions<T extends VersionedTransaction | Transaction>(
    transactions: T[]
  ): Promise<T[]>;
}

// Adapter to make ISolanaSigner work with Anchor's Wallet interface
class SolanaSignerWalletAdapter implements WalletInterface {
  private signer: ISolanaSigner;

  constructor(signer: ISolanaSigner) {
    this.signer = signer;
  }

  get publicKey(): PublicKey {
    if (!this.signer.publicKey) {
      throw new Error("Wallet not connected or public key not available");
    }
    return new PublicKey(this.signer.publicKey.toBytes());
  }

  async signTransaction<T extends VersionedTransaction | Transaction>(
    transaction: T
  ): Promise<T> {
    return await this.signer.signTransaction(transaction);
  }

  async signAllTransactions<T extends VersionedTransaction | Transaction>(
    transactions: T[]
  ): Promise<T[]> {
    return await this.signer.signAllTransactions(transactions);
  }
}

/**
 * Enhanced Vaults SDK with wallet adapter support
 *
 * This SDK supports multiple signer types through a common interface:
 * - Standard Solana Keypair (built-in)
 * - Wallet adapters implementing ISolanaSigner (Dynamic.xyz, Phantom, etc.)
 * - Extended wallet signers (IBackpackSolanaSigner)
 *
 * The SDK provides interfaces and reference implementations for wallet integration
 * but does not include wallet SDKs as dependencies, keeping it lightweight.
 *
 * Example with Dynamic.xyz (requires Dynamic packages in your app):
 * ```typescript
 * // In your app: npm install @dynamic-labs/sdk-react-core @dynamic-labs/solana
 * import { useUserWallets } from '@dynamic-labs/sdk-react-core';
 * import { EnhancedVaultsSdk, DynamicWalletAdapter } from '@starke/vaults-sdk';
 *
 * const userWallets = useUserWallets();
 * const solanaWallet = userWallets.find(w => w.chain === 'SOL');
 *
 * if (solanaWallet?.connector) {
 *   const adapter = new DynamicWalletAdapter(solanaWallet.connector);
 *   const sdk = new EnhancedVaultsSdk(connection, adapter);
 *   await sdk.connectWallet();
 * }
 * ```
 */
export class EnhancedVaultsSdk {
  private program: Program;
  private provider: AnchorProvider;
  private signer: ISolanaSigner | Keypair;
  private connection: Connection;
  public events: EventHandler;

  // Jup API client
  private jup: DefaultApi;

  constructor(
    connection: Connection,
    signer: Keypair | ISolanaSigner | IBackpackSolanaSigner
  ) {
    this.connection = connection;
    this.signer = signer;

    // Create appropriate wallet adapter
    let walletAdapter: WalletInterface;

    if (isKeypair(signer)) {
      // For Keypair, use the standard Anchor Wallet
      walletAdapter = new Wallet(signer);
    } else if (isSolanaSigner(signer)) {
      // For ISolanaSigner, use our adapter
      walletAdapter = new SolanaSignerWalletAdapter(signer);
    } else {
      throw new Error("Invalid signer type provided");
    }

    this.provider = new AnchorProvider(
      connection,
      walletAdapter,
      AnchorProvider.defaultOptions()
    );

    this.program = new Program(idl, this.provider);
    this.events = new EventHandler(connection, this.program.programId);

    // Jup API client
    this.jup = createJupiterApiClient();
  }

  /**
   * Get the signer instance (Keypair or ISolanaSigner)
   */
  public getSigner(): ISolanaSigner | Keypair {
    return this.signer;
  }

  /**
   * Get the public key of the current signer
   */
  public getPublicKey(): PublicKey {
    if (isKeypair(this.signer)) {
      return this.signer.publicKey;
    } else {
      if (!this.signer.publicKey) {
        throw new Error("Wallet not connected or public key not available");
      }
      return new PublicKey(this.signer.publicKey.toBytes());
    }
  }

  /**
   * Check if the signer supports Dynamic.xyz wallet interface
   */
  public isWalletSigner(): boolean {
    return isSolanaSigner(this.signer);
  }

  /**
   * Connect wallet (only applicable for ISolanaSigner)
   */
  public async connectWallet(args?: { onlyIfTrusted: boolean }): Promise<void> {
    if (isSolanaSigner(this.signer)) {
      await this.signer.connect(args);
    }
    // Keypair is always "connected"
  }

  /**
   * Disconnect wallet (only applicable for ISolanaSigner)
   */
  public async disconnectWallet(): Promise<void> {
    if (isSolanaSigner(this.signer)) {
      await this.signer.disconnect();
    }
    // Keypair doesn't need disconnection
  }

  /**
   * Sign a message using the current signer
   */
  public async signMessage(
    message: Uint8Array
  ): Promise<{ signature: Uint8Array; message: Uint8Array }> {
    if (isSolanaSigner(this.signer)) {
      const result = await this.signer.signMessage(message);
      return { signature: result.signature, message };
    } else {
      // For Keypair, use tweetnacl for message signing
      const { sign } = await import("tweetnacl");
      const signature = sign.detached(message, this.signer.secretKey);
      return { signature, message };
    }
  }

  /**
   * Create a compatible signer from various inputs
   */
  public static createSigner(
    signerInput: Keypair | ISolanaSigner,
    connection: Connection,
    options?: { supportsBackpack?: boolean }
  ): ISolanaSigner {
    return createSolanaSigner(
      signerInput,
      connection,
      options?.supportsBackpack || false
    );
  }

  // Accounts
  async fetchStarkeConfig(): Promise<StarkeConfig> {
    try {
      // @ts-ignore
      return (await this.program.account.starkeConfig.fetch(
        getStarkeConfigPda()[0]
      )) as StarkeConfig;
    } catch (e) {
      if (e.toString().includes("Account does not exist")) {
        throw new StarkeNotInitializedError();
      }
      throw e;
    }
  }

  async fetchTokenWhitelist(): Promise<TokenWhitelist> {
    try {
      // @ts-ignore
      return (await this.program.account.tokenWhitelist.fetch(
        getTokenWhitelistPda()[0]
      )) as TokenWhitelist;
    } catch (e) {
      if (e.toString().includes("Account does not exist")) {
        throw new StarkeNotInitializedError();
      }
      throw e;
    }
  }

  async fetchManagerWhitelist(): Promise<ManagerWhitelist> {
    try {
      // @ts-ignore
      return (await this.program.account.managerWhitelist.fetch(
        getManagerWhitelistPda()[0]
      )) as ManagerWhitelist;
    } catch (e) {
      if (e.toString().includes("Account does not exist")) {
        throw new StarkeNotInitializedError();
      }
      throw e;
    }
  }

  async fetchUserWhitelist(): Promise<UserWhitelist> {
    try {
      // @ts-ignore
      return (await this.program.account.userWhitelist.fetch(
        getUserWhitelistPda()[0]
      )) as UserWhitelist;
    } catch (e) {
      if (e.toString().includes("Account does not exist")) {
        throw new StarkeNotInitializedError();
      }
      throw e;
    }
  }

  async fetchWhitelistedToken(mint: PublicKey): Promise<Token> {
    const whitelist = await this.fetchTokenWhitelist();
    const token = whitelist.tokens.find(
      (token) => token.mint.toBase58() === mint.toBase58()
    );
    if (!token) {
      throw new TokenNotWhitelistedError(mint);
    }
    return token;
  }

  async fetchVault(manager: PublicKey): Promise<Vault> {
    try {
      // @ts-ignore
      return (await this.program.account.vault.fetch(
        getVaultPda(manager)[0]
      )) as Vault;
    } catch (e) {
      if (e.toString().includes("Account does not exist")) {
        throw new VaultNotFoundError(manager);
      }
    }
  }

  // Admin instructions
  async initializeStarke(
    signers: (Keypair | Signer)[]
  ): Promise<TransactionSignature> {
    // Check if starke config is already initialized
    try {
      const starkeConfig = await this.fetchStarkeConfig();
      if (starkeConfig) {
        throw new StarkeAlreadyInitializedError();
      }
    } catch (e) {
      if (!(e instanceof StarkeNotInitializedError)) {
        throw e;
      }
    }

    const tx = await this.program.methods
      .initializeStarke()
      .accounts({
        authority: AUTHORITY_PROGRAM_ID,
      })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Missing signature")) {
        throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
      }
      throw e;
    }
  }

  async pauseStarke(
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const starkeConfig = await this.fetchStarkeConfig();
    if (starkeConfig.isPaused) {
      throw new StarkeAlreadyPausedError();
    }

    const tx = await this.program.methods.pauseStarke().transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
      }
      throw e;
    }
  }

  async resumeStarke(
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const starkeConfig = await this.fetchStarkeConfig();
    if (!starkeConfig.isPaused) {
      throw new StarkeAlreadyResumedError();
    }

    const tx = await this.program.methods.resumeStarke().transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
      }
      throw e;
    }
  }

  async addTokenToWhitelist(
    supportedToken: Token,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const whitelist = await this.fetchTokenWhitelist();
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
      .accounts({ authority: AUTHORITY_PROGRAM_ID })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
      }
      throw e;
    }
  }

  async removeTokenFromWhitelist(
    mint: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const whitelist = await this.fetchTokenWhitelist();
    const tokenInWhitelist = whitelist.tokens.find(
      (t) => t.mint.toBase58() === mint.toBase58()
    );
    if (!tokenInWhitelist) {
      throw new TokenNotWhitelistedError(mint);
    }

    const tx = await this.program.methods
      .removeToken(mint)
      .accounts({ authority: AUTHORITY_PROGRAM_ID })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
      }
      throw e;
    }
  }

  async addManagerToWhitelist(
    manager: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const whitelist = await this.fetchManagerWhitelist();
    const managerInWhitelist = whitelist.managers.find(
      (m) => m.toBase58() === manager.toBase58()
    );
    if (managerInWhitelist) {
      throw new ManagerAlreadyInWhitelistError(manager);
    }

    const tx = await this.program.methods
      .addManager(manager)
      .accounts({ authority: AUTHORITY_PROGRAM_ID })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
      }
      throw e;
    }
  }

  async removeManagerFromWhitelist(
    manager: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const whitelist = await this.fetchManagerWhitelist();
    const managerInWhitelist = whitelist.managers.find(
      (m) => m.toBase58() === manager.toBase58()
    );
    if (!managerInWhitelist) {
      throw new ManagerNotWhitelistedError(manager);
    }

    const tx = await this.program.methods
      .removeManager(manager)
      .accounts({ authority: AUTHORITY_PROGRAM_ID })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
      }
      throw e;
    }
  }

  async addUserToWhitelist(
    user: PublicKey,
    investorType: InvestorType,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const tx = await this.program.methods
      .addUser(user, investorType)
      .accounts({ starkeAuthority: AUTHORITY_PROGRAM_ID })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
      }
      throw e;
    }
  }

  async removeUserFromWhitelist(
    user: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const tx = await this.program.methods
      .removeUser(user)
      .accounts({ starkeAuthority: AUTHORITY_PROGRAM_ID })
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      if (e.toString().includes("Signature verification failed")) {
        throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
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

  // Manager instructions
  async createVault(
    name: string,
    symbol: string,
    uri: string,
    manager: PublicKey,
    depositTokenMint: PublicKey,
    isVtokenTransferrable: boolean,
    maxAllowedAum: BN | null,
    allowRetail: boolean,
    allowAccredited: boolean,
    allowInstitutional: boolean,
    allowQualified: boolean,
    individualMinDeposit?: number, // u32, 0 = no minimum, optional defaults to 0
    institutionalMinDeposit?: number, // u32, 0 = no minimum, optional defaults to 0
    maxDepositors?: number, // u32, 0 = unlimited, optional defaults to 0
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    // No validation needed - max_allowed_aum can be set or not set for any vault

    const tokenProgram = await this.getTokenProgram(depositTokenMint);

    // Set defaults for optional parameters (0 = no limit/minimum)
    const finalIndividualMinDeposit = individualMinDeposit ?? 0;
    const finalInstitutionalMinDeposit = institutionalMinDeposit ?? 0;
    const finalMaxDepositors = maxDepositors ?? 0;

    const tx = await this.program.methods
      .createVault(
        name,
        symbol,
        uri,
        isVtokenTransferrable,
        maxAllowedAum,
        allowRetail,
        allowAccredited,
        allowInstitutional,
        allowQualified,
        finalIndividualMinDeposit,
        finalInstitutionalMinDeposit,
        finalMaxDepositors
      )
      .accounts({
        manager,
        depositTokenMint,
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
        throw new SignatureVerificationFailedError(
          this.provider.wallet.publicKey
        );
      } else if (e.toString().includes("already in use")) {
        const [vault] = getVaultPda(this.provider.wallet.publicKey);
        throw new VaultAlreadyCreatedError(vault);
      } else if (
        e
          .toString()
          .includes(
            "Attempt to debit an account but found no record of a prior credit"
          )
      ) {
        throw new InsufficientBalanceError(this.provider.wallet.publicKey);
      } else if (e.toString().includes("Token is not whitelisted")) {
        throw new TokenNotWhitelistedError(depositTokenMint);
      } else if (e.toString().includes("Manager is not whitelisted")) {
        throw new ManagerNotWhitelistedError(this.provider.wallet.publicKey);
      } else if (
        e
          .toString()
          .includes(
            "caused by account: vtoken_config. Error Code: AccountNotInitialized"
          )
      ) {
        throw new AccountNotInitializedError("vtoken_config");
      } else if (e.toString().includes("StarkePaused")) {
        throw new StarkePausedError();
      }
      throw e;
    }
  }

  async closeVault(
    manager: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    // Get vault and validate it exists
    const vault = await this.fetchVault(manager);

    const tokenProgram = await this.getTokenProgram(vault.depositTokenMint);
    const tx = await this.program.methods
      .closeVault()
      .accounts({
        manager,
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
      } else if (
        e.toString().includes("VaultHasActiveDepositors") ||
        e.toString().includes("Cannot close vault with active depositors")
      ) {
        throw new Error("Cannot close vault: vault has active depositors");
      } else if (
        e.toString().includes("VTokensOutstanding") ||
        e.toString().includes("Cannot close vault with outstanding vtokens")
      ) {
        throw new Error("Cannot close vault: outstanding vtokens exist");
      } else if (
        e.toString().includes("FundsRemaining") ||
        e.toString().includes("Cannot close vault with remaining funds")
      ) {
        throw new Error("Cannot close vault: funds remaining in vault");
      }
      throw e;
    }
  }

  async swapOnJupiter(
    inputMint: PublicKey,
    outputMint: PublicKey,
    amount: BN,
    manager: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const quoteResponse = await this.jup.quoteGet({
      inputMint: inputMint.toBase58(),
      outputMint: outputMint.toBase58(),
      amount: amount.toNumber(),
    });

    const [vault] = getVaultPda(manager);

    const swapIxsResponse = await this.jup.swapInstructionsPost({
      swapRequest: {
        userPublicKey: vault.toBase58(),
        quoteResponse,
      },
    });

    const swapInstruction = constructSwapInstruction(
      swapIxsResponse.swapInstruction
    );

    const instruction = await this.program.methods
      .swapOnJupiter(swapInstruction.data)
      .accounts({
        manager,
        inputTokenMint: inputMint,
        outputTokenMint: outputMint,
        tokenProgram: await this.getTokenProgram(outputMint),
      })
      .remainingAccounts(swapInstruction.keys)
      .instruction();

    const modifyComputeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 1_400_000,
    });

    const addPriorityFeeIx = ComputeBudgetProgram.setComputeUnitPrice({
      // Does not work with 50k micro lamports, works with 100k
      microLamports: 150_000,
    });

    let recentBlockhash = (await this.provider.connection.getLatestBlockhash())
      .blockhash;

    const messageV0 = new TransactionMessage({
      payerKey: manager,
      recentBlockhash,
      instructions: [modifyComputeUnitsIx, addPriorityFeeIx, instruction],
    }).compileToV0Message(
      await getAddressLookupTables(
        this.provider.connection,
        swapIxsResponse.addressLookupTableAddresses
      )
    );

    return await this.provider.sendAndConfirm(
      new VersionedTransaction(messageV0),
      signers,
      {
        ...DEFAULT_RETRY_CONFIG,
        skipPreflight: true,
        maxRetries: 3, // Internal retries for network issues
      }
    );
  }

  // User instructions
  private async getDepositRemainingAccounts(
    vault: PublicKey,
    whitelistedTokens: Token[]
  ): Promise<AccountMeta[]> {
    const tokenAccounts =
      await this.provider.connection.getParsedTokenAccountsByOwner(vault, {
        programId: TOKEN_PROGRAM_ID,
      });
    const token2022Accounts =
      await this.provider.connection.getParsedTokenAccountsByOwner(vault, {
        programId: TOKEN_2022_PROGRAM_ID,
      });

    const remainingAccounts: AccountMeta[] = [];
    for (const tokenAccount of [
      ...tokenAccounts.value,
      ...token2022Accounts.value,
    ]) {
      const tokenMint = new PublicKey(
        tokenAccount.account.data.parsed.info.mint
      );
      const token = whitelistedTokens.find(
        (token) => token.mint.toBase58() === tokenMint.toBase58()
      );
      if (!token) continue;
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

    return remainingAccounts;
  }

  async deposit(
    amount: BN,
    depositor: PublicKey,
    manager: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const whitelistedTokens = (await this.fetchTokenWhitelist()).tokens;
    const remainingAccounts = await this.getDepositRemainingAccounts(
      getVaultPda(manager)[0],
      whitelistedTokens
    );

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
      } else if (e.toString().includes("DepositBelowMinimum")) {
        // Extract amount info if possible from error message
        throw new DepositBelowMinimumError(amount.toString(), "minimum");
      } else if (e.toString().includes("MaxAumExceeded")) {
        throw new MaxAumExceededError("current", "max");
      } else if (e.toString().includes("InvalidAmount")) {
        throw new InvalidAmountError();
      } else if (e.toString().includes("UserNotWhitelisted")) {
        throw new UserNotWhitelistedError(depositor);
      } else if (e.toString().includes("InvestorTypeNotAllowed")) {
        throw new InvestorTypeNotAllowedError("unknown");
      } else if (e.toString().includes("MaxDepositorsExceeded")) {
        throw new MaxDepositorsExceededError();
      }
      throw e;
    }
  }

  async getVtokenBalance(vault: PublicKey, user: PublicKey): Promise<BN> {
    const [vtokenMint] = getVtokenMintPda(vault);
    const vtokenAta = await getAssociatedTokenAddress(
      vtokenMint,
      user,
      false,
      TOKEN_2022_PROGRAM_ID
    );
    const vtokenBalance = await this.provider.connection.getTokenAccountBalance(
      vtokenAta
    );
    return new BN(vtokenBalance.value.amount);
  }

  private async getWithdrawRemainingAccounts(
    vault: PublicKey,
    withdrawer: PublicKey,
    whitelistedTokens: Token[]
  ): Promise<AccountMeta[]> {
    const tokenAccounts =
      await this.provider.connection.getParsedTokenAccountsByOwner(vault, {
        programId: TOKEN_PROGRAM_ID,
      });
    const token2022Accounts =
      await this.provider.connection.getParsedTokenAccountsByOwner(vault, {
        programId: TOKEN_2022_PROGRAM_ID,
      });

    const remainingAccounts: AccountMeta[] = [];
    let programId = TOKEN_PROGRAM_ID;
    for (const vaultTokenAccount of [
      ...tokenAccounts.value,
      null,
      ...token2022Accounts.value,
    ]) {
      if (vaultTokenAccount === null) {
        programId = TOKEN_2022_PROGRAM_ID;
        continue;
      }

      const tokenMint = new PublicKey(
        vaultTokenAccount.account.data.parsed.info.mint
      );
      const token = whitelistedTokens.find(
        (token) => token.mint.toBase58() === tokenMint.toBase58()
      );
      if (!token) continue;
      const userTokenAccount = await getAssociatedTokenAddress(
        tokenMint,
        withdrawer,
        false,
        programId
      );

      remainingAccounts.push(
        {
          pubkey: tokenMint,
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: vaultTokenAccount.pubkey,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: userTokenAccount,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: programId,
          isWritable: false,
          isSigner: false,
        }
      );
    }

    return remainingAccounts;
  }

  async withdraw(
    amount: BN,
    withdrawer: PublicKey,
    manager: PublicKey,
    signers: (Keypair | Signer)[] = []
  ): Promise<TransactionSignature> {
    const whitelistedTokens = (await this.fetchTokenWhitelist()).tokens;
    const remainingAccounts = await this.getWithdrawRemainingAccounts(
      getVaultPda(manager)[0],
      withdrawer,
      whitelistedTokens
    );

    const tx = await this.program.methods
      .withdraw(amount)
      .accounts({
        user: withdrawer,
        manager,
      })
      .remainingAccounts(remainingAccounts)
      .signers(signers)
      .transaction();

    try {
      return await sendAndConfirmWithRetry(this.provider, tx, signers);
    } catch (e) {
      // console.log(e);
      if (e.toString().includes("Signature verification failed")) {
        if (e.toString().includes(withdrawer.toBase58())) {
          throw new SignatureVerificationFailedError(withdrawer);
        } else if (e.toString().includes(AUTHORITY_PROGRAM_ID.toBase58())) {
          throw new SignatureVerificationFailedError(AUTHORITY_PROGRAM_ID);
        }
      } else if (e.toString().includes("insufficient funds")) {
        throw new InsufficientBalanceError(withdrawer);
      }
      throw e;
    }
  }
}
