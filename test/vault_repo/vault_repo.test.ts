import {getActor, getIdentity} from "../util/deployment.util";
import {DFX} from "../constanst/dfx.const";
import {execute} from "../util/call.util";
import * as fs from "fs";
import {idlFactory} from "./sdk/vr_idl";
import {VaultWasm} from "./sdk/vr";
import {sha256} from "ethers/lib/utils";
import {expect} from "chai";
import {fail} from "assert";


describe("VR Test", () => {
    let identity = getIdentity("87654321876543218765432187654321")
    let canister_id
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`dfx deploy vault_repo  --argument '(record { controllers = vec {}; origins = vec {}; })' `))
        canister_id = DFX.GET_CANISTER_ID("vault_repo");
        DFX.ADD_CONTROLLER(identity.getPrincipal().toText(), canister_id);
        await console.log(execute(`dfx canister call vault_repo sync_controllers`))
    });

    after(() => {
        DFX.STOP();
    });

    it("Set 0.0.1 Vault", async function () {
        let wasm_bytes = readWasmFile("test/vault_repo/vault_001.wasm");
        let actor =  await getActor(canister_id, identity, idlFactory);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: "0.0.1"
        }
        await actor.add_version(wasm);
        let wasm2 = await actor.get_by_version("0.0.1") as VaultWasm;
        expect(wasm2.hash).eq(hash);
        expect(wasm2.version).eq("0.0.1");
    });

    it("Get versions", async function () {
        let wasm_bytes = readWasmFile("test/vault_repo/vault_002.wasm");
        let actor =  await getActor(canister_id, identity, idlFactory);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: "0.0.2"
        }
        await actor.add_version(wasm);
        let wasm2 = await actor.get_by_version("0.0.2") as VaultWasm;
        expect(wasm2.version).eq("0.0.2");
        let versions = await actor.get_available_versions() as string[];
        expect(versions.length).eq(2);
        versions.sort()
        expect(versions[0]).eq("0.0.1");
        expect(versions[1]).eq("0.0.2");
    });


    it("Check security", async function () {
        let not_admin = getIdentity("87654321876543218765432187654322")

        let wasm_bytes = readWasmFile("test/vault_repo/vault_002.wasm");
        let actor =  await getActor(canister_id, not_admin, idlFactory);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: "0.0.3"
        }
        try {
            await actor.add_version(wasm);
            fail("Should not be able to add version")
        }catch (e) {
            expect(e.message).contains("Unauthorised")
        }

    });
})

export function readWasmFile(filePath: string): Uint8Array {
    try {
        const buffer = fs.readFileSync(filePath);
        return new Uint8Array(buffer);
    } catch (error) {
        throw new Error(`Error reading Wasm file: ${error}`);
    }
}