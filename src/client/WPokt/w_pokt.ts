/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */

import {
  Keypair,
  Connection,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import * as splToken from "@solana/spl-token";
import { WPoktInstruction } from "./instructions";
import {WPoktLayout} from "./state";

// returns the WPokt PDA
export const wPoktPdaKeypair = async (
  mintAcc: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]> => {
  let seeds: Uint8Array[] = [mintAcc.toBytes(), Buffer.from("WPokt")];
  const [wpokt_pda, seedBump] = await PublicKey.findProgramAddress(
    seeds,
    programId
  );
  return [wpokt_pda, seedBump];
};

/**
 * Genarates randon keypairs WPokt Mint system account with appropriate data field layout.
 * @param connection The RPC connection instance
 * @param payer The payer, deployer and owner of WPokt Progarm
 * @param programId The program id of WPokt program
 * @returns [WPokt Uninitialized Mint Account Keypair]
 */
export const createOrInitializeAccounts = async (
  connection: Connection,
  payer: Keypair,
  programId: PublicKey
): Promise<Keypair> => {
  const mint = Keypair.generate();

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
  tx.add(createMintAccountIx);

  await sendAndConfirmTransaction(connection, tx, [payer, mint]);

  return mint;
};

export const construct = async (
  connection: Connection,
  payer: Keypair,
  mintAccount: Keypair,
  programId: PublicKey
) => {
  const [pda_account, seedBump] = await wPoktPdaKeypair(
    mintAccount.publicKey,
    programId
  );

  // create WPokt constructor instruction
  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: pda_account, isSigner: false, isWritable: true },
      { pubkey: mintAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: splToken.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(Uint8Array.of(WPoktInstruction.Construct)),
  });

  const tx = new Transaction().add(ix);

  await sendAndConfirmTransaction(connection, tx, [payer]);
};

/**
 * Verifies all required accounts were created and have the correct initial states
 * @param connection the rpc connection instance
 * @param programId the WPokt programId
 * @param owner the payer/deployer of WPokt
 * @param w_pokt the WPokt PDA account
 * @param mint the WPokt Mint account
 */
export const verifyAccountsCreationAndInitialState = async (
  connection: Connection,
  programId: PublicKey,
  owner: Keypair,
  w_pokt: PublicKey, // doesn't yet exist, as its created on chain by Construct instruction
  mint: Keypair
) => {
  let ownerAcc = await connection.getAccountInfo(owner.publicKey);
  let wPoktAcc = await connection.getAccountInfo(w_pokt);
  let mintAcc = await connection.getAccountInfo(mint.publicKey);

  // || owner_acc.data.length === 0
  if (ownerAcc === null || ownerAcc.data.length !== 0) {
    console.log(`TSX: verifyWPoktAccountsCreation(): WPokt Owner account not found or has data at ${owner}`);
    process.exit(1);
  }

  // check for PDA account non-existance as it's created on-chain
  if (wPoktAcc !== null) {
    console.log(`TSX: verifyWPoktAccountsCreation(): WPokt PDA already in use at ${w_pokt}`);
    process.exit(1);
  }

  if (mintAcc === null || mintAcc.data.length === 0) {
    console.log(`TSX: verifyWPoktAccountsCreation(): WPokt Mint account not found at ${mint}`);
    process.exit(1);
  }
}
