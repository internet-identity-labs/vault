import {Actor, ActorMethod, HttpAgent, Identity} from "@dfinity/agent";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {Dfx} from "../type/dfx";
import {idlFactory as vaultIdl} from "../idl/vault_idl";
import {TextEncoder} from "util";
import {App} from "../constanst/app.enum";
import {IDL} from "@dfinity/candid";
import {DFX} from "../constanst/dfx.const";
import {execute} from "./call.util";

const localhost: string = "http://127.0.0.1:8000";

export const deploy = async ({clean = true, apps}: { clean?: boolean, apps: App[] }): Promise<Dfx> => {
    var i = 0;
    var dfx: Dfx = {
        root: null,
        user: {
            principal: null,
            identity: null
        },
        vault: {
            id: null,
            admin_actor: null,
            actor_member_1: null,
            actor_member_2: null,
            member_1: null,
            member_2: null
        }
    };

    while (++i <= 5) {
        dfx.user.identity = getIdentity("87654321876543218765432187654321");
        dfx.user.principal = dfx.user.identity.getPrincipal().toString();

        if (clean) {
            DFX.STOP();
            DFX.REMOVE_DFX_FOLDER();
            DFX.CREATE_TEST_PERSON();
            DFX.USE_TEST_ADMIN();
        }

        dfx.root = DFX.GET_PRINCIPAL();

        if (clean) {
            DFX.INIT();
        }

        if (apps.includes(App.Vault)) {
            DFX.USE_TEST_ADMIN();
            await console.log(execute(`./test/resource/ledger.sh`))
            await console.log(execute(`./test/resource/vault.sh`))

            dfx.vault.id = DFX.GET_CANISTER_ID("vault");
            console.log(">> ", dfx.vault.id);

            dfx.vault.admin_actor = await getActor(dfx.vault.id, dfx.user.identity, vaultIdl);
            dfx.vault.member_1 = Ed25519KeyIdentity.generate();
            dfx.vault.member_2 = Ed25519KeyIdentity.generate();
            dfx.vault.actor_member_1 = await getActor(dfx.vault.id, dfx.vault.member_1, vaultIdl);
            dfx.vault.actor_member_2 = await getActor(dfx.vault.id, dfx.vault.member_2, vaultIdl);
            return dfx;
        }
        throw Error("Empty App")
    }

    DFX.STOP();
    process.exit(1);
};

export const getIdentity = (seed: string): Ed25519KeyIdentity => {
    let seedEncoded = new TextEncoder().encode(seed);
    return Ed25519KeyIdentity.generate(seedEncoded);
};

export const getActor = async (
    imCanisterId: string,
    identity: Identity,
    idl: IDL.InterfaceFactory
): Promise<Record<string, ActorMethod>> => {
    const agent: HttpAgent = new HttpAgent({ host: localhost, identity: identity });
    await agent.fetchRootKey();
    return Actor.createActor(idl, { agent, canisterId: imCanisterId });
};
