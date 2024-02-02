import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Conf {
    'destination_address' : string,
    'initial_cycles_balance' : bigint,
    'origins' : Array<string>,
    'repo_canister_id' : string,
    'icp_price' : bigint,
}
export interface CreateResult { 'canister_id' : Principal }
export type Result = { 'Ok' : CreateResult } |
    { 'Err' : string };
export type Result_1 = { 'Ok' : null } |
    { 'Err' : string };
export interface VaultCanister {
    'initiator' : Principal,
    'canister_id' : Principal,
    'block_number' : bigint,
}
export interface _SERVICE {
    'canister_balance' : ActorMethod<[], bigint>,
    'create_canister_call' : ActorMethod<[bigint], Result>,
    'get_all_canisters' : ActorMethod<[], Array<VaultCanister>>,
    'get_config' : ActorMethod<[], Conf>,
    'update_canister_self' : ActorMethod<[string], Result_1>,
}
