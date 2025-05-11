import {
  AddressLookupTableAccount,
  Connection,
  PublicKey,
} from "@solana/web3.js";

// TODO: Consider moving this to the transaction.ts file
export async function getAddressLookupTables(
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
