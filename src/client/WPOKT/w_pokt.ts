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
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import * as splToken from "@solana/spl-token";
import * as WPoktState from "./state";
import { verifyMint } from "../utils";
import * as WPoktInstruction from "./instructions";

// // returns the WPokt PDA
// export const wpoktPdaKeypair = async (
//   mintAcc: PublicKey,
//   programId: PublicKey
// ): Promise<[PublicKey, number]> => {
//   let seeds: Uint8Array[] = [mintAcc.toBytes(), Buffer.from("WPokt")];
//   const [wpokt_pda, seedBump] = await PublicKey.findProgramAddress(
//     seeds,
//     programId
//   );
//   return [wpokt_pda, seedBump];
// };

// /**
//  * Genarates randon keypairs WPokt Mint system account with appropriate data field layout.
//  * @param connection The RPC connection instance
//  * @param payer The payer, deployer and owner of WPokt Progarm
//  * @param programId The program id of WPokt program
//  * @returns [WPokt Uninitialized Mint Account Keypair]
//  */
// export const createOrInitializeAccounts = async (
//   connection: Connection,
//   payer: Keypair,
//   mint: Keypair,
//   programId: PublicKey
// ): Promise<string> => {
//   const createMintAccountIx = SystemProgram.createAccount({
//     programId: splToken.TOKEN_PROGRAM_ID,
//     space: splToken.MintLayout.span,
//     lamports: await connection.getMinimumBalanceForRentExemption(
//       splToken.MintLayout.span
//     ),
//     fromPubkey: payer.publicKey,
//     newAccountPubkey: mint.publicKey,
//   });

//   const tx = new Transaction();
//   tx.add(createMintAccountIx);

//   return await sendAndConfirmTransaction(connection, tx, [payer, mint]);
// };

// export const construct = async (
//   connection: Connection,
//   payer: Keypair,
//   mintAccount: PublicKey,
//   programId: PublicKey
// ): Promise<string> => {
//   const [pda_account, seedBump] = await wPoktPdaKeypair(mintAccount, programId);

//   // create WPokt constructor instruction
//   const ix = new TransactionInstruction({
//     programId,
//     keys: [
//       { pubkey: payer.publicKey, isSigner: true, isWritable: true },
//       { pubkey: pda_account, isSigner: false, isWritable: true },
//       { pubkey: mintAccount, isSigner: false, isWritable: true },
//       { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
//       { pubkey: splToken.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
//       { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
//     ],
//     data: Buffer.from(Uint8Array.of(WPoktInstruction.WPoktInstruction.Construct)),
//   });

//   const tx = new Transaction().add(ix);
//   return await sendAndConfirmTransaction(connection, tx, [payer]);
// };

// export const setBridge = async (
//   connection: Connection,
//   programId: PublicKey,
//   owner: Keypair,
//   wPoktPda: PublicKey,
//   bridgePubkey: PublicKey
// ): Promise<string> => {
//   // const buffers = ;
//   const data = Buffer.concat([
//     Buffer.from(Uint8Array.of(WPoktInstruction.WPoktInstruction.SetBridgeOnlyOwner)),
//     bridgePubkey.toBuffer(),
//   ]);
//   //  buffers.concat(wPoktPda.toBuffer());
//   const ix = new TransactionInstruction({
//     programId,
//     keys: [
//       { pubkey: owner.publicKey, isSigner: true, isWritable: true },
//       { pubkey: wPoktPda, isSigner: false, isWritable: true },
//     ],
//     data,
//   });
//   const tx = new Transaction().add(ix);
//   return await sendAndConfirmTransaction(connection, tx, [owner]);
// };

// export const mint = async (
//   connection: Connection,
//   programId: PublicKey,
//   pdaAccount: PublicKey,
//   mint: PublicKey,
//   bridgeAccount: Keypair,
//   receiverAccount: PublicKey,
//   amount: number
// ) => {
//   let data = Buffer.alloc(9); // 1B Instruction, 9B amount
 

//   const instructionDataLength = WPoktInstruction.W_POKT_MINT_INSTRUCTION_LAYOUT.encode(
//     {
//       instruction: WPoktInstruction.WPoktInstruction.MintOnlyBridge,
//       amount,
//     },
//     data
//   );

//   const ix = new TransactionInstruction({
//     programId,
//     keys: [
//       { pubkey: pdaAccount, isSigner: false, isWritable: false },
//       { pubkey: bridgeAccount.publicKey, isSigner: true, isWritable: false },
//       { pubkey: mint, isSigner: false, isWritable: true },
//       { pubkey: receiverAccount, isSigner: false, isWritable: true },
//       { pubkey: splToken.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
//     ],
//     data,
//   });
//   const tx = new Transaction().add(ix);
//   return await sendAndConfirmTransaction(connection, tx, [bridgeAccount]);
// };

// // export const verifyMintInstruction = async (connection: Connection, tokenAccount: PublicKey, balance: number)=>{
// //     // get bridge token account, and check its balance

// // }
// /**
//  * Verifies all required accounts were created and have the correct initial states
//  * @param connection the rpc connection instance
//  * @param programId the WPokt programId
//  * @param owner the payer/deployer of WPokt
//  * @param w_pokt the WPokt PDA account
//  * @param mint the WPokt Mint account
//  */
// export const verifyCreateOrInitializeAccounts = async (
//   connection: Connection,
//   programId: PublicKey,
//   owner: PublicKey,
//   w_pokt: PublicKey, // doesn't yet exist, as its created on chain by Construct instruction
//   mint: PublicKey
// ) => {
//   let ownerAcc = await connection.getAccountInfo(owner);
//   let wPoktAcc = await connection.getAccountInfo(w_pokt);
//   let mintAcc = await connection.getAccountInfo(mint);

//   // || owner_acc.data.length === 0
//   if (ownerAcc === null || ownerAcc.data.length !== 0) {
//     console.log(
//       `TSX: verifyWPoktAccountsCreation(): WPokt Owner account not found or has data at ${owner}`
//     );
//     process.exit(1);
//   }

//   // check for PDA account non-existance as it's created on-chain
//   if (wPoktAcc !== null) {
//     console.log(
//       `TSX: verifyWPoktAccountsCreation(): WPokt PDA already in use at ${w_pokt}`
//     );
//     process.exit(1);
//   }

//   if (mintAcc === null || mintAcc.data.length === 0) {
//     console.log(
//       `TSX: verifyWPoktAccountsCreation(): WPokt Mint account not found at ${mint}`
//     );
//     process.exit(1);
//   }
// };

// export const verifyWpoktPda = async (
//   programId: PublicKey,
//   owner: PublicKey,
//   mint: PublicKey,
//   wPokt: WPoktState.WPoktLayout
// ) => {
//   if (wPokt.isInitialized == false) {
//     throw Error(
//       `TSX verifyWpoktPdaDataConstruction(): WPokt PDA Account Uninitialized`
//     );
//   }
//   if (!owner.equals(wPokt.owner)) {
//     throw Error(
//       `TSX verifyWpoktPdaDataConstruction(): WPokt PDA Account Owner Uninitialized`
//     );
//   }
//   if (!mint.equals(wPokt.mint)) {
//     throw Error(
//       `TSX verifyWpoktPdaDataConstruction(): WPokt PDA Account Mint Uninitialized`
//     );
//   }
//   if (!wPokt.bridgeAddress.equals(PublicKey.default)) {
//     throw Error(
//       `TSX verifyWpoktPdaDataConstruction(): WPokt PDA Account BridgeAddress Improper Initialization`
//     );
//   }
// };

// export const verifyConstruction = async (
//   connection: Connection,
//   programId: PublicKey,
//   owner: PublicKey,
//   w_pokt: PublicKey, // doesn't yet exist, as its created on chain by Construct instruction
//   mint: PublicKey
// ) => {
//   // get and verify WPokt PDA account
//   let wPoktAcc = await connection.getAccountInfo(w_pokt);
//   // check for PDA account non-existance as it's created on-chain
//   if (wPoktAcc === null) {
//     console.log(
//       `TSX: verifyWpoktConstruction(): WPokt PDA account not found ${w_pokt}`
//     );
//     process.exit(1);
//   }

//   // decode account
//   const wPoktAccData = WPoktState.W_POKT_ACCOUNT_DATA_LAYOUT.decode(
//     Buffer.from(wPoktAcc.data)
//   );

//   await verifyWpoktPda(programId, owner, mint, wPoktAccData);

//   // get and decode mint
//   const wPoktMintData = await splToken.getMint(
//     connection,
//     mint,
//     "confirmed",
//     splToken.TOKEN_PROGRAM_ID
//   );

//   await verifyMint(wPoktMintData, true, w_pokt, 0);
// };

// export const VerifyWPoktBridgeAddress = async (
//   connection: Connection,
//   wPoktPda: PublicKey,
//   bridge: PublicKey
// ) => {
//   // query bridge PDA account
//   const pdaAccount = await connection.getAccountInfo(wPoktPda);

//   if (pdaAccount === null) {
//     console.log(
//       `TSX: constVerifyWPoktBridgeAddress(): WPokt PDA account not found ${wPoktPda}`
//     );
//     process.exit(1);
//   }

//   // decode account
//   const pdaAccountData = WPoktState.W_POKT_ACCOUNT_DATA_LAYOUT.decode(
//     Buffer.from(pdaAccount.data)
//   );

//   // verify bridge address
//   if (!pdaAccountData.bridgeAddress.equals(bridge)) {
//     console.log(
//       `TSX: constVerifyWPoktBridgeAddress(): WPokt PDA Bridge Address is ${pdaAccountData.bridgeAddress.toBase58()}`
//     );
//     process.exit(1);
//   }
// };
