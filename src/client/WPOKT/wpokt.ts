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
  SYSVAR_CLOCK_PUBKEY,
} from "@solana/web3.js";
import * as splToken from "@solana/spl-token";
import * as WPOKTState from "./state";
import { verifyMint } from "../utils";
import * as WPOKTInstruction from "./instructions";
import { Key } from "readline";
import { connect } from "http2";
import { bigInt } from "@solana/buffer-layout-utils";

export const generateAuthorizationStateDictionaryKey = async (
  programId: PublicKey,
  from: PublicKey,
  mint: PublicKey,
  nonce: string
): Promise<[PublicKey, number]> => {
  const seeds: Uint8Array[] = [
    from.toBytes(),
    Buffer.from(nonce),
    mint.toBytes(),
    Buffer.from("WPOKT"),
    Buffer.from("authorization_dictionary_key"),
  ];

  const [pda, seedBump] = await PublicKey.findProgramAddress(seeds, programId);
  return [pda, seedBump];
};

export const generateNonceDictionaryKey = async (
  programId: PublicKey,
  owner: PublicKey,
  mint: PublicKey
): Promise<[PublicKey, number]> => {
  let seeds: Uint8Array[] = [
    owner.toBytes(),
    mint.toBytes(),
    Buffer.from("WPOKT"),
    Buffer.from("nonces_dictionary_key"),
  ];
  const [nonces_key, seedBump] = await PublicKey.findProgramAddress(
    seeds,
    programId
  );
  return [nonces_key, seedBump];
};

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
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer]);
};

export const mint = async (
  connection: Connection,
  programId: PublicKey,
  minter: Keypair,
  pdaAccount: PublicKey,
  mint: PublicKey,
  receiverAccount: PublicKey,
  amount: number
) => {
  let data = Buffer.alloc(WPOKTInstruction.MINT_ONLY_MINTER_LAYOUT.span); // 1B Instruction,32B to, 8B amount

  const instructionDataLength = WPOKTInstruction.MINT_ONLY_MINTER_LAYOUT.encode(
    {
      instruction: WPOKTInstruction.WPOKTInstruction.MintOnlyMinter,
      to: receiverAccount,
      value: amount,
    },
    data
  );

  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: minter.publicKey, isSigner: true, isWritable: false },
      { pubkey: pdaAccount, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: receiverAccount, isSigner: false, isWritable: true },
      { pubkey: splToken.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data,
  });
  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [minter]);
};

export const changeMinter = async (
  connection: Connection,
  programId: PublicKey,
  currentMinter: Keypair,
  newMinter: PublicKey,
  mint: PublicKey,
  wpoktPda: PublicKey
) => {
  const data = Buffer.alloc(
    WPOKTInstruction.CHANGE_MINTER_ONLY_MINTER_LAYOUT.span
  );

  WPOKTInstruction.CHANGE_MINTER_ONLY_MINTER_LAYOUT.encode(
    {
      instruction: WPOKTInstruction.WPOKTInstruction.ChangeMinterOnlyMinter,
      newMinter,
    },
    data
  );

  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: currentMinter.publicKey, isSigner: true, isWritable: false },
      { pubkey: wpoktPda, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: splToken.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: newMinter, isSigner: false, isWritable: true },
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [currentMinter]);
};

export const initializeAuthorizeStatePdaAccount = async (
  connection: Connection,
  programId: PublicKey,
  payer: Keypair,
  from: Keypair,
  nonce: string,
  authStatePdaAccount: PublicKey,
  mint: PublicKey
) => {
  let data = Buffer.alloc(
    WPOKTInstruction.INITIALIZE_AUTHORIZATION_STATE_PDA_ACCOUNT_LAYOUT.span
  );
  WPOKTInstruction.INITIALIZE_AUTHORIZATION_STATE_PDA_ACCOUNT_LAYOUT.encode(
    {
      instruction:
        WPOKTInstruction.WPOKTInstruction
          .InitializeAuthorizationStatePdaAccount,
      from: from.publicKey,
      nonce: Buffer.from(nonce),
    },
    data
  );

  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: from.publicKey, isSigner: true, isWritable: false },
      { pubkey: authStatePdaAccount, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: true },
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer, from]);
};

export const getAuthStateDictionaryAccount = async (
  connection: Connection,
  authStatePdaAccount: PublicKey
) => {
  const account = await connection.getAccountInfo(authStatePdaAccount);
  if (account === null) {
    throw Error("TSX: getNonceDictionaryItemAccount(): Account not found.");
  }
  //decode account
  return WPOKTState.WPOKT_AUTHORIZATION_DICTIONARY_LAYOUT.decode(
    Buffer.from(account.data)
  );
};

/**
 *
 * @param connection The rpc connection instance
 * @param programId The WPOKT programId
 * @param from the source token authority giving the allowance/authorization
 * @param nonce the nonce of the nonce account for offline transaction siging
 * @param mint the WPOKT mint account
 * @param state the state of authorization to verify
 */
export const verifyAuthStatePdaAccount = async (
  connection: Connection,
  programId: PublicKey,
  from: PublicKey,
  nonce: string,
  mint: PublicKey,
  state: boolean
) => {
  const [pda, bump] = await generateAuthorizationStateDictionaryKey(
    programId,
    from,
    mint,
    nonce
  );

  const data = await getAuthStateDictionaryAccount(connection, pda);

  if (data.authorization !== state) {
    throw Error(
      `TSX - verifyAuthStatePdaAccount(): Invalid auth state, it's ${data.authorization}`
    );
  }
  if (!data.from.equals(from)) {
    throw Error(
      `TSX - verifyAuthStatePdaAccount(): Unequal 'from' ${data.from.toBase58()}`
    );
  }
  if (data.nonce.toString() !== nonce) {
    throw Error(
      `TSX - verifyAuthStatePdaAccount(): Unequal 'nonce' ${data.nonce.toString()} !== ${nonce}`
    );
  }
};

/**
 *
 * @param connection The rpc connection instance
 * @param programId The WPOKT programId
 * @param owner The source token account authority giving the allowance
 * @param payer pays for the transaction
 * @param nonceAccount the Nonce Dictionary account holding the incremental nonce
 * @param mint the WPOKT mint account
 * @returns transaction signature
 */
export const initializeNoncePdaAccount = async (
  connection: Connection,
  programId: PublicKey,
  owner: Keypair,
  payer: Keypair,
  nonceAccount: PublicKey,
  mint: PublicKey
) => {
  let data = Buffer.alloc(
    WPOKTInstruction.INITIALIZE_NONCE_PDA_ACCOUNT_LAYOUT.span
  );
  WPOKTInstruction.INITIALIZE_NONCE_PDA_ACCOUNT_LAYOUT.encode(
    {
      instruction: WPOKTInstruction.WPOKTInstruction.InitializeNoncePdaAccount,
      owner: owner.publicKey,
    },
    data
  );

  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: owner.publicKey, isSigner: true, isWritable: false },
      { pubkey: nonceAccount, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: true },
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer, owner]);
};

export const getNonceDictionaryItemAccount = async (
  connection: Connection,
  nonceAccountAddress: PublicKey
) => {
  const account = await connection.getAccountInfo(nonceAccountAddress);

  if (account === null) {
    throw Error("TSX: getNonceDictionaryItemAccount(): Account not found.");
  }
  //decode account
  return WPOKTState.WPOKT_NONCES_DICTIONARY_LAYOUT.decode(
    Buffer.from(account.data)
  );
};

export const verifyNonceDictionaryItemAccount = async (
  connection: Connection,
  programId: PublicKey,
  owner: PublicKey,
  mint: PublicKey,
  nonce: number
) => {
  const [nonceAccount, bump] = await generateNonceDictionaryKey(
    programId,
    owner,
    mint
  );

  const data = await getNonceDictionaryItemAccount(connection, nonceAccount);

  if (!data.owner.equals(owner)) {
    throw Error(`TSX: validateNonceDictionaryItemAccount(): Invalid Owner`);
  }

  if (data.nonce !== BigInt(nonce)) {
    throw Error(`TSX: validateNonceDictionaryItemAccount(): Invalid Nonce`);
  }
};

// TODO implement ofc
export const transferWithAuthorization = async (){}
/**
 *
 * @param connection
 * @param programId
 * @param sourceToken
 * @param sourceTokenAuthority
 * @param mint
 * @param delegate
 */
export const permit = async (
  connection: Connection,
  programId: PublicKey,
  payer: Keypair,
  sourceToken: PublicKey,
  sourceTokenAuthority: Keypair,
  mint: PublicKey,
  delegateToken: PublicKey,
  amount: number,
  deadline: number,
  noncePdaAccount: PublicKey
) => {
  const data = Buffer.alloc(WPOKTInstruction.PERMIT_LAYOUT.span);
  const [nonceKey, bump] = await generateNonceDictionaryKey(
    programId,
    sourceTokenAuthority.publicKey,
    mint
  );

  if (!nonceKey.equals(noncePdaAccount)) {
    throw Error(`TSX - permit(): Nonce Account key mismatch`);
  }

  WPOKTInstruction.PERMIT_LAYOUT.encode(
    {
      instruction: WPOKTInstruction.WPOKTInstruction.Permit,
      owner: sourceTokenAuthority.publicKey,
      spender: delegateToken,
      value: amount,
      deadline,
    },
    data
  );

  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      {
        pubkey: sourceTokenAuthority.publicKey,
        isSigner: true,
        isWritable: false,
      },
      { pubkey: noncePdaAccount, isSigner: false, isWritable: true },
      { pubkey: sourceToken, isSigner: false, isWritable: true },
      { pubkey: delegateToken, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: splToken.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [
    sourceTokenAuthority,
    payer,
  ]);
};

export const verifyPermit = async (
  connection: Connection,
  programId: PublicKey,
  sourceToken: PublicKey,
  sourceTokenAuthority: PublicKey,
  mint: PublicKey,
  delegateToken: PublicKey,
  amount: number,
  noncePdaAccount: PublicKey,
  expectedNonce: number
) => {
  const [nonceKey, bump] = await generateNonceDictionaryKey(
    programId,
    sourceTokenAuthority,
    mint
  );

  if (!nonceKey.equals(noncePdaAccount)) {
    throw Error(`TSX - permit(): Nonce Account key mismatch`);
  }

  // verify nonce account
  await verifyNonceDictionaryItemAccount(
    connection,
    programId,
    sourceTokenAuthority,
    mint,
    expectedNonce
  );

  // verify delegate on src token account
  const srcTokenAccount = await splToken.getAccount(connection, sourceToken);

  if (srcTokenAccount === null) {
    throw Error(
      `TSX - verifyPermit(): source token account not found at ${sourceToken}`
    );
  }

  if (!srcTokenAccount.delegate?.equals(delegateToken)) {
    throw Error(
      `TSX - verifyPermit(): Delegate Key mismatch. srcTokenAccount.delegat is ${srcTokenAccount.delegate?.toBase58()}`
    );
  }

  if (srcTokenAccount.delegatedAmount !== BigInt(amount)) {
    throw Error(
      `TSX - verifyPermit(): Delegate Amount mismatch. srcTokenAccount.delegateAmount is ${srcTokenAccount.delegatedAmount}`
    );
  }
};
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
    throw Error(`TSX verifyWPOKTPda(): Invalid Minter at ${wpokt.minter}`);
  }

  if (!mint.equals(wpokt.mint)) {
    throw Error(`TSX verifyWPOKTPda(): WPOKT PDA Account Mint Uninitialized`);
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
  const wpoktMintData = await splToken.getMint(connection, mint);
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
