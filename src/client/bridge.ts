import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
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
import * as Bridge from "./Bridge/bridge";
import * as SPLToken from "@solana/spl-token";
import { CalcuateFeeResultLayout } from "./Bridge/state";
import { connect } from "http2";
import { transfer } from "@solana/spl-token";

// program lib names
const BRIDGE_LIB_NAME = "bridge";

/**
 * Path to program files
 */
const PROGRAM_PATH = path.resolve(__dirname, "../../target/deploy/");

async function bridgeTests(connection: Connection, payer: Keypair) {
  const tokenIndex = 1;
  const chainId = 1;
  const stableFee = 1;
  const index = tokenIndex;

  //deploy WPOKT program
  const programId: PublicKey = await checkOrDeployProgram(
    connection,
    PROGRAM_PATH,
    BRIDGE_LIB_NAME
  );
  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} deployed at ${programId}...`
  );

  const wPoktMint = await SPLToken.createMint(
    connection,
    payer,
    payer.publicKey,
    null,
    0
  );
  const [bridgePda, bridgeBump] = await Bridge.generateBridgePda(programId);
  const [tokenListPda, tokenListBump] =
    await Bridge.generateTokenListDictionaryPda(programId, index);
  const [tokenAddedPda, tokenAddedBump] =
    await Bridge.generateTokenAddedDictionaryPda(programId, wPoktMint);
  const [bridgeWpoktTokenAccountPda, bridgeWpoktTokenAccountBump] =
    await Bridge.generateBridgeTokenAcccountPda(
      connection,
      programId,
      wPoktMint
    );

  await Bridge.construct(
    connection,
    programId,
    payer,
    bridgePda,
    tokenAddedPda,
    tokenListPda,
    wPoktMint,
    payer.publicKey,
    chainId,
    stableFee,
    bridgeWpoktTokenAccountPda
  );

  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} BridgeInstruction::Construct...`
  );

  await Bridge.verifyConstruction(
    connection,
    payer.publicKey,
    bridgePda,
    tokenListPda,
    wPoktMint
  );
  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} BridgeInstruction::Construct Verified...`
  );

  // create source and source auth
  const from = payer;
  const fromTokenAccount = await SPLToken.createAccount(
    connection,
    payer,
    wPoktMint,
    payer.publicKey
  );
  // mint to source
  const mintAmount = 100;
  await SPLToken.mintTo(
    connection,
    payer,
    wPoktMint,
    fromTokenAccount,
    payer,
    mintAmount
  );
  // create account for return value
  const calculateFeeAccount = Keypair.generate();
  const createCalculateFeeAccountIx = SystemProgram.createAccount({
    programId,
    space: CalcuateFeeResultLayout.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      CalcuateFeeResultLayout.span
    ),
    fromPubkey: from.publicKey,
    newAccountPubkey: calculateFeeAccount.publicKey,
  });
  let tx = new Transaction().add(createCalculateFeeAccountIx);
  await sendAndConfirmTransaction(connection, tx, [payer, calculateFeeAccount]);

  await Bridge.transferRequest(
    connection,
    programId,
    payer,
    bridgePda,
    tokenListPda,
    wPoktMint,
    fromTokenAccount,
    calculateFeeAccount.publicKey,
    bridgeWpoktTokenAccountPda,
    from,
    tokenIndex,
    bridgeWpoktTokenAccountPda,
    mintAmount / 2,
    chainId + 1
  );
  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} BridgeInstruction::TransferRequest...`
  );

  await Bridge.verifyTransferRequest(
    connection,
    bridgePda,
    fromTokenAccount,
    bridgeWpoktTokenAccountPda,
    mintAmount / 2,
    mintAmount / 2,
    2
  );
  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} BridgeInstruction::TransferRequest Verified...`
  );

  const [dtcPda, dtcPdaBump] =
    await Bridge.genereteDailyTokenClaimsDictionaryPda(programId, tokenIndex);
  await Bridge.createDailyTokenClaimsDictionaryPdaAccount(
    connection,
    programId,
    payer,
    dtcPda,
    tokenIndex
  );

  const [claimedPda, claimedPdaBump] =
    await Bridge.generateClaimedDictionaryPda(programId, chainId, index);
  await Bridge.createClaimedDictionaryPdaAccount(
    connection,
    programId,
    payer,
    claimedPda,
    index,
    chainId
  );

  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} DTC & Claimed Pda Accounts created...`
  );

  // create a temporary receiver system and token account
  const toAuth = Keypair.generate();
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(
      SystemProgram.createAccount({
        programId: SystemProgram.programId,
        space: 1,
        lamports: await connection.getMinimumBalanceForRentExemption(
          1
        ),
        fromPubkey: payer.publicKey,
        newAccountPubkey: toAuth.publicKey,
      })
    ),
    [payer, toAuth]
  );
  await connection.requestAirdrop(toAuth.publicKey, 1000 * LAMPORTS_PER_SOL);
  // // create temporary
  const toTokenAcccount = await SPLToken.createAccount(
    connection,
    payer,
    wPoktMint,
    toAuth.publicKey
  );
  
  // get fromTokenAccount current balance
  const fromTokenData = await SPLToken.getAccount(connection, fromTokenAccount);
  const transferAmount = Number(fromTokenData.amount);
  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} toAuth and toTokenAccount created...`
  );
  await Bridge.transferReceipt(
    connection,
    programId,
    bridgePda,
    tokenListPda,
    claimedPda,
    dtcPda,
    wPoktMint,
    fromTokenAccount,
    payer,
    tokenIndex,
    toTokenAcccount,
    toAuth,
    transferAmount,
    chainId+1,
    index
  );
  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} BridgeInstruction::TransferReceipt...`
  );
  const claimedPdaData = await Bridge.getClaimedPdaData(connection, claimedPda);
  const dtcPdaData = await Bridge.getDailtTokenClaimsPdaData(connection, dtcPda);

  if (claimedPdaData.claimed !== true) {
    throw Error(`TSX - bridgeTests(): claimedPdaData.claimed !== true`);
  }
  if (dtcPdaData.dailyTokenClaims !== transferAmount){
    throw Error(`TSX - bridgeTests(): dtcPdaData.dailyTokenClaims !== transferAmount`);

  }

  const toTokenData = await SPLToken.getAccount(connection, toTokenAcccount);
  if (toTokenData.amount !== BigInt(transferAmount)){
    throw Error(`TSX - bridgeTests(): toTokenData.amount !== BigInt(transferAmount)`);

  }
  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} BridgeInstruction::TransferReceipt Verified...`
  );

}

async function main() {
  const connection: Connection = await establishConnection();
  console.log(
    `TSX - main(): Established Connection at ${connection.rpcEndpoint}`
  );

  // Determine who pays for the fees
  const payer: Keypair = await establishPayer(connection);
  console.log(`TSX - main(): Established Payer at ${payer.publicKey}`);

  await bridgeTests(connection, payer);

  console.log(`TSX - main(): Finished...`);
}

main().then(
  () => process.exit(),
  (err) => {
    console.error(err);
    process.exit(-1);
  }
);
