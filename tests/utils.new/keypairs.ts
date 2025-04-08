import { Keypair } from "@solana/web3.js";
import fs from "fs";

export function getAuthorityKeypair(): Keypair {
  return Keypair.fromSecretKey(
    new Uint8Array(
      JSON.parse(fs.readFileSync("./deploy/authority.json", "utf8"))
    )
  );
}

export function getManagerKeypair(): Keypair {
  return Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync("./deploy/tester.json", "utf8")))
  );
}

export function getDeployerKeypair(): Keypair {
  return Keypair.fromSecretKey(
    new Uint8Array(
      JSON.parse(fs.readFileSync("./deploy/deployer.json", "utf8"))
    )
  );
}

export function getTesterKeypair(): Keypair {
  return Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync("./deploy/tester2.json", "utf8")))
  );
}
