import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  checkOrDeployProgram,
  establishConnection,
  establishPayer,
} from "./utils";
import path from "path";
import * as WPOKT from "./WPOKT/wpokt";

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
