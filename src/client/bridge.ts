import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  checkOrDeployProgram,
  establishConnection,
  establishPayer,
} from "./utils";
import path from "path";
import * as Bridge from "./Bridge/bridge";
import * as SPLToken from "@solana/spl-token";

// program lib names
const BRIDGE_LIB_NAME = "bridge";

/**
 * Path to program files
 */
const PROGRAM_PATH = path.resolve(__dirname, "../../target/deploy/");

async function bridgeTests(connection: Connection, payer: Keypair) {
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
    await Bridge.generateTokenListDictionaryPda(programId, 1);
  const [tokenAddedPda, tokenAddedBump] =
    await Bridge.generateTokenAddedDictionaryPda(programId, wPoktMint);

  await Bridge.construct(
    connection,
    programId,
    payer,
    bridgePda,
    tokenAddedPda,
    tokenListPda,
    wPoktMint,
    payer.publicKey,
    1,
    1
  );
  console.log(
    `TSX - bridgeTests(): ${BRIDGE_LIB_NAME} BridgeInstruction::Construct...`
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
