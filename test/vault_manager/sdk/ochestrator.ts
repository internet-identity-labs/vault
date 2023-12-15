import {getActor} from "../../util/deployment.util";
import {idlFactory} from "./vm_idl";
import {SignIdentity} from "@dfinity/agent";
import {Result} from "./vm";

export async function createCanister(canister_id: string, identity: SignIdentity, transactionBlock: BigInt) {
    let actor = await getActor(canister_id, identity, idlFactory);
    let result = await actor.create_canister_call() as Result;
    console.log(123123)
    // @ts-ignore
    if (result.Err) {
        // @ts-ignore
        throw Error(result.Err)
    }
    // @ts-ignore
    return result.Ok.canister_id;
}