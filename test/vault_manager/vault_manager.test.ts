import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {createCanister, getCanisters} from "./sdk/ochestrator";
import {VaultManager} from "../vault/sdk_prototype/vault_manager";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {VaultCanister} from "./sdk/vm";
import {idlFactory} from "../vault_repo/sdk/vr_idl";
import {sha256} from "ethers/lib/utils";
import {VaultWasm} from "../vault_repo/sdk/vr";
import {readWasmFile} from "../vault_repo/vault_repo.test";


describe("VM Test", () => {
    let canister;
    let identity = getIdentity("87654321876543218765432187654321")
    let canister_id
    let vault_canister_id
    before(async () => {
        // DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`dfx deploy vault_repo --specified-id=6jq2j-daaaa-aaaap-absuq-cai`))
        vault_canister_id = DFX.GET_CANISTER_ID("vault_repo");
        DFX.ADD_CONTROLLER(identity.getPrincipal().toText(), vault_canister_id);
        await console.log(execute(`dfx canister call vault_repo sync_controllers`))

        let wasm_bytes = readWasmFile("test/vault_repo/vault_001.wasm");
        let actor =  await getActor(vault_canister_id, identity, idlFactory);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: "0.0.1"
        }
        await actor.add_version(wasm);

        await console.log(execute(`dfx deploy vault_manager --specified-id=sgk26-7yaaa-aaaan-qaovq-cai`))

    });

    after(() => {
        DFX.STOP();
    });

    it("Create Vault from the VaultManagerCanister", async function () {
        canister_id = DFX.GET_CANISTER_ID("vault_manager");
        canister = await createCanister(canister_id, identity, BigInt(0));
        let vaultManager = new VaultManager()
        await vaultManager.init(canister, identity, true)
        let state = await vaultManager.redefineState()
        expect(state.members.length).eq(1);
        let address = principalToAddress(identity.getPrincipal() as any)
        expect(state.members[0].userId).eq(address);
        let canisters = await getCanisters(canister_id, identity);
        expect(canisters.length).eq(1);
        expect(canisters[0].canister_id.toText()).eq(canister.toText());
    });


    it("Create Vault from the VaultManagerCanister with newer version by default", async function () {
        let wasm_bytes = readWasmFile("test/vault_repo/vault_002.wasm");
        let actor =  await getActor(vault_canister_id, identity, idlFactory);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: "0.0.2"
        }
        await actor.add_version(wasm);

        canister = await createCanister(canister_id, identity, BigInt(0));

        let vaultManager = new VaultManager()
        await vaultManager.init(canister, identity, true)

        let canisters = await getCanisters(canister_id, identity);
        expect(canisters.length).eq(2);
    });



    it("Get all canisters", async function () {
        let canisters: [VaultCanister] = await getCanisters(canister_id, identity);
        canisters.sort();
        expect(canisters.length).eq(2);
        expect(canisters[0].canister_id.toText()).eq(canister.toText());
        expect(canisters[0].initiator.toText()).eq(identity.getPrincipal().toText());
    });
})


