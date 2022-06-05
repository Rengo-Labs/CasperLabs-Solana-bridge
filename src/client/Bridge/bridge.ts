import {
  PublicKey,
  Connection,
  Keypair,
  TransactionInstruction,
  SystemProgram,
  sendAndConfirmTransaction,
  SYSVAR_RENT_PUBKEY,
  Transaction,
} from "@solana/web3.js";
import { BN } from "bn.js";
import * as BridgeInstruction from "./instructions";
import {
  BRIDGE_LAYOUT,
  CLAIMED_DICTIONARY_LAYOUT,
  DAILY_TOKEN_CLAIMS_DICTIONARY_LAYOUT,
  TOKEN_ADDED_ACCOUNT_LAYOUT,
  TOKEN_LIST_DICTIONARY_LAYOUT,
} from "./state";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import * as SPLToken from "@solana/spl-token";
import { Key } from "readline";

export const generateBridgeTokenAcccountPda = async (
  connection: Connection,
  programId: PublicKey,
  mintAccount: PublicKey
): Promise<[PublicKey, number]> => {
  const seeds: Uint8Array[] = [
    mintAccount.toBytes(),
    Buffer.from("bridge"),
    Buffer.from("bridge_token_account"),
  ];

  const [pda, seedBump] = await PublicKey.findProgramAddress(seeds, programId);
  return [pda, seedBump];
};
export const generateBridgePda = async (
  programId: PublicKey
): Promise<[PublicKey, number]> => {
  const seeds: Uint8Array[] = [
    Buffer.from("bridge"),
    Buffer.from("signature_account"),
  ];

  const [pda, seedBump] = await PublicKey.findProgramAddress(seeds, programId);
  return [pda, seedBump];
};

export const generateClaimedDictionaryPda = async (
  programId: PublicKey,
  chainId: number,
  index: number
): Promise<[PublicKey, number]> => {
  const seeds: Uint8Array[] = [
    numberToLeBytes(chainId, 8),
    numberToLeBytes(index, 8),
    Buffer.from("bridge"),
    Buffer.from("claimed_dictionary_key"),
  ];
  const [pda, seedBump] = await PublicKey.findProgramAddress(seeds, programId);
  return [pda, seedBump];
};

export const numberToLeBytes = (num: number, length: number) => {
  const n = new BN(num);
  const buffer = n.toBuffer("le", length);
  return buffer;
};

export const generateTokenListDictionaryPda = async (
  programId: PublicKey,
  index: number
): Promise<[PublicKey, number]> => {
  const seeds: Uint8Array[] = [
    numberToLeBytes(index, 8),
    Buffer.from("bridge"),
    Buffer.from("token_list_dictionary_key"),
  ];
  const [pda, seedBump] = await PublicKey.findProgramAddress(seeds, programId);
  return [pda, seedBump];
};

export const genereteDailyTokenClaimsDictionaryPda = async (
  programId: PublicKey,
  tokenIndex: number
): Promise<[PublicKey, number]> => {
  const seeds: Uint8Array[] = [
    numberToLeBytes(tokenIndex, 8),
    Buffer.from("bridge"),
    Buffer.from("dtc_dictionary_key"),
  ];

  const [pda, seedBump] = await PublicKey.findProgramAddress(seeds, programId);
  return [pda, seedBump];
};

export const generateTokenAddedDictionaryPda = async (
  programId: PublicKey,
  tokenMintAddress: PublicKey
): Promise<[PublicKey, number]> => {
  const seeds: Uint8Array[] = [
    tokenMintAddress.toBytes(),
    Buffer.from("bridge"),
    Buffer.from("token_added_dictionary_key"),
  ];

  const [pda, seedBump] = await PublicKey.findProgramAddress(seeds, programId);
  return [pda, seedBump];
};

export const getBridgePdaData = async (
  connection: Connection,
  address: PublicKey
) => {
  const account = await connection.getAccountInfo(address);
  if (account === null) {
    throw Error("TSX: getBridgePdaData(): Account not found.");
  }
  //decode account
  return BRIDGE_LAYOUT.decode(Buffer.from(account.data));
};

export const getTokenAddedPdaData = async (
  connection: Connection,
  address: PublicKey
) => {
  const account = await connection.getAccountInfo(address);
  if (account === null) {
    throw Error("TSX: getTokenAddedPdaData(): Account not found.");
  }
  //decode account
  return TOKEN_ADDED_ACCOUNT_LAYOUT.decode(Buffer.from(account.data));
};

export const getClaimedPdaData = async (
  connection: Connection,
  address: PublicKey
) => {
  const account = await connection.getAccountInfo(address);
  if (account === null) {
    throw Error("TSX: getTokenAddedPdaData(): Account not found.");
  }
  //decode account
  return CLAIMED_DICTIONARY_LAYOUT.decode(Buffer.from(account.data));
};

export const getDailtTokenClaimsPdaData = async (
  connection: Connection,
  address: PublicKey
) => {
  const account = await connection.getAccountInfo(address);
  if (account === null) {
    throw Error("TSX: getTokenAddedPdaData(): Account not found.");
  }
  //decode account
  return DAILY_TOKEN_CLAIMS_DICTIONARY_LAYOUT.decode(Buffer.from(account.data));
};


export const getTokenListPdaData = async (
  connection: Connection,
  address: PublicKey
) => {
  const account = await connection.getAccountInfo(address);
  if (account === null) {
    throw Error("TSX: getTokenListPdaData(): Account not found.");
  }
  //decode account
  return TOKEN_LIST_DICTIONARY_LAYOUT.decode(Buffer.from(account.data));
};

export const construct = async (
  connection: Connection,
  programId: PublicKey,
  payer: Keypair,
  bridgePda: PublicKey,
  tokenAddedAccount: PublicKey,
  tokenListAccount: PublicKey,
  wPoktMint: PublicKey,
  verifyAddress: PublicKey,
  chainId: number,
  stableFee: number,
  bridgeTokenAccountPda: PublicKey
) => {
  const data = Buffer.alloc(BridgeInstruction.CONSTRUCT_LAYOUT.span);
  BridgeInstruction.CONSTRUCT_LAYOUT.encode(
    {
      instruction: BridgeInstruction.BridgeInstruction.Construct,
      wPoktAddress: wPoktMint,
      verifyAddress,
      chainId,
      stableFee,
    },
    data
  );

  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: bridgePda, isSigner: false, isWritable: true },
      { pubkey: tokenAddedAccount, isSigner: false, isWritable: true },
      { pubkey: tokenListAccount, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: bridgeTokenAccountPda, isSigner: false, isWritable: true },
      { pubkey: wPoktMint, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer]);
};

export const verifyConstruction = async (
  connection: Connection,
  // programId: PublicKey,
  owner: PublicKey,
  bridgePda: PublicKey,
  // tokenAddedAccount: PublicKey,
  tokenListAccount: PublicKey,
  wPoktMint: PublicKey
  // verifyAddress: PublicKey,
  // chainId: number,
  // stableFee: number
) => {
  // verify accounts
  await verifyBridgeData(connection, bridgePda, true, owner);
  await verifyTokenAddedData(connection, wPoktMint, true);
  await verifyTokenListData(
    connection,
    tokenListAccount,
    true,
    true,
    wPoktMint
  );
};

export const transferRequest = async (
  connection: Connection,
  programId: PublicKey,
  payer: Keypair,
  bridgePda: PublicKey,
  tokenListPda: PublicKey,
  mintAccount: PublicKey,
  fromTokenAccount: PublicKey,
  calculateFeeAccount: PublicKey,
  bridgeTokenAccount: PublicKey,
  fromAuth: Keypair,
  tokenIndex: number,
  to: PublicKey,
  amount: number,
  chainId: number
) => {
  const data = Buffer.alloc(BridgeInstruction.TRANSFER_REQUEST_LAYOUT.span);

  BridgeInstruction.TRANSFER_REQUEST_LAYOUT.encode(
    {
      instruction: BridgeInstruction.BridgeInstruction.TransferRequest,
      tokenIndex,
      to,
      amount,
      chainId,
    },
    data
  );

  const ix = new TransactionInstruction({
    programId,
    keys: [
      // { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: bridgePda, isSigner: false, isWritable: true },
      { pubkey: tokenListPda, isSigner: false, isWritable: true },
      { pubkey: mintAccount, isSigner: false, isWritable: true },
      { pubkey: fromTokenAccount, isSigner: false, isWritable: true },
      { pubkey: calculateFeeAccount, isSigner: false, isWritable: true },
      { pubkey: bridgeTokenAccount, isSigner: false, isWritable: true },
      { pubkey: fromAuth.publicKey, isSigner: true, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer, fromAuth]);
};

export const verifyTransferRequest = async (
  connection: Connection,
  bridgePda: PublicKey,
  sourceToken: PublicKey,
  bridgeTokenPda: PublicKey,
  sourceTokenExpectedBalance: number,
  bridgeTokenExpectedBalance: number,
  currentIndex: number
) => {
  const bridgeData = await getBridgePdaData(connection, bridgePda);
  const sourceTokenData = await SPLToken.getAccount(connection, sourceToken);
  const bridgeTokenData = await SPLToken.getAccount(connection, bridgeTokenPda);

  if (bridgeData.currentIndex !== currentIndex) {
    throw Error(
      `TSX - verifyTransferRequest(): Invalid current index: ${bridgeData.currentIndex}`
    );
  }

  if (sourceTokenData.amount !== BigInt(sourceTokenExpectedBalance)) {
    throw Error(
      `TSX - verifyTransferRequest(): Unequal Source token balance: ${sourceTokenData.amount}`
    );
  }

  if (bridgeTokenData.amount !== BigInt(bridgeTokenExpectedBalance)) {
    throw Error(
      `TSX - verifyTransferRequest(): Unequal Source token balance: ${bridgeTokenData.amount}`
    );
  }
};

export const transferReceipt = async (
  connection: Connection,
  programId: PublicKey,
  bridgePda: PublicKey,
  tokenListPda: PublicKey,
  claimedPda: PublicKey,
  dtcPda: PublicKey,
  mintAccount: PublicKey,
  from: PublicKey, // source token account
  fromAuth: Keypair, // source token account auth - the offline signer, 'signatureAccount'
  tokenIndex: number,
  to: PublicKey, // destination Token Account
  toAuth: Keypair,// destination Token Account Auth - the payer of the transaction
  amount: number,
  chainId: number,
  index: number,
)=>{
  const data = Buffer.alloc(BridgeInstruction.TRANSFER_RECEIPT_LAYOUT.span);

  BridgeInstruction.TRANSFER_RECEIPT_LAYOUT.encode(
    {
      instruction: BridgeInstruction.BridgeInstruction.TransferReceipt,
      tokenIndex,
      from: fromAuth.publicKey,
      to: toAuth.publicKey,
      amount,
      chainId,
      index,
      signatureAccount: fromAuth.publicKey
    },
    data
  );
 
  
  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: toAuth.publicKey, isSigner: true, isWritable: true },
      { pubkey: bridgePda, isSigner: false, isWritable: true },
      { pubkey: claimedPda, isSigner: false, isWritable: true },
      { pubkey: tokenListPda, isSigner: false, isWritable: true },
      { pubkey: dtcPda, isSigner: false, isWritable: true },
      { pubkey: fromAuth.publicKey, isSigner: true, isWritable: true },
      { pubkey: from, isSigner: false, isWritable: true },
      { pubkey: to, isSigner: false, isWritable: true },
      { pubkey: mintAccount, isSigner: false, isWritable: true },
      { pubkey: SPLToken.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },

    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [fromAuth, toAuth]);
}


export const verifyTokenAddedData = async (
  connection: Connection,
  tokenAddedPda: PublicKey,
  status: boolean
) => {
  const data = await getTokenAddedPdaData(connection, tokenAddedPda);
  if (!data.tokenAdded) {
    throw Error(`TSX - verifyTokenAddedData(): Token not added`);
  }
};

export const verifyTokenListData = async (
  connection: Connection,
  tokenListAccount: PublicKey,
  exists: boolean,
  isInitialized: boolean,
  tokenAddress: PublicKey
) => {
  const data = await getTokenListPdaData(connection, tokenListAccount);
  if (data.isInitialized !== isInitialized) {
    throw Error(
      `TSX - verifyTokenListData(): Bridge account Initialization status: ${data.isInitialized}`
    );
  }
  if (data.exists !== exists) {
    throw Error(
      `TSX - verifyTokenListData(): Bridge account Initialization status: ${data.isInitialized}`
    );
  }
  if (!data.tokenAddress.equals(tokenAddress)) {
    throw Error(
      `TSX - verifyTokenListData(): Verify Token Invalid Token Address: ${data.tokenAddress.toBase58()}`
    );
  }
};
export const verifyBridgeData = async (
  connection: Connection,
  bridgeAccountAddress: PublicKey,
  isInitialized: boolean,
  owner: PublicKey,
  verifyAddress?: PublicKey,
  feeUpdateDuration?: number,
  currentIndex?: number,
  chainId?: number,
  stableFeeUpdateTime?: number,
  stableFee?: number,
  newStableFee?: number
) => {
  const bridge = await getBridgePdaData(connection, bridgeAccountAddress);
  if (bridge.isInitialized !== isInitialized) {
    throw Error(
      `TSX - verifyBridgeData(): Bridge account Initialization status: ${bridge.isInitialized}`
    );
  }
  if (!bridge.owner.equals(owner)) {
    throw Error(
      `TSX - verifyBridgeData(): Bridge account Invalid Owner: ${bridge.owner.toBase58()}`
    );
  }
};

export const createClaimedDictionaryPdaAccount = async (
  connection: Connection,
  programId: PublicKey,
  payer: Keypair,
  claimedPda: PublicKey,
  index: number,
  chainId: number
) => {
  const data = Buffer.alloc(
    BridgeInstruction.CREATE_CLAIMED_DICTIONARY_PDA_ACCOUNT_LAYOUT.span
  );
  BridgeInstruction.CREATE_CLAIMED_DICTIONARY_PDA_ACCOUNT_LAYOUT.encode(
    {
      instruction:
        BridgeInstruction.BridgeInstruction.CreateClaimedDictionaryPdaAccount,
      index,
      chainId,
    },
    data
  );

  const ix = new TransactionInstruction({
    programId,
    keys: [
      // { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: claimedPda, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer]);
};

export const createDailyTokenClaimsDictionaryPdaAccount = async (
  connection: Connection,
  programId: PublicKey,
  payer: Keypair,
  dtcPda: PublicKey,
  tokenIndex: number
) => {
  const data = Buffer.alloc(
    BridgeInstruction.CREATE_DAILY_TOKEN_CLAIMS_DICTIONARY_PDA_ACCOUNT.span
  );
  BridgeInstruction.CREATE_DAILY_TOKEN_CLAIMS_DICTIONARY_PDA_ACCOUNT.encode(
    {
      instruction:
        BridgeInstruction.BridgeInstruction
          .CreateDailyTokenClaimsDictionaryPdaAccount,
      tokenIndex,
    },
    data
  );

  const ix = new TransactionInstruction({
    programId,
    keys: [
      // { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: dtcPda, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer]);
};
