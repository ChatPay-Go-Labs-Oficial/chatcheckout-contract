import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




/**
 * Custom error types for the escrow contract
 */
export const EscrowError = {
  /**
   * Contract is already initialized
   */
  1: {message:"AlreadyInitialized"},
  /**
   * Contract configuration not found
   */
  2: {message:"ConfigNotInitialized"},
  /**
   * Unauthorized access
   */
  3: {message:"Unauthorized"},
  /**
   * Escrow not found
   */
  4: {message:"EscrowNotFound"},
  /**
   * Invalid amount (must be greater than 0)
   */
  5: {message:"InvalidAmount"},
  /**
   * Invalid fee basis points (must be 0-10000)
   */
  6: {message:"InvalidFeeBps"},
  /**
   * Invalid guarantee days (must be 1-36500)
   */
  7: {message:"InvalidGuaranteeDays"},
  /**
   * Invalid product ID (cannot be empty)
   */
  8: {message:"InvalidProductId"},
  /**
   * Escrow is not in active status
   */
  9: {message:"EscrowNotActive"},
  /**
   * Guarantee period not expired
   */
  10: {message:"GuaranteePeriodNotExpired"},
  /**
   * Guarantee period already expired
   */
  11: {message:"GuaranteePeriodExpired"},
  /**
   * Fee amount exceeds escrow amount
   */
  12: {message:"FeeExceedsAmount"},
  /**
   * Invalid signature for meta-transaction
   */
  13: {message:"InvalidSignature"},
  /**
   * Nonce mismatch - replay attack detected
   */
  14: {message:"InvalidNonce"},
  /**
   * Signature expired
   */
  15: {message:"SignatureExpired"},
  /**
   * Invalid function selector for meta-transaction
   */
  16: {message:"InvalidFunctionSelector"},
  /**
   * Counter overflow (too many escrows)
   */
  17: {message:"CounterOverflow"},
  /**
   * Token not in allowed list
   */
  18: {message:"TokenNotAllowed"}
}


/**
 * Event emitted when a token is added to the allowed list
 */
export interface TokenAddedEvent {
  token: string;
}


/**
 * Event emitted when a new escrow is created
 */
export interface CreateEscrowEvent {
  amount: i128;
  asset: string;
  buyer: string;
  escrow_id: u64;
  fee_bps: u32;
  guarantee_days: u32;
  product_id: string;
  seller: string;
}


/**
 * Event emitted when a token is removed from the allowed list
 */
export interface TokenRemovedEvent {
  token: string;
}


/**
 * Event emitted when refund is issued to buyer
 */
export interface RequestRefundEvent {
  amount: i128;
  asset: string;
  buyer: string;
  escrow_id: u64;
}


/**
 * Event emitted when payment is released to seller
 */
export interface ReleasePaymentEvent {
  amount: i128;
  escrow_id: u64;
  fee: i128;
  seller: string;
  to_seller: i128;
}


/**
 * Contract configuration
 */
export interface Config {
  admin: string;
  collect_on_create: boolean;
}

/**
 * Storage keys for the escrow contract
 */
export type DataKey = {tag: "Escrow", values: readonly [u64]} | {tag: "Counter", values: void} | {tag: "Config", values: void} | {tag: "Nonce", values: readonly [string]} | {tag: "AllowedTokens", values: void};


/**
 * Escrow data record
 */
export interface EscrowData {
  amount: i128;
  asset: string;
  buyer: string;
  created_at: u64;
  /**
 * Fee snapshot used at creation (comes as parameter from backend)
 */
fee_bps: u32;
  guarantee_days: u32;
  product_id: string;
  release_at: u64;
  seller: string;
  status: EscrowStatus;
}

/**
 * Status of an escrow
 */
export type EscrowStatus = {tag: "Active", values: void} | {tag: "Released", values: void} | {tag: "Refunded", values: void};

export interface Client {
  /**
   * Construct and simulate a get_nonce transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get current nonce for a user
   * Used to get the current nonce before calling functions that require it
   */
  get_nonce: ({user}: {user: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a get_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_config: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Config>>

  /**
   * Construct and simulate a get_escrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_escrow: ({escrow_id}: {escrow_id: u64}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<EscrowData>>>

  /**
   * Construct and simulate a create_escrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Create escrow with guarantee period
   */
  create_escrow: ({buyer, seller, amount, asset, fee_bps, guarantee_days, product_id}: {buyer: string, seller: string, amount: i128, asset: string, fee_bps: u32, guarantee_days: u32, product_id: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a update_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Update configuration (only current admin)
   */
  update_config: ({new_admin, collect_on_create}: {new_admin: string, collect_on_create: boolean}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a request_refund transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Refund (only buyer and within guarantee period)
   * Includes nonce for replay attack protection and traceability
   */
  request_refund: ({escrow_id, buyer, nonce}: {escrow_id: u64, buyer: string, nonce: u64}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a release_payment transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Release payment to seller (after guarantee period or by seller)
   * Includes nonce for replay attack protection and traceability
   */
  release_payment: ({escrow_id, seller, nonce}: {escrow_id: u64, seller: string, nonce: u64}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a is_token_allowed transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Check if a token is allowed (public helper function)
   */
  is_token_allowed: ({token}: {token: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a add_allowed_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Add a token to the allowed list (admin only)
   * This ensures only trusted tokens can be used in escrows
   */
  add_allowed_token: ({token}: {token: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_allowed_tokens transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get the list of allowed tokens (public)
   */
  get_allowed_tokens: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a remove_allowed_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Remove a token from the allowed list (admin only)
   * Note: Existing escrows with this token will still function
   */
  remove_allowed_token: ({token}: {token: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, collect_on_create}: {admin: string, collect_on_create: boolean},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({admin, collect_on_create}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAGNHZXQgY3VycmVudCBub25jZSBmb3IgYSB1c2VyClVzZWQgdG8gZ2V0IHRoZSBjdXJyZW50IG5vbmNlIGJlZm9yZSBjYWxsaW5nIGZ1bmN0aW9ucyB0aGF0IHJlcXVpcmUgaXQAAAAACWdldF9ub25jZQAAAAAAAAEAAAAAAAAABHVzZXIAAAATAAAAAQAAAAY=",
        "AAAAAAAAAAAAAAAKZ2V0X2NvbmZpZwAAAAAAAAAAAAEAAAfQAAAABkNvbmZpZwAA",
        "AAAAAAAAAAAAAAAKZ2V0X2VzY3JvdwAAAAAAAQAAAAAAAAAJZXNjcm93X2lkAAAAAAAABgAAAAEAAAPpAAAH0AAAAApFc2Nyb3dEYXRhAAAAAAfQAAAAC0VzY3Jvd0Vycm9yAA==",
        "AAAAAAAAACNDcmVhdGUgZXNjcm93IHdpdGggZ3VhcmFudGVlIHBlcmlvZAAAAAANY3JlYXRlX2VzY3JvdwAAAAAAAAcAAAAAAAAABWJ1eWVyAAAAAAAAEwAAAAAAAAAGc2VsbGVyAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAAAAAAHZmVlX2JwcwAAAAAEAAAAAAAAAA5ndWFyYW50ZWVfZGF5cwAAAAAABAAAAAAAAAAKcHJvZHVjdF9pZAAAAAAAEAAAAAEAAAPpAAAABgAAB9AAAAALRXNjcm93RXJyb3IA",
        "AAAAAAAAAClVcGRhdGUgY29uZmlndXJhdGlvbiAob25seSBjdXJyZW50IGFkbWluKQAAAAAAAA11cGRhdGVfY29uZmlnAAAAAAAAAgAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAAAAAARY29sbGVjdF9vbl9jcmVhdGUAAAAAAAABAAAAAA==",
        "AAAAAAAAAFRJbml0aWFsaXplIGNvbnRyYWN0IGNvbmZpZ3VyYXRpb24KQ2FuIG9ubHkgYmUgY2FsbGVkIG9uY2Ugb24gZGVwbG95IChvciB3aGVuIGVtcHR5KS4AAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAARY29sbGVjdF9vbl9jcmVhdGUAAAAAAAABAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAALRXNjcm93RXJyb3IA",
        "AAAAAAAAAGxSZWZ1bmQgKG9ubHkgYnV5ZXIgYW5kIHdpdGhpbiBndWFyYW50ZWUgcGVyaW9kKQpJbmNsdWRlcyBub25jZSBmb3IgcmVwbGF5IGF0dGFjayBwcm90ZWN0aW9uIGFuZCB0cmFjZWFiaWxpdHkAAAAOcmVxdWVzdF9yZWZ1bmQAAAAAAAMAAAAAAAAACWVzY3Jvd19pZAAAAAAAAAYAAAAAAAAABWJ1eWVyAAAAAAAAEwAAAAAAAAAFbm9uY2UAAAAAAAAGAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAALRXNjcm93RXJyb3IA",
        "AAAAAAAAAHxSZWxlYXNlIHBheW1lbnQgdG8gc2VsbGVyIChhZnRlciBndWFyYW50ZWUgcGVyaW9kIG9yIGJ5IHNlbGxlcikKSW5jbHVkZXMgbm9uY2UgZm9yIHJlcGxheSBhdHRhY2sgcHJvdGVjdGlvbiBhbmQgdHJhY2VhYmlsaXR5AAAAD3JlbGVhc2VfcGF5bWVudAAAAAADAAAAAAAAAAllc2Nyb3dfaWQAAAAAAAAGAAAAAAAAAAZzZWxsZXIAAAAAABMAAAAAAAAABW5vbmNlAAAAAAAABgAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAC0VzY3Jvd0Vycm9yAA==",
        "AAAAAAAAADRDaGVjayBpZiBhIHRva2VuIGlzIGFsbG93ZWQgKHB1YmxpYyBoZWxwZXIgZnVuY3Rpb24pAAAAEGlzX3Rva2VuX2FsbG93ZWQAAAABAAAAAAAAAAV0b2tlbgAAAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAGRBZGQgYSB0b2tlbiB0byB0aGUgYWxsb3dlZCBsaXN0IChhZG1pbiBvbmx5KQpUaGlzIGVuc3VyZXMgb25seSB0cnVzdGVkIHRva2VucyBjYW4gYmUgdXNlZCBpbiBlc2Nyb3dzAAAAEWFkZF9hbGxvd2VkX3Rva2VuAAAAAAAAAQAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAA==",
        "AAAAAAAAACdHZXQgdGhlIGxpc3Qgb2YgYWxsb3dlZCB0b2tlbnMgKHB1YmxpYykAAAAAEmdldF9hbGxvd2VkX3Rva2VucwAAAAAAAAAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAGxSZW1vdmUgYSB0b2tlbiBmcm9tIHRoZSBhbGxvd2VkIGxpc3QgKGFkbWluIG9ubHkpCk5vdGU6IEV4aXN0aW5nIGVzY3Jvd3Mgd2l0aCB0aGlzIHRva2VuIHdpbGwgc3RpbGwgZnVuY3Rpb24AAAAUcmVtb3ZlX2FsbG93ZWRfdG9rZW4AAAABAAAAAAAAAAV0b2tlbgAAAAAAABMAAAAA",
        "AAAABAAAACpDdXN0b20gZXJyb3IgdHlwZXMgZm9yIHRoZSBlc2Nyb3cgY29udHJhY3QAAAAAAAAAAAALRXNjcm93RXJyb3IAAAAAEgAAAB9Db250cmFjdCBpcyBhbHJlYWR5IGluaXRpYWxpemVkAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAAEAAAAgQ29udHJhY3QgY29uZmlndXJhdGlvbiBub3QgZm91bmQAAAAUQ29uZmlnTm90SW5pdGlhbGl6ZWQAAAACAAAAE1VuYXV0aG9yaXplZCBhY2Nlc3MAAAAADFVuYXV0aG9yaXplZAAAAAMAAAAQRXNjcm93IG5vdCBmb3VuZAAAAA5Fc2Nyb3dOb3RGb3VuZAAAAAAABAAAACdJbnZhbGlkIGFtb3VudCAobXVzdCBiZSBncmVhdGVyIHRoYW4gMCkAAAAADUludmFsaWRBbW91bnQAAAAAAAAFAAAAKkludmFsaWQgZmVlIGJhc2lzIHBvaW50cyAobXVzdCBiZSAwLTEwMDAwKQAAAAAADUludmFsaWRGZWVCcHMAAAAAAAAGAAAAKEludmFsaWQgZ3VhcmFudGVlIGRheXMgKG11c3QgYmUgMS0zNjUwMCkAAAAUSW52YWxpZEd1YXJhbnRlZURheXMAAAAHAAAAJEludmFsaWQgcHJvZHVjdCBJRCAoY2Fubm90IGJlIGVtcHR5KQAAABBJbnZhbGlkUHJvZHVjdElkAAAACAAAAB5Fc2Nyb3cgaXMgbm90IGluIGFjdGl2ZSBzdGF0dXMAAAAAAA9Fc2Nyb3dOb3RBY3RpdmUAAAAACQAAABxHdWFyYW50ZWUgcGVyaW9kIG5vdCBleHBpcmVkAAAAGUd1YXJhbnRlZVBlcmlvZE5vdEV4cGlyZWQAAAAAAAAKAAAAIEd1YXJhbnRlZSBwZXJpb2QgYWxyZWFkeSBleHBpcmVkAAAAFkd1YXJhbnRlZVBlcmlvZEV4cGlyZWQAAAAAAAsAAAAgRmVlIGFtb3VudCBleGNlZWRzIGVzY3JvdyBhbW91bnQAAAAQRmVlRXhjZWVkc0Ftb3VudAAAAAwAAAAmSW52YWxpZCBzaWduYXR1cmUgZm9yIG1ldGEtdHJhbnNhY3Rpb24AAAAAABBJbnZhbGlkU2lnbmF0dXJlAAAADQAAACdOb25jZSBtaXNtYXRjaCAtIHJlcGxheSBhdHRhY2sgZGV0ZWN0ZWQAAAAADEludmFsaWROb25jZQAAAA4AAAARU2lnbmF0dXJlIGV4cGlyZWQAAAAAAAAQU2lnbmF0dXJlRXhwaXJlZAAAAA8AAAAuSW52YWxpZCBmdW5jdGlvbiBzZWxlY3RvciBmb3IgbWV0YS10cmFuc2FjdGlvbgAAAAAAF0ludmFsaWRGdW5jdGlvblNlbGVjdG9yAAAAABAAAAAjQ291bnRlciBvdmVyZmxvdyAodG9vIG1hbnkgZXNjcm93cykAAAAAD0NvdW50ZXJPdmVyZmxvdwAAAAARAAAAGVRva2VuIG5vdCBpbiBhbGxvd2VkIGxpc3QAAAAAAAAPVG9rZW5Ob3RBbGxvd2VkAAAAABI=",
        "AAAAAQAAADdFdmVudCBlbWl0dGVkIHdoZW4gYSB0b2tlbiBpcyBhZGRlZCB0byB0aGUgYWxsb3dlZCBsaXN0AAAAAAAAAAAPVG9rZW5BZGRlZEV2ZW50AAAAAAEAAAAAAAAABXRva2VuAAAAAAAAEw==",
        "AAAAAQAAACpFdmVudCBlbWl0dGVkIHdoZW4gYSBuZXcgZXNjcm93IGlzIGNyZWF0ZWQAAAAAAAAAAAARQ3JlYXRlRXNjcm93RXZlbnQAAAAAAAAIAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAAAAAAFYnV5ZXIAAAAAAAATAAAAAAAAAAllc2Nyb3dfaWQAAAAAAAAGAAAAAAAAAAdmZWVfYnBzAAAAAAQAAAAAAAAADmd1YXJhbnRlZV9kYXlzAAAAAAAEAAAAAAAAAApwcm9kdWN0X2lkAAAAAAAQAAAAAAAAAAZzZWxsZXIAAAAAABM=",
        "AAAAAQAAADtFdmVudCBlbWl0dGVkIHdoZW4gYSB0b2tlbiBpcyByZW1vdmVkIGZyb20gdGhlIGFsbG93ZWQgbGlzdAAAAAAAAAAAEVRva2VuUmVtb3ZlZEV2ZW50AAAAAAAAAQAAAAAAAAAFdG9rZW4AAAAAAAAT",
        "AAAAAQAAACxFdmVudCBlbWl0dGVkIHdoZW4gcmVmdW5kIGlzIGlzc3VlZCB0byBidXllcgAAAAAAAAASUmVxdWVzdFJlZnVuZEV2ZW50AAAAAAAEAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAAAAAAFYnV5ZXIAAAAAAAATAAAAAAAAAAllc2Nyb3dfaWQAAAAAAAAG",
        "AAAAAQAAADBFdmVudCBlbWl0dGVkIHdoZW4gcGF5bWVudCBpcyByZWxlYXNlZCB0byBzZWxsZXIAAAAAAAAAE1JlbGVhc2VQYXltZW50RXZlbnQAAAAABQAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAllc2Nyb3dfaWQAAAAAAAAGAAAAAAAAAANmZWUAAAAACwAAAAAAAAAGc2VsbGVyAAAAAAATAAAAAAAAAAl0b19zZWxsZXIAAAAAAAAL",
        "AAAAAQAAABZDb250cmFjdCBjb25maWd1cmF0aW9uAAAAAAAAAAAABkNvbmZpZwAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABFjb2xsZWN0X29uX2NyZWF0ZQAAAAAAAAE=",
        "AAAAAgAAACRTdG9yYWdlIGtleXMgZm9yIHRoZSBlc2Nyb3cgY29udHJhY3QAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAEAAAAAAAAABkVzY3JvdwAAAAAAAQAAAAYAAAAAAAAAAAAAAAdDb3VudGVyAAAAAAAAAAAAAAAABkNvbmZpZwAAAAAAAQAAAAAAAAAFTm9uY2UAAAAAAAABAAAAEwAAAAAAAAAAAAAADUFsbG93ZWRUb2tlbnMAAAA=",
        "AAAAAQAAABJFc2Nyb3cgZGF0YSByZWNvcmQAAAAAAAAAAAAKRXNjcm93RGF0YQAAAAAACgAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAVhc3NldAAAAAAAABMAAAAAAAAABWJ1eWVyAAAAAAAAEwAAAAAAAAAKY3JlYXRlZF9hdAAAAAAABgAAAD9GZWUgc25hcHNob3QgdXNlZCBhdCBjcmVhdGlvbiAoY29tZXMgYXMgcGFyYW1ldGVyIGZyb20gYmFja2VuZCkAAAAAB2ZlZV9icHMAAAAABAAAAAAAAAAOZ3VhcmFudGVlX2RheXMAAAAAAAQAAAAAAAAACnByb2R1Y3RfaWQAAAAAABAAAAAAAAAACnJlbGVhc2VfYXQAAAAAAAYAAAAAAAAABnNlbGxlcgAAAAAAEwAAAAAAAAAGc3RhdHVzAAAAAAfQAAAADEVzY3Jvd1N0YXR1cw==",
        "AAAAAgAAABNTdGF0dXMgb2YgYW4gZXNjcm93AAAAAAAAAAAMRXNjcm93U3RhdHVzAAAAAwAAAAAAAAAAAAAABkFjdGl2ZQAAAAAAAAAAAAAAAAAIUmVsZWFzZWQAAAAAAAAAAAAAAAhSZWZ1bmRlZA==" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_nonce: this.txFromJSON<u64>,
        get_config: this.txFromJSON<Config>,
        get_escrow: this.txFromJSON<Result<EscrowData>>,
        create_escrow: this.txFromJSON<Result<u64>>,
        update_config: this.txFromJSON<null>,
        request_refund: this.txFromJSON<Result<void>>,
        release_payment: this.txFromJSON<Result<void>>,
        is_token_allowed: this.txFromJSON<boolean>,
        add_allowed_token: this.txFromJSON<null>,
        get_allowed_tokens: this.txFromJSON<Array<string>>,
        remove_allowed_token: this.txFromJSON<null>
  }
}