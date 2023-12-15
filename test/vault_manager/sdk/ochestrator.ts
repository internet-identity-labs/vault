import {getActor} from "../../util/deployment.util";
import {idlFactory} from "./vm_idl";
import {SignIdentity} from "@dfinity/agent";
import {Result, VaultCanister} from "./vm";

export async function createCanister(canister_id: string, identity: SignIdentity, transactionBlock: BigInt) {
    let actor = await getActor(canister_id, identity, idlFactory);
    let result = await actor.create_canister_call() as Result;
    // @ts-ignore
    if (result.Err) {
        // @ts-ignore
        throw Error(result.Err)
    }
    // @ts-ignore
    return result.Ok.canister_id;
}
export async function getCanisters(canister_id: string, identity: SignIdentity) {
    let actor = await getActor(canister_id, identity, idlFactory);
    return await actor.get_all_canisters() as [VaultCanister];
}