import { bool, publicKey } from "@solana/buffer-layout-utils";
import { PublicKey } from "@solana/web3.js";
import { struct, Structure } from "@solana/buffer-layout";
/**
 * WPokt state account interface
 */
export interface WPoktLayout {
  isInitialized: boolean;
  bridgeAddress: PublicKey;
  owner: PublicKey;
  mint: PublicKey;
}

// /**
//  * Layout for WPokt state struct
//  */
// bufferLayout.str
export const W_POKT_ACCOUNT_DATA_LAYOUT: Structure<WPoktLayout> = struct([
  bool("IsInitialized"),
  publicKey("bridgeAddress"),
  publicKey("owner"),
  publicKey("mint"),
]);
