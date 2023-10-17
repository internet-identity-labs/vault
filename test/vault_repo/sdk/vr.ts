import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface VaultWasm {
    'wasm_module' : Uint8Array | number[],
    'hash' : string,
    'version' : string,
}
export interface _SERVICE {
    'add_version' : ActorMethod<[VaultWasm], undefined>,
    'canister_balance' : ActorMethod<[], bigint>,
    'get_available_versions' : ActorMethod<[], Array<string>>,
    'get_by_version' : ActorMethod<[string], VaultWasm>,
}
