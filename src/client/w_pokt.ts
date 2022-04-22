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

// const WPoktInstruction {
//   /// Accounts expected:
//   /// 0. `[signer]` The account of person deploying WPokt - the 'owner'.
//   /// 1. `[writable]` The account used as WPokt's global state
//   /// 2. `[]` the Mint account created by 'owner'.
//   Construct,
//   /// 0. `[signer]` The account of person deploying WPokt - the 'owner'.
//   SetBridgeOnlyOwner :{ bridge_address: PublicKey },
//   /// Accounts expected:
//   /// 0. `[]` The program owner's account.
//   /// 1. `[]` The account used as WPokt's global state
//   /// 2. `[signer]` the account used by Bridge as global state.
//   /// 3. `[writable]` the Mint account created by 'owner'.
//   /// 4. `[writeable]` the token account to mint to.
//   /// 5. `[]` The PDA account of WPokt to sign for mint
//   MintOnlyBridge : { amount: BN },
//   /// Accounts expected:
//   /// 0. `[writable]` The token account to burn from.
//   /// 1. `[signer]` the 0th token account's owner/delegate
//   /// 2. `[writable]` the mint account
//   Burn :{ amount: BN },
//   /// Accounts expected:
//   /// 0. `[signer]` The program owner's account.
//   /// 1. `[writeable]` The account used as WPokt's global state
//   RenounceOwnership,
//   /// Accounts expected:
//   /// 0. `[signer]` The program owner's account.
//   /// 1. `[writeable]` The account used as WPokt's global state
//   TransferOwnership: { new_owner: Pubkey },
// } as const;

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
const PROGRAM_PATH = path.resolve(__dirname, "../../build");

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

/**
 * Genarates randon keypairs and creates Owner, WPokt and Mint system accounts with appropriate data field layout.
 * @param programId The program id of WPokt program
 * @returns [owner account, WPokt account, WPokt mint account]
 */
export const initializeAccounts = async (
  programId: PublicKey
): Promise<[Keypair, Keypair, Keypair]> => {
  const owner = Keypair.generate();
  const global_state = Keypair.generate();
  const mint = Keypair.generate();

  const createOwnerAccountIx = SystemProgram.createAccount({
    programId: SystemProgram.programId,
    space: W_POKT_ACCOUNT_DATA_LAYOUT.span,
    lamports: await connection.getMinimumBalanceForRentExemption(100), // arbitrary min length
    fromPubkey: payer.publicKey,
    newAccountPubkey: owner.publicKey,
  });

  const createWPoktGlobalStateIx = SystemProgram.createAccount({
    programId,
    space: W_POKT_ACCOUNT_DATA_LAYOUT.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      W_POKT_ACCOUNT_DATA_LAYOUT.span
    ),
    fromPubkey: payer.publicKey,
    newAccountPubkey: global_state.publicKey,
  });

  const createMintAccountIx = SystemProgram.createAccount({
    programId: splToken.TOKEN_PROGRAM_ID,
    space: splToken.AccountLayout.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      splToken.AccountLayout.span
    ),
    fromPubkey: payer.publicKey,
    newAccountPubkey: mint.publicKey,
  });

  const tx = new Transaction();
  tx.add(createOwnerAccountIx, createWPoktGlobalStateIx, createMintAccountIx);
  await sendAndConfirmTransaction(connection, tx, [
    payer,
    owner,
    global_state,
    mint,
  ]);

  return [owner, global_state, mint];
};
