/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */

import {
  Keypair,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  SystemInstruction,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import * as splToken from "@solana/spl-token";
import fs from "mz/fs";
import path from "path";
import { getPayer, getRpcUrl, createKeypairFromFile } from "./utils";
import { W_POKT_ACCOUNT_DATA_LAYOUT } from "./state";
import { WPoktInstruction } from "./instructions";
/**
 * Connection to the network
 */
let connection: Connection;

/**
 * Keypair associated to the fees' payer
 */
let payer: Keypair;

/**
 * Hello world's program id
 */
let programId: PublicKey;

/**
 * Path to program files
 */
const PROGRAM_PATH = path.resolve(__dirname, "../../target/deploy/");

/**
 * Path to program shared object file which should be deployed on chain.
 * This file is created when running either:
 *   - `npm run build:program-rust`
 */
const PROGRAM_SO_PATH = path.join(PROGRAM_PATH, "w_pokt.so");

/**
 * Path to the keypair of the deployed program.
 * This file is created when running `solana program deploy build/w_pokt.so`
 */
const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, "w_pokt-keypair.json");

/**
 * WPokt PDA account seed
 */
export let W_POKT_PDA_ACCOUNT_SEED: String;

export const establishConnection = async () => {
  const rpcUrl = await getRpcUrl();
  connection = new Connection(rpcUrl, "confirmed");
  const version = await connection.getVersion();
  console.log("Connection to cluster established:", rpcUrl, version);
  return connection;
};

// /**
//  * Establish an account to pay for everything
//  */
export const establishPayer = async () => {
  const fees = 0;
  if (!payer) {
    // const {blockhash} = await connection.getLatestBlockhash();
    // const feeCalculator = await connection.getFeeForMessage();
    // // Calculate the cost to fund the greeter account
    // fees += await connection.getMinimumBalanceForRentExemption(W_POKT_SIZE);

    // // Calculate the cost of sending transactions
    // fees += feeCalculator.lamportsPerSignature * 100; // wag

    payer = await getPayer();
  }

  let lamports = await connection.getBalance(payer.publicKey);
  if (lamports < fees) {
    // If current balance is not enough to pay for fees, request an airdrop
    const sig = await connection.requestAirdrop(
      payer.publicKey,
      fees - lamports
    );
    await connection.confirmTransaction(sig);
    lamports = await connection.getBalance(payer.publicKey);
  }

  console.log(
    "Using account",
    payer.publicKey.toBase58(),
    "containing",
    lamports / LAMPORTS_PER_SOL,
    "SOL to pay for fees"
  );
  return payer;
};

/**
 * Check if the WPokt BPF program has been deployed
 */
export const checkOrDeployProgram = async () => {
  // Read program id from keypair file
  try {
    const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
    programId = programKeypair.publicKey;
  } catch (err) {
    const errMsg = (err as Error).message;
    throw new Error(
      `Failed to read program keypair at '${PROGRAM_KEYPAIR_PATH}' due to error: ${errMsg}. Program may need to be deployed with \`solana program deploy dist/program/helloworld.so\``
    );
  }

  // Check if the program has been deployed
  const programInfo = await connection.getAccountInfo(programId);
  if (programInfo === null) {
    if (fs.existsSync(PROGRAM_SO_PATH)) {
      throw new Error(
        "Program needs to be deployed with `solana program deploy build/w_pokt.so`"
      );
    } else {
      throw new Error("Program needs to be built and deployed");
    }
  } else if (!programInfo.executable) {
    throw new Error(`Program is not executable`);
  }
  console.log(`Using program ${programId.toBase58()}`);
  return programId;
};

// returns the WPokt PDA
export const wPoktPdaKey = async (
  mintAcc: Keypair,
  programId: PublicKey
): Promise<PublicKey> => {
  let seeds: Uint8Array[] = [mintAcc.publicKey.toBytes(), Buffer.from("WPokt")];
  const [wpokt_pda, seedBump] = await PublicKey.findProgramAddress(
    seeds,
    programId
  );
  return wpokt_pda;
};

/**
 * Genarates randon keypairs and creates Owner, WPokt and Mint system accounts with appropriate data field layout.
 * @param programId The program id of WPokt program
 * @returns [owner account, WPokt account, WPokt mint account]
 */
export const initializeAccounts = async (
  programId: PublicKey
): Promise<[Keypair, PublicKey, Keypair]> => {
  const owner = Keypair.generate();
  const mint = Keypair.generate();
  const global_state = await wPoktPdaKey(mint, programId);

  // const createOwnerAccountIx = SystemProgram.createAccount({
  //   programId: SystemProgram.programId,
  //   space: W_POKT_ACCOUNT_DATA_LAYOUT.span,
  //   lamports: await connection.getMinimumBalanceForRentExemption(100), // arbitrary min length
  //   fromPubkey: payer.publicKey,
  //   newAccountPubkey: owner.publicKey,
  // });

  // let airdropSignatureToAccount = await connection.requestAirdrop(
  //   payer.publicKey,
  //   LAMPORTS_PER_SOL
  // );

  // const createWPoktGlobalStateIx = SystemProgram.createAccount({
  //   programId,
  //   space: W_POKT_ACCOUNT_DATA_LAYOUT.span,
  //   lamports: await connection.getMinimumBalanceForRentExemption(
  //     W_POKT_ACCOUNT_DATA_LAYOUT.span
  //   ),
  //   fromPubkey: payer.publicKey,
  //   newAccountPubkey: global_state,
  // });

  console.log("Mint layout span: ", splToken.MintLayout.span);
  console.log("Lamports required: ", await connection.getMinimumBalanceForRentExemption(
    splToken.MintLayout.span
  ));

  const createMintAccountIx = SystemProgram.createAccount({
    programId: splToken.TOKEN_PROGRAM_ID,
    space: splToken.MintLayout.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      splToken.MintLayout.span
    ),
    fromPubkey: payer.publicKey,
    newAccountPubkey: mint.publicKey,
  });

  const tx = new Transaction();
  tx.add(
    // createOwnerAccountIx, 
    // createWPoktGlobalStateIx, 
    createMintAccountIx
    );

  // create WPokt constructor instruction
  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: global_state, isSigner: false, isWritable: true },
      { pubkey: mint.publicKey, isSigner: false, isWritable: true },
      {pubkey: SystemProgram.programId, isSigner: false, isWritable: false}
    ],
    data: Buffer.from(Uint8Array.of(0)),
  });
  tx.add(ix);

  await sendAndConfirmTransaction(connection, tx, [
    payer,
    // owner,
    // global_state,
    mint,
  ]);

  return [payer, global_state, mint];
};
