import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Conf {
    'controllers' : Array<Principal>,
    'origins' : Array<string>,
}
export interface VaultWasm {
    'wasm_module' : Uint8Array | number[],
    'hash' : string,
    'description' : [] | [string],
    'version' : string,
}
export interface VersionWrapper {
    'description' : [] | [string],
    'version' : string,
}
export interface _SERVICE {
    'add_version' : ActorMethod<[VaultWasm], undefined>,
    'canister_balance' : ActorMethod<[], bigint>,
    'get_available_versions' : ActorMethod<[], Array<VersionWrapper>>,
    'get_by_version' : ActorMethod<[string], VaultWasm>,
    'get_latest_version' : ActorMethod<[], VaultWasm>,
    'sync_controllers' : ActorMethod<[], Array<string>>,
    'get_trusted_origins' : ActorMethod<[], Array<string>>,
}
