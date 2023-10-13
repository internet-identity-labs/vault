import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Approve {
    'status' : TransactionState,
    'signer' : string,
    'created_date' : bigint,
}
export type Backup = { 'Users' : null } |
    { 'Wallets' : null } |
    { 'Vaults' : null } |
    { 'Transactions' : null } |
    { 'Policies' : null };
export interface Conf {
    'controllers' : [] | [Array<Principal>],
    'origins' : [] | [Array<string>],
    'ledger_canister_id' : Principal,
    'is_test_env' : [boolean]
}
export type Currency = { 'ICP' : null };
export type ObjectState = { 'Active' : null } |
    { 'Archived' : null };
export interface Policy {
    'id' : bigint,
    'vault' : bigint,
    'modified_date' : bigint,
    'state' : ObjectState,
    'policy_type' : PolicyType,
    'created_date' : bigint,
}
export interface PolicyRegisterRequest {
    'vault_id' : bigint,
    'policy_type' : PolicyType,
}
export type PolicyType = { 'threshold_policy' : ThresholdPolicy };
export interface ThresholdPolicy {
    'member_threshold' : [] | [number],
    'amount_threshold' : bigint,
    'wallets' : [] | [Array<string>],
    'currency' : Currency,
}
export interface Transaction {
    'id' : bigint,
    'to' : string,
    'member_threshold' : number,
    'block_index' : [] | [bigint],
    'owner' : string,
    'from' : string,
    'modified_date' : bigint,
    'memo' : [] | [string],
    'vault_id' : bigint,
    'amount_threshold' : bigint,
    'state' : TransactionState,
    'approves' : Array<Approve>,
    'currency' : Currency,
    'amount' : bigint,
    'created_date' : bigint,
    'policy_id' : bigint,
}
export interface TransactionApproveRequest {
    'transaction_id' : bigint,
    'state' : TransactionState,
}
export interface TransactionRegisterRequest {
    'address' : string,
    'amount' : bigint,
    'wallet_id' : string,
}
export type TransactionState = { 'Approved' : null } |
    { 'Rejected' : null } |
    { 'Canceled' : null } |
    { 'Pending' : null };
export interface Vault {
    'id' : bigint,
    'members' : Array<VaultMember>,
    'modified_date' : bigint,
    'name' : string,
    'description' : [] | [string],
    'state' : ObjectState,
    'wallets' : Array<string>,
    'created_date' : bigint,
    'policies' : [bigint],
}
export interface VaultMember {
    'user_uuid' : string,
    'name' : [] | [string],
    'role' : VaultRole,
    'state' : ObjectState,
    'migrated' : [] | [boolean]
}
export interface VaultMemberRequest {
    'name' : [] | [string],
    'role' : VaultRole,
    'vault_id' : bigint,
    'state' : ObjectState,
    'address' : string,
}
export interface VaultRegisterRequest {
    'name' : string,
    'description' : [] | [string],
}
export type VaultRole = { 'Member' : null } |
    { 'Admin' : null };
export interface Wallet {
    'uid' : string,
    'modified_date' : bigint,
    'name' : [] | [string],
    'vaults' : [bigint],
    'state' : ObjectState,
    'created_date' : bigint,
}
export interface WalletRegisterRequest {
    'name' : [] | [string],
    'vault_id' : bigint,
}
export interface _SERVICE {
    'approve_transaction' : ActorMethod<[TransactionApproveRequest], Transaction>,
    'config' : ActorMethod<[Conf], undefined>,
    'count' : ActorMethod<[Backup], bigint>,
    'get_all_json' : ActorMethod<[number, number, Backup], string>,
    'get_config' : ActorMethod<[], Conf>,
    'get_policies' : ActorMethod<[bigint], Array<Policy>>,
    'get_transactions' : ActorMethod<[], Array<Transaction>>,
    'get_trusted_origins' : ActorMethod<[], Array<string>>,
    'get_vaults' : ActorMethod<[], Array<Vault>>,
    'get_wallets' : ActorMethod<[bigint], Array<Wallet>>,
    'migrate_user' : ActorMethod<[string], boolean>,
    'register_policy' : ActorMethod<[PolicyRegisterRequest], Policy>,
    'register_transaction' : ActorMethod<
        [TransactionRegisterRequest],
        Transaction
    >,
    'register_vault' : ActorMethod<[VaultRegisterRequest], Vault>,
    'register_wallet' : ActorMethod<[WalletRegisterRequest], Wallet>,
    'store_member' : ActorMethod<[VaultMemberRequest], Vault>,
    'sync_controllers' : ActorMethod<[], Array<string>>,
    'update_policy' : ActorMethod<[Policy], Policy>,
    'update_vault' : ActorMethod<[Vault], Vault>,
    'update_wallet' : ActorMethod<[Wallet], Wallet>,
}
