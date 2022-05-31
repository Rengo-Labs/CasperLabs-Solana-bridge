import { PublicKey } from "@solana/web3.js";

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
