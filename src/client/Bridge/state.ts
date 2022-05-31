import { PublicKey } from "@solana/web3.js";
import { nu64, struct } from "@solana/buffer-layout";
import { bool } from "@solana/buffer-layout-utils";
import { publicKey } from "@solana/buffer-layout-utils";

export interface Bridge {
  isInitialized: boolean; // 1 byte
  owner: PublicKey; // 32 bytes
  feeUpdateDuration: number; //8 bytes
  verifyAddress: PublicKey; //32 bytes
  currentIndex: number; //8 bytes
  chainId: number; //8 bytes
  stableFeeUpdateTime: number; //8 bytes
  stableFee: number; //8 bytes
  newStableFee: number; //8 bytes
}

export const BRIDGE_LAYOUT = struct<Bridge>([
  bool("isInitialized"),
  publicKey("owner"),
  nu64("feeUpdateDuration"),
  publicKey("verifyAddress"),
  nu64("currentIndex"),
  nu64("chainId"),
  nu64("stableFeeUpdateTime"),
  nu64("stableFee"),
  nu64("newStableFee"),
]);

/**
 * Reusable as TokenAddedDictionary interface
 */
export interface ClaimedDictionary {
  claimed: boolean;
}

/**
 * Reusable as TokenAddedDictionary Layout
 */
export const CLAIMED_DICTIONARY_LAYOUT = struct<ClaimedDictionary>([
  bool("claimed"),
]);

export interface TokenListDictionary {
  isInitialized: boolean; // 1B
  tokenAddress: PublicKey; // 32B
  exists: boolean; // 1B
  paused: boolean; // 1B
  totalFeesCollected: number; //8B
  fee: number; //8B
  feeUpdateTime: number; //8B
  newFee: number; //8B
  limit: number; //8B
  limitTimestamp: number; //8B
}

export const TOKEN_LIST_DICTIONARY_LAYOUT = struct<TokenListDictionary>([
  bool("isInitialized"),
  publicKey("tokenAddress"),
  bool("exists"),
  bool("paused"),
  nu64("totalFeesCollected"),
  nu64("fee"),
  nu64("feeUpdateTime"),
  nu64("newFee"),
  nu64("limit"),
  nu64("limitTimestamp"),
]);

/**
 * Reusable as CalcuateFeeResult interface
 */
export interface DailyTokenClaimsDictionary {
  dailyTokenClaims: number;
}

/**
 * Reusable as CalcuateFeeResult Layout
 */
export const DAILY_TOKEN_CLAIMS_DICTIONARY_LAYOUT =
  struct<DailyTokenClaimsDictionary>([nu64("dailyTokenClaims")]);
