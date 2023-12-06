import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Approve {
    'status' : TransactionState,
    'signer' : string,
    'created_date' : bigint,
}
export interface BasicTransactionFields {
    'id' : bigint,
    'transaction_type' : TrType,
    'threshold' : [] | [number],
    'initiator' : string,
    'modified_date' : bigint,
    'memo' : [] | [string],
    'state' : TransactionState,
    'approves' : Array<Approve>,
    'is_vault_state' : boolean,
    'created_date' : bigint,
    'batch_uid' : [] | [string],
}
export interface Conf {
    'origins' : [] | [Array<string>],
    'ledger_canister_id' : Principal,
}
export type Currency = { 'ICP' : null };
export interface Member {
    'modified_date' : bigint,
    'name' : string,
    'role' : VaultRole,
    'state' : ObjectState,
    'member_id' : string,
    'created_date' : bigint,
}
export interface MemberCreateTransaction {
    'name' : string,
    'role' : VaultRole,
    'member_id' : string,
    'common' : BasicTransactionFields,
}
export interface MemberCreateTransactionRequest {
    'name' : string,
    'role' : VaultRole,
    'member_id' : string,
    'batch_uid' : [] | [string],
}
export interface MemberRemoveTransaction {
    'member_id' : string,
    'common' : BasicTransactionFields,
}
export interface MemberRemoveTransactionRequest {
    'member_id' : string,
    'batch_uid' : [] | [string],
}
export interface MemberUpdateNameTransaction {
    'name' : string,
    'member_id' : string,
    'common' : BasicTransactionFields,
}
export interface MemberUpdateNameTransactionRequest {
    'name' : string,
    'member_id' : string,
    'batch_uid' : [] | [string],
}
export interface MemberUpdateRoleTransaction {
    'role' : VaultRole,
    'member_id' : string,
    'common' : BasicTransactionFields,
}
export interface MemberUpdateRoleTransactionRequest {
    'role' : VaultRole,
    'member_id' : string,
    'batch_uid' : [] | [string],
}
export type Network = { 'IC' : null } |
    { 'BTC' : null } |
    { 'ETH' : null };
export type ObjectState = { 'Active' : null } |
    { 'Archived' : null };
export interface Policy {
    'uid' : string,
    'member_threshold' : number,
    'modified_date' : bigint,
    'amount_threshold' : bigint,
    'wallets' : Array<string>,
    'currency' : Currency,
    'created_date' : bigint,
}
export interface PolicyCreateTransaction {
    'uid' : string,
    'member_threshold' : number,
    'amount_threshold' : bigint,
    'wallets' : Array<string>,
    'currency' : Currency,
    'common' : BasicTransactionFields,
}
export interface PolicyCreateTransactionRequest {
    'member_threshold' : number,
    'amount_threshold' : bigint,
    'wallets' : Array<string>,
    'currency' : Currency,
    'batch_uid' : [] | [string],
}
export interface PolicyRemoveTransaction {
    'uid' : string,
    'common' : BasicTransactionFields,
}
export interface PolicyRemoveTransactionRequest {
    'uid' : string,
    'batch_uid' : [] | [string],
}
export interface PolicyUpdateTransaction {
    'uid' : string,
    'member_threshold' : number,
    'amount_threshold' : bigint,
    'common' : BasicTransactionFields,
}
export interface PolicyUpdateTransactionRequest {
    'uid' : string,
    'member_threshold' : number,
    'amount_threshold' : bigint,
    'batch_uid' : [] | [string],
}
export interface Quorum { 'modified_date' : bigint, 'quorum' : number }
export interface QuorumUpdateTransaction {
    'common' : BasicTransactionFields,
    'quorum' : number,
}
export interface QuorumUpdateTransactionRequest {
    'quorum' : number,
    'batch_uid' : [] | [string],
}
export type TrType = { 'WalletUpdateName' : null } |
    { 'MemberCreate' : null } |
    { 'PolicyRemove' : null } |
    { 'WalletCreate' : null } |
    { 'PolicyCreate' : null } |
    { 'MemberRemove' : null } |
    { 'PolicyUpdate' : null } |
    { 'MemberUpdateName' : null } |
    { 'VaultNamingUpdate' : null } |
    { 'MemberUpdateRole' : null } |
    { 'QuorumUpdate' : null } |
    { 'Transfer' : null };
export interface TransactionApproveRequest {
    'transaction_id' : bigint,
    'state' : TransactionState,
}
export type TransactionCandid = {
    'WalletCreateTransactionV' : WalletCreateTransaction
} |
    { 'PolicyCreateTransactionV' : PolicyCreateTransaction } |
    { 'MemberUpdateRoleTransactionV' : MemberUpdateRoleTransaction } |
    { 'VaultNamingUpdateTransactionV' : VaultNamingUpdateTransaction } |
    { 'PolicyRemoveTransactionV' : PolicyRemoveTransaction } |
    { 'PolicyUpdateTransactionV' : PolicyUpdateTransaction } |
    { 'TransferTransactionV' : TransferTransaction } |
    { 'MemberCreateTransactionV' : MemberCreateTransaction } |
    { 'MemberUpdateNameTransactionV' : MemberUpdateNameTransaction } |
    { 'QuorumUpdateTransactionV' : QuorumUpdateTransaction } |
    { 'WalletUpdateNameTransactionV' : WalletUpdateNameTransaction } |
    { 'MemberRemoveTransactionV' : MemberRemoveTransaction };
export type TransactionRequest = {
    'QuorumUpdateTransactionRequestV' : QuorumUpdateTransactionRequest
} |
    {
        'VaultNamingUpdateTransactionRequestV' : VaultNamingUpdateTransactionRequest
    } |
    {
        'MemberUpdateNameTransactionRequestV' : MemberUpdateNameTransactionRequest
    } |
    { 'WalletCreateTransactionRequestV' : WalletCreateTransactionRequest } |
    { 'MemberRemoveTransactionRequestV' : MemberRemoveTransactionRequest } |
    { 'MemberCreateTransactionRequestV' : MemberCreateTransactionRequest } |
    { 'TransferTransactionRequestV' : TransferTransactionRequest } |
    {
        'MemberUpdateRoleTransactionRequestV' : MemberUpdateRoleTransactionRequest
    } |
    {
        'WalletUpdateNameTransactionRequestV' : WalletUpdateNameTransactionRequest
    } |
    { 'PolicyUpdateTransactionRequestV' : PolicyUpdateTransactionRequest } |
    { 'PolicyRemoveTransactionRequestV' : PolicyRemoveTransactionRequest } |
    { 'PolicyCreateTransactionRequestV' : PolicyCreateTransactionRequest };
export type TransactionState = { 'Blocked' : null } |
    { 'Approved' : null } |
    { 'Rejected' : null } |
    { 'Executed' : null } |
    { 'Canceled' : null } |
    { 'Pending' : null };
export interface TransferTransaction {
    'block_index' : [] | [bigint],
    'currency' : Currency,
    'address' : string,
    'wallet' : string,
    'common' : BasicTransactionFields,
    'amount' : bigint,
    'policy' : [] | [string],
}
export interface TransferTransactionRequest {
    'currency' : Currency,
    'address' : string,
    'wallet' : string,
    'amount' : bigint,
}
export interface VaultNamingUpdateTransaction {
    'name' : [] | [string],
    'description' : [] | [string],
    'common' : BasicTransactionFields,
}
export interface VaultNamingUpdateTransactionRequest {
    'name' : [] | [string],
    'description' : [] | [string],
    'batch_uid' : [] | [string],
}
export type VaultRole = { 'Member' : null } |
    { 'Admin' : null };
export interface VaultState {
    'members' : Array<Member>,
    'name' : [] | [string],
    'description' : [] | [string],
    'wallets' : Array<Wallet>,
    'quorum' : Quorum,
    'policies' : Array<Policy>,
}
export interface Wallet {
    'uid' : string,
    'modified_date' : bigint,
    'name' : string,
    'network' : Network,
    'state' : ObjectState,
    'created_date' : bigint,
}
export interface WalletCreateTransaction {
    'uid' : string,
    'name' : string,
    'network' : Network,
    'common' : BasicTransactionFields,
}
export interface WalletCreateTransactionRequest {
    'name' : string,
    'network' : Network,
    'batch_uid' : [] | [string],
}
export interface WalletUpdateNameTransaction {
    'uid' : string,
    'name' : string,
    'common' : BasicTransactionFields,
}
export interface WalletUpdateNameTransactionRequest {
    'uid' : string,
    'name' : string,
    'batch_uid' : [] | [string],
}
export interface _SERVICE {
    'approve' : ActorMethod<
        [Array<TransactionApproveRequest>],
        Array<TransactionCandid>
    >,
    'execute' : ActorMethod<[], undefined>,
    'get_state' : ActorMethod<[[] | [bigint]], VaultState>,
    'get_transactions_all' : ActorMethod<[], Array<TransactionCandid>>,
    'request_transaction' : ActorMethod<
        [Array<TransactionRequest>],
        Array<TransactionCandid>
    >,
}
