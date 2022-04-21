/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-return */

import os from "os";
import fs from "mz/fs";
import path from "path";
import yaml from "yaml";
import { Keypair } from "@solana/web3.js";
// import * as BufferLayout from "@solana/buffer-layout";
import * as buffer from "buffer";
/**
 * @private
 */
export const getConfig = async () => {
  // Path to Solana CLI config file
  const CONFIG_FILE_PATH = path.resolve(
    os.homedir(),
    ".config",
    "solana",
    "cli",
    "config.yml"
  );
  const configYml = await fs.readFile(CONFIG_FILE_PATH, { encoding: "utf8" });
  return yaml.parse(configYml);
};

/**
 * Load and parse the Solana CLI config file to determine which RPC url to use
 */
export const getRpcUrl = async () => {
  try {
    const config = await getConfig();
    if (!config.json_rpc_url) throw new Error("Missing RPC URL");
    return config.json_rpc_url;
  } catch (err) {
    console.warn(
      "Failed to read RPC url from CLI config file, falling back to localhost"
    );
    return "http://localhost:8899";
  }
};

/**
 * Load and parse the Solana CLI config file to determine which payer to use
 */
export const getPayer = async () => {
  try {
    const config = await getConfig();
    if (!config.keypair_path) throw new Error("Missing keypair path");
    return await createKeypairFromFile(config.keypair_path);
  } catch (err) {
    console.warn(
      "Failed to create keypair from CLI config file, falling back to new random keypair"
    );
    return Keypair.generate();
  }
};

/**
 * Create a Keypair from a secret key stored in file as bytes' array
 */
export const createKeypairFromFile = async (filePath: string) => {
  const secretKeyString = await fs.readFile(filePath, { encoding: "utf8" });
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
};


// /**
//  * Layout for a public key
//  */
//  const publicKey = (property = "publicKey") => {
//   return BufferLayout.blob(32, property);
// };

// /**
//  * Layout for a 64bit unsigned value
//  */
// const uint64 = (property = "uint64") => {
//   return BufferLayout.blob(8, property);
// };

// /**
//  * Layout for WPokt state struct
//  */
// export const  W_POKT_ACCOUNT_DATA_LAYOUT = BufferLayout.struct([
//   BufferLayout.u8("isInitialized"),
//   publicKey("bridgeAddress")
// ]);
// // export const W_POKT_ACCOUNT_DATA_LAYOUT = BufferLayout.struct([
// //   BufferLayout.u8("isInitialized"),
// //   publicKey("bridgeAddress"),
// //   publicKey("owner"),
// //   publicKey("mint"),
// // ]);

// Flexible class that takes properties and imbues them
// to the object instance
class Assignable {
  constructor(properties) {
      Object.keys(properties).map((key) => {
          return (this[key] = properties[key]);
      });
  }
}

export class WPoktData extends Assignable { }

const dataSchema = new Map([
  [
    WPoktData,
      {
          kind: "struct",
          fields: [
              ["initialized", "u8"],
              ["tree_length", "u32"],
              ["map", { kind: 'map', key: 'string', value: 'string' }]
          ]
      }
  ]
]);

// /**
// * Fetch program account data
// * @param {Connection} connection - Solana RPC connection
// * @param {PublicKey} account - Public key for account whose data we want
// * @return {Promise<AccoundData>} - Keypair
// */
// export async function getAccountData(connection: Connection, account: PublicKey): Promise<AccoundData> {
//   let nameAccount = await connection.getAccountInfo(
//       account,
//       'processed'
//   );
//   return deserializeUnchecked(dataSchema, AccoundData, nameAccount.data)
// }

// /**
//  * WPokt state account interface
//  */
// export interface WPoktLayout {
//   isInitialized: number;
//   bridgeAddress: Uint8Array;
//   owner: Uint8Array;
//   mint: Uint8Array;
// }
