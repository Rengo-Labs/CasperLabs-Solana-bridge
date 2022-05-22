import { bool, publicKey } from "@solana/buffer-layout-utils";
import { PublicKey } from "@solana/web3.js";
import { struct, Structure } from "@solana/buffer-layout";
/**
 * WPOKT state account interface
 */
export interface WPOKT {
  isInitialized: boolean;
  minter: PublicKey;
  mint: PublicKey;
  noncesDict: PublicKey;
  authorizationStateDict: PublicKey;
}

// /**
//  * Layout for WPOKT state struct
//  */
export const WPOKT_ACCOUNT_DATA_LAYOUT: Structure<WPOKT> = struct([
  bool("IsInitialized"),
  publicKey("minter"),
  publicKey("mint"),
  publicKey("noncesDict"),
  publicKey("authorizationStateDict"),
]);
