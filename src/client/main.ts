import { establishConnection, establishPayer, checkProgram } from "./WPokt";

async function main() {
  console.log("Establoshing connection...");
  await establishConnection();

  // Determine who pays for the fees
  await establishPayer();

  // Check if the program has been deployed
  await checkProgram();

  console.log("Success");
}

main().then(
  () => process.exit(),
  (err) => {
    console.error(err);
    process.exit(-1);
  }
);
