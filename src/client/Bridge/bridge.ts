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
import * as BridgeInstruction from "./instructions";
import * as BridgeState from "./state";

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

export const generateTokenListDictionaryPda = async (
  programId: PublicKey,
  index: number
): Promise<[PublicKey, number]> => {
  const seeds: Uint8Array[] = [
    Buffer.of(index),
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
    Buffer.from("token_added_dictionary"),
  ];

  const [pda, seedBump] = await PublicKey.findProgramAddress(seeds, programId);
  return [pda, seedBump];
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
  stableFee: number
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
    ],
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer]);
};

// export const verifyConstruction = async (
//   connection: Connection,
//   programId: PublicKey,
//   payer: Keypair,
//   bridgePda: PublicKey,
//   tokenAddedAccount: PublicKey,
//   tokenListAccount: PublicKey,
//   wPoktMint: PublicKey,
//   verifyAddress: PublicKey,
//   chainId: number,
//   stableFee: number
// ) => {};
