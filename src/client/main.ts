import {
  establishConnection,
  establishPayer,
  checkOrDeployProgram,
  initializeAccounts,
} from "./w_pokt";
import { PublicKey, Keypair, Connection } from "@solana/web3.js";
import { AccountLayout } from "@solana/spl-token";
import { W_POKT_ACCOUNT_DATA_LAYOUT } from "./state";

async function verifyAccountsCreation(
  connection: Connection,
  programId: PublicKey,
  owner: Keypair,
  wpokt: Keypair,
  mint: Keypair
) {
  let owner_acc = await connection.getAccountInfo(owner.publicKey);
  let wpokt_acc = await connection.getAccountInfo(wpokt.publicKey);
  let mint_acc = await connection.getAccountInfo(mint.publicKey);

  if (owner_acc === null || owner_acc.data.length === 0) {
    console.log("owner_acc state account has not been initialized properly");
    process.exit(1);
  }

  if (wpokt_acc === null || wpokt_acc.data.length === 0) {
    console.log("wpokt_acc state account has not been initialized properly");
    process.exit(1);
  }

  if (mint_acc === null || mint_acc.data.length === 0) {
    console.log("mint_acc state account has not been initialized properly");
    process.exit(1);
  }

  // const wpokt_data = W_POKT_ACCOUNT_DATA_LAYOUT.decode(wpokt_acc.data);
  // const mint_data = AccountLayout.decode(mint_acc.data);
  console.log("verifyAccountsCreation: Accounts creation verified.");
}

async function main() {
  console.log("---> main(): Establoshing connection...\n");
  const connection: Connection = await establishConnection();

  // Determine who pays for the fees
  const payer: Keypair = await establishPayer();

  // Check if the program has been deployed
  const programId: PublicKey = await checkOrDeployProgram();

  let [owner, global_state, mint] = await initializeAccounts(programId);

  await verifyAccountsCreation(
    connection,
    programId,
    owner,
    global_state,
    mint
  );
  console.log("---> main(): Wraping up...\n");
}

main().then(
  () => process.exit(),
  (err) => {
    console.error(err);
    process.exit(-1);
  }
);
