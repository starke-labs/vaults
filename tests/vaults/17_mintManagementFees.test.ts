import { Connection, Keypair } from "@solana/web3.js";
import { expect } from "chai";
import { VaultsSdk } from "@starke/sdk";
import {
	createConnection,
	getManagerKeypair,
} from "../utils.new";

describe("mintManagementFees", () => {
	let connection: Connection;
	let vaults: VaultsSdk;
	let manager: Keypair;
	before(async () => {
		connection = createConnection();
		manager = getManagerKeypair();
		vaults = new VaultsSdk(connection, manager);
	});

	it("should be able to collect management fees", async () => {
		await vaults.mintManagementFees(manager.publicKey, [manager]);
		expect(true).to.be.true;
	});
});
