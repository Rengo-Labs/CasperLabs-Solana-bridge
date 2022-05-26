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
import * as WPOKTState from "./state";
import { verifyMint } from "../utils";
import * as WPOKTInstruction from "./instructions";

// returns the WPOKT PDA
export const wpoktPdaKeypair = async (
  mintAcc: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]> => {
  let seeds: Uint8Array[] = [
    mintAcc.toBytes(),
    Buffer.from("WPOKT"),
    Buffer.from("global_state_account"),
  ];
  const [wpokt_pda, seedBump] = await PublicKey.findProgramAddress(
    seeds,
    programId
  );
  return [wpokt_pda, seedBump];
};

/**
 * Genarates randon keypairs WPOKT Mint system account with appropriate data field layout.
 * @param connection The RPC connection instance
 * @param payer The payer, deployer and owner of WPOKT Progarm
 * @returns [WPOKT Uninitialized Mint Account Keypair]
 */
export const createOrInitializeAccounts = async (
  connection: Connection,
  payer: Keypair,
  mint: Keypair
): Promise<string> => {
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

  return await sendAndConfirmTransaction(connection, tx, [payer, mint]);
};

export const construct = async (
  connection: Connection,
  payer: Keypair,
  mintAccount: PublicKey,
  initialMinter: PublicKey,
  programId: PublicKey
): Promise<string> => {
  const [pda_account, seedBump] = await wpoktPdaKeypair(mintAccount, programId);
  const data = Buffer.alloc(WPOKTInstruction.CONSTRUCT_LAYOUT.span);
  const encodedDataLen = WPOKTInstruction.CONSTRUCT_LAYOUT.encode(
    {
      instruction: WPOKTInstruction.WPOKTInstruction.Construct,
      initialMinter: initialMinter,
    },
    data
  );

  if (encodedDataLen !== WPOKTInstruction.CONSTRUCT_LAYOUT.span) {
    throw Error("TSX: construct(): encodedDataLen !== CONSTRUCT_LAYOUT.span");
  }

  // create WPOKT constructor instruction
  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: mintAccount, isSigner: false, isWritable: true },
      { pubkey: pda_account, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: splToken.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: initialMinter, isSigner: false, isWritable: false },
    ],
    data
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer]);
};

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

// }
/**
 * Verifies all required accounts were created and have the correct initial states
 * @param connection the rpc connection instance
 * @param programId the WPOKT programId
 * @param owner the payer/deployer of WPOKT
 * @param wpokt the WPOKT PDA account
 * @param mint the WPOKT Mint account
 */
export const verifyCreateOrInitializeAccounts = async (
  connection: Connection,
  wpokt: PublicKey, // doesn't yet exist, as its created on chain by Construct instruction
  mint: PublicKey
) => {
  let wpoktAcc = await connection.getAccountInfo(wpokt);
  let mintAcc = await connection.getAccountInfo(mint);

  // check for PDA account non-existance as it's created on-chain
  if (wpoktAcc !== null) {
    throw Error(
      `TSX: verifyCreateOrInitializeAccounts(): WPOKT PDA already in use at ${wpokt}`
    );
  }

  if (mintAcc === null || mintAcc.data.length === 0) {
    throw Error(
      `TSX: verifyCreateOrInitializeAccounts(): WPOKT Mint account not found at ${mint}`
    );
  }
  const mintData = await splToken.getMint(connection, mint);
  if (mintData.isInitialized) {
    throw Error(
      `TSX: verifyCreateOrInitializeAccounts(): WPOKT Mint account already initialized ${mint}`
    );
  }
};

// TODO fails to decode WPOKT state
export const verifyWPOKTPda = async (
  connection: Connection,
  programId: PublicKey,
  minter: PublicKey,
  mint: PublicKey,
  wpoktPda: PublicKey,
  initialization: boolean
) => {
  const wpoktInfo = await connection.getAccountInfo(wpoktPda);
  if (wpoktInfo === null) {
    throw Error(`TSX: verifyWPOKTPda(): WPOKT PDA not found at ${wpoktPda}`);
  }

  if (!wpoktInfo.owner.equals(programId)) {
    throw Error(
      `TSX: verifyWPOKTPda(): WPOKT PDA Account Invalid Owner ${wpoktInfo.owner}`
    );
  }

  const wpokt = WPOKTState.WPOKT_ACCOUNT_DATA_LAYOUT.decode(
    Buffer.from(wpoktInfo.data)
  );

  if (wpokt.isInitialized !== initialization) {
    throw Error(
      `TSX verifyWPOKTPda(): WPOKT PDA Account Initialization status is ${wpokt.isInitialized}`
    );
  }

  if (!minter.equals(wpokt.minter)) {
    throw Error(
      `TSX verifyWPOKTPda(): Invalid Minter at ${wpokt.minter}`
    );
  }

  if (!mint.equals(wpokt.mint)) {
    throw Error(
      `TSX verifyWPOKTPda(): WPOKT PDA Account Mint Uninitialized`
    );
  }
};

export const verifyConstruction = async (
  connection: Connection,
  programId: PublicKey,
  minter: PublicKey,
  wpokt: PublicKey,
  mint: PublicKey
) => {
 
  // TODO cant decode WPOKT struct
  // await verifyWPOKTPda(connection, programId, minter, mint, wpokt, true);
  // get and decode mint
  const wpoktMintData = await splToken.getMint(
    connection,
    mint,
  );
  await verifyMint(wpoktMintData, true, wpokt, 0);
};

// export const VerifyWPoktBridgeAddress = async (
//   connection: Connection,
//   wPoktPda: PublicKey,
//   bridge: PublicKey
// ) => {
//   // query bridge PDA account
//   const pdaAccount = await connection.getAccountInfo(wPoktPda);

//   if (pdaAccount === null) {
//     console.log(
//       `TSX: constVerifyWPoktBridgeAddress(): WPOKT PDA account not found ${wPoktPda}`
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
//       `TSX: constVerifyWPoktBridgeAddress(): WPOKT PDA Bridge Address is ${pdaAccountData.bridgeAddress.toBase58()}`
//     );
//     process.exit(1);
//   }
// };
