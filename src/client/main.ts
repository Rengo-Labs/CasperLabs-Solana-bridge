import * as WPokt from "./WPokt/w_pokt";
import { PublicKey, Keypair, Connection } from "@solana/web3.js";
import { AccountLayout } from "@solana/spl-token";
import {
  establishConnection,
  establishPayer,
  checkOrDeployProgram,
} from "./utils";
import path from "path";

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
  const wPoktProgramId: PublicKey = await checkOrDeployProgram(
    connection,
    PROGRAM_PATH,
    W_POKT_LIB_NAME
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} deployed at ${wPoktProgramId}...`
  );

  // create WPokt accounts
  const wPoktMintAccount = await WPokt.createOrInitializeAccounts(
    connection,
    payer,
    wPoktProgramId
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Mint Account Created at ${wPoktMintAccount.publicKey}...`
  );

  const [wPoktPdaAccount, bumpSeed] = await WPokt.wPoktPdaKeypair(
    wPoktMintAccount.publicKey,
    wPoktProgramId
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} PDA Key Created at ${wPoktPdaAccount}...`
  );

  await WPokt.verifyAccountsCreationAndInitialState(
    connection,
    wPoktProgramId,
    payer,
    wPoktPdaAccount,
    wPoktMintAccount
  );
  console.log(
    `TSX - wPoktTests(): ${W_POKT_LIB_NAME} Accounts creation and initial state verified...`
  );

  return [wPoktProgramId, wPoktMintAccount, wPoktPdaAccount];
};

async function main() {
  const connection: Connection = await establishConnection();
  console.log(
    `TSX - main(): Established Connection at ${connection.rpcEndpoint}`
  );

  // Determine who pays for the fees
  const payer: Keypair = await establishPayer(connection);
  console.log(`TSX - main(): Established Payer at ${payer.publicKey}`);

  const [wPoktProgramId, wPoktMintAccount, wPoktPdaAccount] = await wPoktTests(connection, payer);
}

main().then(
  () => process.exit(),
  (err) => {
    console.error(err);
    process.exit(-1);
  }
);
