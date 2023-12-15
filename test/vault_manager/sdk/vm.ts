import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface CreateResult { 'canister_id' : Principal }
export type Result = { 'Ok' : CreateResult } |
    { 'Err' : string };
export interface _SERVICE {
    'create_canister_call' : ActorMethod<[], Result>,
}
