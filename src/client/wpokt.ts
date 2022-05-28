import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  checkOrDeployProgram,
  establishConnection,
  establishPayer,
} from "./utils";
import path from "path";
import * as WPOKT from "./WPOKT/wpokt";
import * as SPLToken from "@solana/spl-token";
import assert from "assert";
import { mint } from "./WPokt/w_pokt";
import { connect } from "http2";

// program lib names
const WPOKT_LIB_NAME = "wpokt";

/**
 * Path to program files
 */
const PROGRAM_PATH = path.resolve(__dirname, "../../target/deploy/");

async function wpoktTests(
  connection: Connection,
  payer: Keypair
): Promise<[PublicKey, Keypair, PublicKey]> {
  //deploy WPOKT program
  const programId: PublicKey = await checkOrDeployProgram(
    connection,
    PROGRAM_PATH,
    WPOKT_LIB_NAME
  );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} deployed at ${programId}...`
  );

  const mintAccount = Keypair.generate();
  // create WPOKT accounts
  await WPOKT.createOrInitializeAccounts(connection, payer, mintAccount);
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} Mint Account Created at ${mintAccount.publicKey}...`
  );

  const [pdaAccount, bumpSeed] = await WPOKT.wpoktPdaKeypair(
    mintAccount.publicKey,
    programId
  );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} PDA Key Created at ${pdaAccount}...`
  );

  await WPOKT.verifyCreateOrInitializeAccounts(
    connection,
    pdaAccount,
    mintAccount.publicKey
  );

  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} Accounts Created and Verified...`
  );

  // construct WPOKT - making payer the initial minter
  await WPOKT.construct(
    connection,
    payer,
    mintAccount.publicKey,
    payer.publicKey,
    programId
  );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} Instruction::Construct...`
  );

  await WPOKT.verifyConstruction(
    connection,
    programId,
    payer.publicKey,
    pdaAccount,
    mintAccount.publicKey
  );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} Instruction::Construct Verified...`
  );

  const receiverAccount = await SPLToken.createAccount(
    connection,
    payer,
    mintAccount.publicKey,
    payer.publicKey
  );

  const mintAmount = 100;
  await WPOKT.mint(
    connection,
    programId,
    payer,
    pdaAccount,
    mintAccount.publicKey,
    receiverAccount,
    mintAmount
  );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} Instruction::MintOnlyMinter...`
  );

  // verify mint
  const receiverData = await SPLToken.getAccount(connection, receiverAccount);
  if (receiverData.amount !== BigInt(mintAmount)) {
    throw Error(
      `TSX - wpoktTests(): ${WPOKT_LIB_NAME} receiverData.amount !== BigInt(mintAmount)`
    );
  }

  let mintData = await SPLToken.getMint(connection, mintAccount.publicKey);
  if (mintData.supply !== BigInt(mintAmount)) {
    throw Error(
      `TSX - wpoktTests(): ${WPOKT_LIB_NAME} mintData.supply !== BigInt(mintAmount)`
    );
  }
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} Instruction::MintOnlyMinter Verified...`
  );

  // create new minter account
  const newMinter = Keypair.generate();
  const createAccountIx = SystemProgram.createAccount({
    programId: SystemProgram.programId,
    space: 1,
    lamports: await connection.getMinimumBalanceForRentExemption(1),
    fromPubkey: payer.publicKey,
    newAccountPubkey: newMinter.publicKey,
  });
  let tx = new Transaction().add(createAccountIx);
  await sendAndConfirmTransaction(connection, tx, [payer, newMinter]);
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} newMinter account created at ${newMinter.publicKey.toBase58()}...`
  );

  await WPOKT.changeMinter(
    connection,
    programId,
    payer,
    newMinter.publicKey,
    mintAccount.publicKey,
    pdaAccount
  );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} Instruction::ChangeMinter...`
  );

  // get mint to verify minter change
  // TODO verify minter change in PDA Account
  mintData = await SPLToken.getMint(connection, mintAccount.publicKey);
  if (!mintData.mintAuthority?.equals(newMinter.publicKey)) {
    throw Error(
      `TSX - wpoktTests(): ${WPOKT_LIB_NAME} mintData.mintAuthority? !== newMinter.publicKey`
    );
  }
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} Instruction::ChangeMinter Verified...`
  );

  // create and valida NoncesDictionaryItem account with owner 'payer'
  const [nonceAccount, nonceAccountBump] =
    await WPOKT.generateNonceDictionaryKey(
      programId,
      payer.publicKey,
      mintAccount.publicKey
    );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} NoncesDictionary PDA generated at ${nonceAccount.toBase58()}...`
  );
  await WPOKT.initializeNoncePdaAccount(
    connection,
    programId,
    payer,
    payer,
    nonceAccount,
    mintAccount.publicKey
  );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} NoncesDictionary PDA Account Initialized. ${nonceAccount.toBase58()}...`
  );
  await WPOKT.validateNonceDictionaryItemAccount(
    connection,
    programId,
    payer.publicKey,
    mintAccount.publicKey,
    0
  );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} NoncesDictionary PDA Account Initialization Validated. ${nonceAccount.toBase58()}...`
  );

  // create delegate token account, owner/auth is payer(not relevant who should be delegate auth)
  let delegateTokenAccount = await SPLToken.createAccount(
    connection,
    payer,
    mintAccount.publicKey,
    newMinter.publicKey
  );
  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} Delegate Token Account created. ${delegateTokenAccount.toBase58()}...`
  );
  // get blocktime to set deadline
  let currentSlot = await connection.getSlot();
  let currentblocktime = await connection.getBlockTime(currentSlot);
  if (currentblocktime === null) {
    throw Error(`TSC - wpoktTests(): blocktime is null`);
  }
  let deadline = currentblocktime * 2;

  // now we permint the delegate
  const delegateAmount = mintAmount / 100;
  await WPOKT.permit(
    connection,
    programId,
    payer,
    receiverAccount,
    payer,
    mintAccount.publicKey,
    delegateTokenAccount,
    delegateAmount,
    deadline,
    nonceAccount
  );

  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} WPOKTInstruction::Permit...`
  );

  await WPOKT.verifyPermit(
    connection,
    programId,
    receiverAccount,
    payer.publicKey,
    mintAccount.publicKey,
    delegateTokenAccount,
    delegateAmount,
    nonceAccount,
    1
  );

  console.log(
    `TSX - wpoktTests(): ${WPOKT_LIB_NAME} WPOKTInstruction::Permit Verified...`
  );
  // NOTE reusing receiverAccount for source token, having being minted to.
  // await WPOKT.permit(connection)
  // cerate source token auth nonces account
  return [PublicKey.default, Keypair.generate(), PublicKey.default];
}

async function main() {
  const connection: Connection = await establishConnection();
  console.log(
    `TSX - main(): Established Connection at ${connection.rpcEndpoint}`
  );

  // Determine who pays for the fees
  const payer: Keypair = await establishPayer(connection);
  console.log(`TSX - main(): Established Payer at ${payer.publicKey}`);

  const [wPoktProgramId, wPoktMintAccount, wPoktPdaAccount] = await wpoktTests(
    connection,
    payer
  );

  console.log(`TSX - main(): Finished...`);
}

main().then(
  () => process.exit(),
  (err) => {
    console.error(err);
    process.exit(-1);
  }
);
