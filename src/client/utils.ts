/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-return */

import os from "os";
import fs from "mz/fs";
import path from "path";
import yaml from "yaml";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { Connection } from "@solana/web3.js";

// import * as buffer from "buffer";
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
export const getPayer = async (): Promise<Keypair> => {
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
export const createKeypairFromFile = async (
  filePath: string
): Promise<Keypair> => {
  const secretKeyString = await fs.readFile(filePath, { encoding: "utf8" });
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
};

/**
 * establishes connection to RPC
 * @returns Connection
 */
export const establishConnection = async () => {
  const rpcUrl = await getRpcUrl();
  const connection = new Connection(rpcUrl, "confirmed");
  const version = await connection.getVersion();
  console.log("Connection to cluster established:", rpcUrl, version);
  return connection;
};

// /**
//  * Establish an account to pay for everything
//  */
export const establishPayer = async (connection: Connection) => {
  let payer = await getPayer();

  // get enough lamports
  let lamports = await connection.getBalance(payer.publicKey);
  // If current balance is not enough to pay for fees, request an airdrop
  const sig = await connection.requestAirdrop(payer.publicKey, lamports);
  lamports = await connection.getBalance(payer.publicKey);

  await connection.confirmTransaction(sig);

  // console.log(
  //   "Using account",
  //   payer.publicKey.toBase58(),
  //   "containing",
  //   lamports / LAMPORTS_PER_SOL,
  //   "SOL to pay for fees"
  // );
  return payer;
};

export const checkOrDeployProgram = async (connection: Connection, programPath: string, programName: string): Promise<PublicKey>=>{
  const programKeypairPath = path.join(programPath, programName + "-keypair.json");
  const programSoPath = path.join(programPath, programName, ".so");

  let programId: PublicKey;

  // read keypair
  try{
    const programKeypair = await createKeypairFromFile(programKeypairPath);
    programId = programKeypair.publicKey;
  } catch(err){
    const errMsg = (err as Error).message;
    throw new Error(
      `Failed to read program keypair at '${programKeypairPath}' due to error: ${errMsg}. Program may need to be deployed with \`solana program deploy ${programKeypairPath}\``
    );
  }

  // Check if the program has been deployed
  const programInfo = await connection.getAccountInfo(programId);
  if (programInfo === null) {
    if (fs.existsSync(programSoPath)) {
      throw new Error(
        `Program needs to be deployed with "solana program deploy ${programSoPath}"`
      );
    } else {
      throw new Error("Program needs to be built and deployed");
    }
  } else if (!programInfo.executable) {
    throw new Error(`Program is not executable`);
  }
  // console.log(`Using program ${programId.toBase58()}`);
  return programId;
}
