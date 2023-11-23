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
}
export interface MemberRemoveTransaction {
    'member_id' : string,
    'common' : BasicTransactionFields,
}
export interface MemberRemoveTransactionRequest { 'member_id' : string }
export interface MemberUpdateNameTransaction {
    'name' : string,
    'member_id' : string,
    'common' : BasicTransactionFields,
}
export interface MemberUpdateNameTransactionRequest {
    'name' : string,
    'member_id' : string,
}
export interface MemberUpdateRoleTransaction {
    'role' : VaultRole,
    'member_id' : string,
    'common' : BasicTransactionFields,
}
export interface MemberUpdateRoleTransactionRequest {
    'role' : VaultRole,
    'member_id' : string,
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
export interface PolicyRemoveTransaction {
    'uid' : string,
    'common' : BasicTransactionFields,
}
export interface PolicyUpdateTransaction {
    'uid' : string,
    'member_threshold' : number,
    'amount_threshold' : bigint,
    'common' : BasicTransactionFields,
}
export interface Quorum { 'modified_date' : bigint, 'quorum' : number }
export interface QuorumUpdateTransaction {
    'transaction_type' : TrType,
    'common' : BasicTransactionFields,
    'quorum' : number,
}
export interface QuorumUpdateTransactionRequest { 'quorum' : number }
export type TrType = { 'WalletUpdateName' : null } |
    { 'MemberCreate' : null } |
    { 'PolicyRemove' : null } |
    { 'WalletCreate' : null } |
    { 'MemberRemove' : null } |
    { 'PolicyCreate' : null } |
    { 'PolicyUpdate' : null } |
    { 'MemberUpdateName' : null } |
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
    { 'PolicyRemoveTransactionV' : PolicyRemoveTransaction } |
    { 'WalletUpdateTransactionV' : WalletUpdateNameTransaction } |
    { 'PolicyUpdateTransactionV' : PolicyUpdateTransaction } |
    { 'MemberCreateTransactionV' : MemberCreateTransaction } |
    { 'MemberUpdateNameTransactionV' : MemberUpdateNameTransaction } |
    { 'QuorumUpdateTransactionV' : QuorumUpdateTransaction } |
    { 'MemberRemoveTransactionV' : MemberRemoveTransaction };
export type TransactionRequest = {
    'QuorumUpdateTransactionRequestV' : QuorumUpdateTransactionRequest
} |
    {
        'MemberUpdateNameTransactionRequestV' : MemberUpdateNameTransactionRequest
    } |
    { 'WalletCreateTransactionRequestV' : WalletCreateTransactionRequest } |
    { 'MemberRemoveTransactionRequestV' : MemberRemoveTransactionRequest } |
    { 'MemberCreateTransactionRequestV' : MemberCreateTransactionRequest } |
    {
        'MemberUpdateRoleTransactionRequestV' : MemberUpdateRoleTransactionRequest
    } |
    {
        'WalletUpdateNameTransactionRequestV' : WalletUpdateNameTransactionRequest
    };
export type TransactionState = { 'Blocked' : null } |
    { 'Approved' : null } |
    { 'Rejected' : null } |
    { 'Executed' : null } |
    { 'Canceled' : null } |
    { 'Pending' : null };
export type VaultRole = { 'Member' : null } |
    { 'Admin' : null };
export interface VaultState {
    'members' : Array<Member>,
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
    'uid' : string,
    'name' : string,
    'network' : Network,
}
export interface WalletUpdateNameTransaction {
    'uid' : string,
    'name' : string,
    'common' : BasicTransactionFields,
}
export interface WalletUpdateNameTransactionRequest {
    'uid' : string,
    'name' : string,
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
