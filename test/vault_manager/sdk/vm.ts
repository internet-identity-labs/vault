import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface CreateResult { 'canister_id' : Principal }
export type Result = { 'Ok' : CreateResult } |
    { 'Err' : string };
export interface VaultCanister {
    'initiator' : Principal,
    'canister_id' : Principal,
}
export interface _SERVICE {
    'canister_balance' : ActorMethod<[], bigint>,
    'create_canister_call' : ActorMethod<[bigint], Result>,
    'get_all_canisters' : ActorMethod<[], Array<VaultCanister>>,
    'get_trusted_origins' : ActorMethod<[], Array<string>>,
}
