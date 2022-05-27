import * as BufferLayout from "@solana/buffer-layout";
import * as BufferLayoutUtils from "@solana/buffer-layout-utils";
import { PublicKey } from "@solana/web3.js";
/** Instructions defined by the program */

export enum WPOKTInstruction {
  /// Accounts Expected
  /// 0. `[writable]` The account to initialize as mint account
  /// 1. `[writable]` The WPOKT global state account
  Construct,
  /// Accounts Expected
  /// 0. `[]` The WPOKT global state account
  /// 1. `[writeable]` The Mint account
  /// 2. `[signer]` The mint authority
  /// 3. `[writeable]` the token account to mint to
  MintOnlyMinter,
  /// Accounts Expected
  /// 0. `[writeable]` The WPOKT global state account
  /// 1. `[writeable]` The Mint account
  /// 2. `[signer]` The mint authority
  /// 3. `[writeable]` The new mint authority
  ChangeMinterOnlyMinter,
  ///   0. `[writable]` The NoncesDictionary account
  ///   1. `[writable]` The source account.
  ///   2. `[]` The delegate.
  ///   3. `[signer]` The source account owner.
  Permit,
  ///   0. `[writable]` The AuthorizationState account
  ///   1. `[writable]` The mint account.
  ///   2. `[writable]` The source token account.
  ///   3. `[signer]` The source token account owner.
  ///   3. `[writable]` The destination token account.
  TransferWithAuthorization,
  ///   0. `[writable]` The NoncesDictionary PDA account
  InitializeNoncePdaAccount,
  ///   0. `[writable]` The AuthorizationState PDA account
  InitializeAuthorizationStatePdaAccount,
}

export interface Construct {
  instruction: number;
  initialMinter: PublicKey;
}

export const CONSTRUCT_LAYOUT: BufferLayout.Layout<Construct> =
  BufferLayout.struct([
    BufferLayout.u8("instruction"),
    BufferLayoutUtils.publicKey("initialMinter"),
  ]);

export interface MintOnlyMinter {
  instruction: number;
  to: PublicKey;
  value: number;
}

export const MINT_ONLY_MINTER_LAYOUT: BufferLayout.Layout<MintOnlyMinter> =
  BufferLayout.struct([
    BufferLayout.u8("instruction"),
    BufferLayoutUtils.publicKey("to"),
    BufferLayout.nu64("value"),
  ]);

export interface ChangeMinterOnlyMinter {
  instruction: number;
  newMinter: PublicKey;
}
export const CHANGE_MINTER_ONLY_MINTER_LAYOUT: BufferLayout.Layout<ChangeMinterOnlyMinter> =
  BufferLayout.struct([
    BufferLayout.u8("instruction"),
    BufferLayoutUtils.publicKey("newMinter"),
  ]);

export interface Permit {
  instruction: number;
  owner: PublicKey;
  spender: PublicKey;
  value: number;
  deadline: number;
}

export const PERMIT_LAYOUT: BufferLayout.Layout<Permit> = BufferLayout.struct([
  BufferLayout.u8("instruction"),
  BufferLayoutUtils.publicKey("owner"),
  BufferLayoutUtils.publicKey("spender"),
  BufferLayout.nu64("value"),
  BufferLayout.nu64("deadline"),
]);

export interface TransferWithAuthorization {
  instruction: number;
  from: PublicKey;
  to: PublicKey;
  value: number;
  validAfter: number;
  validBefore: number;
  nonce: String;
}

export const TRANSFER_WITH_AUTHORIZATION_LAYOUT: BufferLayout.Layout<TransferWithAuthorization> =
  BufferLayout.struct([
    BufferLayout.u8("instruction"),
    BufferLayoutUtils.publicKey("from"),
    BufferLayoutUtils.publicKey("to"),
    BufferLayout.nu64("value"),
    BufferLayout.nu64("validAfter"),
    BufferLayout.nu64("validBefore"),
    BufferLayout.cstr("nonce"),
  ]);

export interface InitializeNoncePdaAccount {
  instruction: number;
  owner: PublicKey;
}

export const INITIALIZE_NONCE_PDA_ACCOUNT_LAYOUT =
  BufferLayout.struct<InitializeNoncePdaAccount>([
    BufferLayout.u8("instruction"),
    BufferLayoutUtils.publicKey("owner"),
  ]);

export interface InitializeAuthorizationStatePdaAccount {
  instruction: number;
  from: PublicKey;
  nonce: Uint8Array;
}

export const INITIALIZE_AUTHORIZATION_STATE_PDA_ACCOUNT_LAYOUT =
  BufferLayout.struct<InitializeAuthorizationStatePdaAccount>([
    BufferLayout.u8("instruction"),
    BufferLayoutUtils.publicKey("from"),
    BufferLayout.blob(32, "nonce"),
  ]);
