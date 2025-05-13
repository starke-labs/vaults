import { VaultsSdk } from "@starke/sdk";
import {
  getManagerWhitelistPda,
  getTokenWhitelistPda,
  getVaultPda,
} from "@starke/sdk/lib/pdas";

import {
  createConnection,
  getAuthorityKeypair,
  getManager2Keypair,
  getManagerKeypair,
  getTesterKeypair,
} from "../utils.new";

describe("Vaults Info", () => {
  const connection = createConnection();

  const authority = getAuthorityKeypair();
  const tester = getTesterKeypair();
  const manager = getManagerKeypair();
  const manager2 = getManager2Keypair();

  const managerVaults = new VaultsSdk(connection, manager);
  const manager2Vaults = new VaultsSdk(connection, manager2);

  const LAMPORTS_PER_SOL = 10 ** 9;

  it("users balances", async () => {
    const authoritySolBalance = await connection.getBalance(
      authority.publicKey
    );

    const testerSolBalance = await connection.getBalance(tester.publicKey);
    const managerSolBalance = await connection.getBalance(manager.publicKey);
    const manager2SolBalance = await connection.getBalance(manager2.publicKey);

    console.log("--------------------------------");
    console.log("authority", authority.publicKey.toBase58());
    console.log(
      "authoritySolBalance",
      authoritySolBalance / LAMPORTS_PER_SOL,
      "SOL"
    );
    console.log("--------------------------------");

    console.log("tester", tester.publicKey.toBase58());
    console.log("testerSolBalance", testerSolBalance / LAMPORTS_PER_SOL, "SOL");
    console.log("--------------------------------");

    console.log("manager", manager.publicKey.toBase58());
    console.log(
      "managerSolBalance",
      managerSolBalance / LAMPORTS_PER_SOL,
      "SOL"
    );
    console.log("--------------------------------");

    console.log("manager2", manager2.publicKey.toBase58());
    console.log(
      "manager2SolBalance",
      manager2SolBalance / LAMPORTS_PER_SOL,
      "SOL"
    );
    console.log("--------------------------------");
  });

  it("token whitelist", async () => {
    const [tokenWhitelistPda] = getTokenWhitelistPda();
    const tokenWhitelist = await managerVaults.fetchTokenWhitelist();

    console.log("--------------------------------");
    console.log("token whitelist", tokenWhitelistPda.toBase58());
    console.log("--------------------------------");
    tokenWhitelist.tokens.forEach((token) => {
      console.log("mint", token.mint.toBase58());
      console.log("price feed id", token.priceFeedId);
      console.log("--------------------------------");
    });
  });

  it("manager whitelist", async () => {
    const [managerWhitelistPda] = getManagerWhitelistPda();
    const managerWhitelist = await managerVaults.fetchManagerWhitelist();

    console.log("--------------------------------");
    console.log("manager whitelist", managerWhitelistPda.toBase58());
    console.log("--------------------------------");
    managerWhitelist.managers.forEach((manager) => {
      console.log(manager.toBase58());
      console.log("--------------------------------");
    });
  });

  it("vault by manager", async () => {
    const [vaultPda] = getVaultPda(manager.publicKey);
    const vault = await managerVaults.fetchVault(manager.publicKey);

    console.log("--------------------------------");
    console.log("vault", vaultPda.toBase58());
    console.log("name", vault.name);
    console.log("manager", vault.manager.toBase58());
    console.log("vtoken", vault.mint.toBase58());
    console.log("--------------------------------");
  });

  it("vault by manager 2", async () => {
    const [vaultPda] = getVaultPda(manager2.publicKey);
    const vault = await manager2Vaults.fetchVault(manager2.publicKey);

    console.log("--------------------------------");
    console.log("vault", vaultPda.toBase58());
    console.log("name", vault.name);
    console.log("manager", vault.manager.toBase58());
    console.log("vtoken", vault.mint.toBase58());
    console.log("--------------------------------");
  });
});
