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
import { off } from "process";
import * as BridgeInstruction from "./instructions";
import {
  BRIDGE_LAYOUT,
  TOKEN_ADDED_ACCOUNT_LAYOUT,
  TOKEN_LIST_DICTIONARY_LAYOUT,
} from "./state";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

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
    Buffer.of(chainId),
    Buffer.of(index),
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
  index: number
): Promise<[PublicKey, number]> => {
  const seeds: Uint8Array[] = [
    Buffer.of(index),
    Buffer.from("bridge"),
    Buffer.from("daily_token_claims_dictionary_key"),
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

export const verifyTokenAddedData = async (
  connection: Connection,
  tokenAddedPda: PublicKey,
  status: boolean
) => {
  const data = await getTokenAddedPdaData(connection, tokenAddedPda);
  if (!data.tokenAdded) {
    throw Error(`Token not added`);
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
    throw Error(`Bridge account Initialization status: ${data.isInitialized}`);
  }
  if (data.exists !== exists) {
    throw Error(`Bridge account Initialization status: ${data.isInitialized}`);
  }
  if (!data.tokenAddress.equals(tokenAddress)) {
    throw Error(
      `Verify Token Invalid Token Address: ${data.tokenAddress.toBase58()}`
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
      `Bridge account Initialization status: ${bridge.isInitialized}`
    );
  }
  if (!bridge.owner.equals(owner)) {
    throw Error(`Bridge account Invalid Owner: ${bridge.owner.toBase58()}`);
  }
};
