import { bool, publicKey, u64 } from "@solana/buffer-layout-utils";
import { PublicKey } from "@solana/web3.js";
import { struct, Structure, blob } from "@solana/buffer-layout";
/**
 * WPOKT state account interface
 */
export interface WPOKT {
  isInitialized: boolean;
  minter: PublicKey;
  mint: PublicKey;
}

// /**
//  * Layout for WPOKT state struct
//  */
export const WPOKT_ACCOUNT_DATA_LAYOUT: Structure<WPOKT> = struct([
  bool("IsInitialized"),
  publicKey("minter"),
  publicKey("mint"),
]);

export interface NoncesDictionary {
  owner: PublicKey;
  nonce: bigint;
}

// /**
//  * Layout for NoncesDictionary item struct
//  */
export const WPOKT_NONCES_DICTIONARY_LAYOUT: Structure<NoncesDictionary> =
  struct([publicKey("owner"), u64("nonce")]);

export interface AuthorizationStateDictionary {
  from: PublicKey;
  nonce: Uint8Array;
  authorization: boolean;
}

// /**
//  * Layout for AuthorizationStateDictionary item struct
//  */
export const WPOKT_AUTHORIZATION_DICTIONARY_LAYOUT =
  struct<AuthorizationStateDictionary>([
    publicKey("from"),
    blob(32, "nonce"),
    bool("authorization"),
  ]);
