import { PublicKey } from "@solana/web3.js";
import { nu64, struct, u8 } from "@solana/buffer-layout";
import { publicKey } from "@solana/buffer-layout-utils";

export enum BridgeInstruction {
  /// Initialize storage accounts for Bridge
  ///
  /// Accounts expected
  /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
  /// 1. `[writeable]` The account used as global storage of bridge
  /// 2. `[writeable]` The account used as 'claimed' dictionary
  /// 3. `[writeable]` The account used as 'token_list' dictionary
  /// 4. `[writeable]` The account used as 'daily_token_claims' dictionary
  /// 5. `[writeable]` The account used as 'token_added' dictionary
  Construct,
  /// Acounts expected
  ///
  /// 0. `[]` The account of person initializing bridge - the 'owner'.
  /// 1. `[writeable]` The account used as global storage of Bridge program
  /// 2. `[writeable]` The account used as 'token_list' dictionary
  /// 3. `[]` the token mint account found at 'token_index' in token_list dictionary
  /// 4. `[writable]` the token account of token sender
  /// 5. `[writeable]` the CalculateFeeResult account
  /// 6. `[writable]` the Token account of Bridge
  /// 7. `[signer]` the sender token account's owner
  TransferRequest,
  /// Accounts expected
  /// 0. `[writeable]` The account used as global storage of Bridge program
  /// 1. `[writeable]` The account used as 'claimed' dictionary
  /// 2. `[writeable]` The account used as 'token_list' dictionary
  /// 3. `[writable]` The account used as 'daily_token_claims' dictionary
  /// 4. `[signer]` The signatory account.
  /// 5. `[writeable]` The token mint account for this token data's mint.
  /// 6. `[signer, writeable]` Bridge's associated token account for this mint.
  /// 7. `[signer, writeable]` receiver's associated token account for this mint.
  /// 8. `[signer]` The bridge PDA account
  TransferReceipt,
  /// Accounts expected
  /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
  /// 1. `[writeable]` The account used as global storage of bridge
  UpdateVerifyAddressOnlyOwner,
  /// Accounts expected
  /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
  /// 1. `[]` The account used as global storage of bridge
  /// 2. `[writable]` The account used as 'token_list' dictionary
  UpdateTokenLimitOnlyOwner,
  /// Accounts expected
  /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
  /// 1. `[]` The account used as global storage of bridge
  /// 2. `[writable]` The account used as 'token_list' dictionary
  SetTokenLimitTimeOnlyOwner,
  /// Accounts expected
  /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
  /// 1. `[writable]` The account used as global storage of bridge
  UpdateStableFeeOnlyOwner,
  /// Accounts expected
  /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
  /// 1. `[]` The account used as global storage of bridge
  /// 2. `[ writable]` The account used as 'token_list' dictionary
  UpdateTokenFeeOnlyOwner,
  /// Accounts expected
  /// 0. `[]` The account used as global storage of bridge
  /// 1. `[]` The account used as 'token_list' dictionary
  UpdateFees,
  /// 0. `[signer, writeable]` Owner Account
  /// 1. `[writeable]` Owner Token Account
  /// 2. `[]` Bridge Account
  /// 3. `[signer]` Bridge Pda Account
  /// 4. `[writeable]` Bridge Token Account, To be created in the same transaction as WithdrawFees, authority set to Bridge PDA Account
  /// 5. `[writeable]` Mint Account at token_index
  /// 6. `[writeable]` Token List Account
  WithdrawFeesOnlyOwner,
  /// Accounts expected
  /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
  /// 1. `[]` The account used as global storage of bridge
  /// 2. `[writable]` The account used as 'token_list' dictionary
  /// 3. `[writable]` The account used as 'token_added' dictionary
  AddTokenOnlyOwner,
  /// Accounts expected
  /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
  /// 1. `[]` The account used as global storage of bridge
  /// 2. `[writable]` The account used as 'token_list' dictionary
  PauseTokenOnlyOwner,
  /// Accounts expected
  /// 0. `[signer]` The account of person initializing bridge - the 'owner'.
  /// 1. `[]` The account used as global storage of bridge
  /// 2. `[writable]` The account used as 'token_list' dictionary
  /// 3. `[writable]` The account used as 'daily_token_claims' dictionary
  ///
  UnpauseTokenOnlyOwner,
  /// Accounts expected
  /// 0. `[]` The account used as global storage of bridge
  /// 1. `[]` The account used as 'token_list' dictionary
  /// 2. `[]` The account used as 'calculate_fee_result' account
  CalculateFee,
  /// Accounts expected:
  /// 0. `[signer]` The program owner's account.
  /// 1. `[writeable]` The account used as WPokt's global state
  RenounceOwnership,
  /// Accounts expected:
  /// 0. `[signer]` The program owner's account.
  /// 1. `[writeable]` The account used as WPokt's global state
  TransferOwnership,
  CreateClaimedDictionaryPdaAccount,
  CreateDailyTokenClaimsDictionaryPdaAccount,
}

export interface Construct {
  instruction: number;
  wPoktAddress: PublicKey;
  verifyAddress: PublicKey;
  chainId: number;
  stableFee: number;
}

export const CONSTRUCT_LAYOUT = struct<Construct>([
  u8("instruction"),
  publicKey("wPoktAddress"),
  publicKey("verifyAddress"),
  nu64("chainId"),
  nu64("stableFee"),
]);

export interface TransferRequest {
  instruction: number;
  tokenIndex: number;
  to: PublicKey;
  amount: number;
  chainId: number;
}

export const TRANSFER_REQUEST_LAYOUT = struct<TransferRequest>([
  u8("instruction"),
  nu64("tokenIndex"),
  publicKey("to"),
  nu64("amount"),
  nu64("chainId"),
]);

export interface TransferReceipt {
  instruction: number;
  tokenIndex: number;
  from: PublicKey;
  to: PublicKey;
  amount: number;
  chainId: number;
  index: number;
  signatureAccount: PublicKey;
}

export const TRANSFER_RECEIPT_LAYOUT = struct<TransferReceipt>([
  u8("instruction"),
  nu64("tokenIndex"),
  publicKey("from"),
  publicKey("to"),
  nu64("amount"),
  nu64("chainId"),
  nu64("index"),
  publicKey("signatureAccount"),
]);

export interface CreateClaimedDictionaryPdaAccount {
  instruction: number;
  index: number;
  chainId: number;
}

export const CREATE_CLAIMED_DICTIONARY_PDA_ACCOUNT_LAYOUT =
  struct<CreateClaimedDictionaryPdaAccount>([
    u8("instruction"),
    nu64("index"),
    nu64("chainId"),
  ]);
export interface CreateDailyTokenClaimsDictionaryPdaAccount {
  instruction: number;
  tokenIndex: number;
}

export const CREATE_DAILY_TOKEN_CLAIMS_DICTIONARY_PDA_ACCOUNT =
  struct<CreateDailyTokenClaimsDictionaryPdaAccount>([
    u8("instruction"),
    nu64("tokenIndex"),
  ]);
