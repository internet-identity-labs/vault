import {getActor} from "../../util/deployment.util";
import {idlFactory} from "./vm_idl";
import {SignIdentity} from "@dfinity/agent";
import {Conf, Result, VaultCanister, VaultType} from "./vm";
import {Principal} from "@dfinity/principal";

export async function createCanister(canister_id: string, identity: SignIdentity, transactionBlock: BigInt, vaultType: Array<VaultType>, owner: Array<Principal> = []) {
    let actor = await getActor(canister_id, identity, idlFactory);
    let result = await actor.create_canister_call(transactionBlock, vaultType, owner) as Result;
    // @ts-ignore
    if (result.Err) {
        // @ts-ignore
        throw Error(result.Err)
    }
    // @ts-ignore
    return result.Ok.canister_id;
}

export async function getCanisters(canister_id: string, identity: SignIdentity): Promise<[VaultCanister]> {
    let actor = await getActor(canister_id, identity, idlFactory);
    return await actor.get_all_canisters() as [VaultCanister];
}

export async function getConfig(canister_id: string, identity: SignIdentity): Promise<Conf> {
    let actor = await getActor(canister_id, identity, idlFactory);
    return await actor.get_config() as Conf;
}