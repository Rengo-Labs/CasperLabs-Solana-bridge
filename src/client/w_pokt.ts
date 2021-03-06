import * as WPokt from "./WPokt/w_pokt";
import {
  PublicKey,
  Keypair,
  Connection,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import * as SplToken from "@solana/spl-token";

import {
  establishConnection,
  establishPayer,
  checkOrDeployProgram,
} from "./utils";
import path from "path";
import { assert } from "console";

// import { W_POKT_ACCOUNT_DATA_LAYOUT } from "./state";
// import { WPoktInstruction } from "./WPokt/instructions";

// program lib names
const W_POKT_LIB_NAME = "w_pokt";
const WPOKT_LIB_NAME = "wpokt";
const BRIDGE_LIB_NAME = "bridge";

/**
 * Path to program files
 */
const PROGRAM_PATH = path.resolve(__dirname, "../../target/deploy/");

const wPoktTests = async (
  connection: Connection,
  payer: Keypair
): Promise<[PublicKey, Keypair, PublicKey]> => {
  //deploy WPokt program
  const programId: PublicKey = await checkOrDeployProgram(
    connection,
    PROGRAM_PATH,
    W_POKT_LIB_NAME
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} deployed at ${programId}...`
  );

  const mintAccount = Keypair.generate();
  // create WPokt accounts
  await WPokt.createOrInitializeAccounts(
    connection,
    payer,
    mintAccount,
    programId
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Mint Account Created at ${mintAccount.publicKey}...`
  );

  const [pdaAccount, bumpSeed] = await WPokt.wPoktPdaKeypair(
    mintAccount.publicKey,
    programId
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} PDA Key Created at ${pdaAccount}...`
  );

  await WPokt.verifyCreateOrInitializeAccounts(
    connection,
    programId,
    payer.publicKey,
    pdaAccount,
    mintAccount.publicKey
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Accounts Created and Verified...`
  );

  // construct WPokt
  await WPokt.construct(connection, payer, mintAccount.publicKey, programId);
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Instruction::Construct...`
  );

  await WPokt.verifyConstruction(
    connection,
    programId,
    payer.publicKey,
    pdaAccount,
    mintAccount.publicKey
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Instruction::Construct Verified...`
  );

  const bridgeAddress = Keypair.generate();
  await connection.requestAirdrop(
    bridgeAddress.publicKey,
    LAMPORTS_PER_SOL * 100
  );

  // setBridge
  await WPokt.setBridge(
    connection,
    programId,
    payer,
    pdaAccount,
    bridgeAddress.publicKey
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Instruction::setBridgeOnlyOwner{ bridgeAddress: ${bridgeAddress.publicKey.toBase58()} }...`
  );

  // verify WPokt Bridge Address
  await WPokt.VerifyWPoktBridgeAddress(
    connection,
    pdaAccount,
    bridgeAddress.publicKey
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Instruction::setBridgeOnlyOwner{...} Verified...`
  );

  // create a token account
  const bridgeTokenAccount = await SplToken.createAccount(
    connection,
    payer,
    mintAccount.publicKey,
    bridgeAddress.publicKey
  );
  // console.log(
  //   `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Bridge Token Account Created and Initialized...`
  // );

  // verify account creation
  let bridgeTokenAccountInfo = await SplToken.getAccount(
    connection,
    bridgeTokenAccount
  );

  assert(
    bridgeTokenAccountInfo.amount === BigInt(0),
    "bridgeTokenAccountInfo.amount !== 0"
  );
  assert(
    bridgeTokenAccountInfo.owner.equals(bridgeAddress.publicKey),
    "bridgeTokenAccountInfo.owner != bridgeAddress"
  );
  // console.log(
  //   `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Bridge Token Account Creation and Initialization Verified...`
  // );

  const amount = 1;
  // Mint instruction
  await WPokt.mint(
    connection,
    programId,
    pdaAccount,
    mintAccount.publicKey,
    bridgeAddress,
    bridgeTokenAccount,
    amount
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Instruction::Mint{ amount: ${amount} }...`
  );

  bridgeTokenAccountInfo = await SplToken.getAccount(
    connection,
    bridgeTokenAccount
  );
  assert(
    bridgeTokenAccountInfo.amount === BigInt(amount),
    `bridgeTokenAccountInfo.amount !== ${amount}`
  );
  assert(
    bridgeTokenAccountInfo.owner.equals(bridgeAddress.publicKey),
    "bridgeTokenAccountInfo.owner != bridgeAddress"
  );

  let mintAccountInfo = await SplToken.getMint(
    connection,
    mintAccount.publicKey
  );
  assert(
    mintAccountInfo.supply === BigInt(amount),
    `mintAccountInfo.supply !== BigInt(amount)`
  );

  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Instruction::Mint{...} Verified....`
  );

  // Burn
  await WPokt.burn(
    connection,
    programId,
    mintAccount.publicKey,
    bridgeTokenAccount,
    bridgeAddress,
    amount
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Instruction::Birn{ amount: ${amount} }...`
  );
  bridgeTokenAccountInfo = await SplToken.getAccount(
    connection,
    bridgeTokenAccount
  );
  assert(
    bridgeTokenAccountInfo.amount === BigInt(0),
    `bridgeTokenAccountInfo.amount !== ${0}`
  );

  mintAccountInfo = await SplToken.getMint(connection, mintAccount.publicKey);
  assert(
    mintAccountInfo.supply === BigInt(0),
    `mintAccountInfo.supply !== BigInt(0)`
  );

  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Instruction::Burn{...} Verified....`
  );
  return [programId, mintAccount, pdaAccount];
};

async function main() {
  const connection: Connection = await establishConnection();
  console.log(
    `TSX - main(): Established Connection at ${connection.rpcEndpoint}`
  );

  // Determine who pays for the fees
  const payer: Keypair = await establishPayer(connection);
  console.log(`TSX - main(): Established Payer at ${payer.publicKey}`);

  const [wPoktProgramId, wPoktMintAccount, wPoktPdaAccount] = await wPoktTests(
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
