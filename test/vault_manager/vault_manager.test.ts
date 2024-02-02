import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {createCanister, getCanisters, getConfig} from "./sdk/ochestrator";
import {VaultManager} from "../vault/sdk_prototype/vault_manager";
import {expect} from "chai";
import {fromHexString, principalToAddress} from "ictool";
import {call, execute} from "../util/call.util";
import {VaultCanister} from "./sdk/vm";
import {idlFactory} from "../vault_repo/sdk/vr_idl";
import {sha256} from "ethers/lib/utils";
import {VaultWasm} from "../vault_repo/sdk/vr";
import {readWasmFile} from "../vault_repo/vault_repo.test";
import {fail} from "assert";


describe("VM Test", () => {
    let canister;
    let identity = getIdentity("87654321876543218765432187654321")
    let canister_id
    let vault_canister_id
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`dfx deploy vault_repo  --argument '(record { controllers = vec {}; origins = vec {}; })' --specified-id=7jlkn-paaaa-aaaap-abvpa-cai`))
        vault_canister_id = DFX.GET_CANISTER_ID("vault_repo");
        DFX.ADD_CONTROLLER(identity.getPrincipal().toText(), vault_canister_id);
        console.log(execute(`dfx canister call vault_repo sync_controllers`))
        console.log(execute(`./test/resource/ledger.sh`))

        let correctBytes = fromHexString("4918c656ea851d74504c84fe61581ef7cc00b282d44aa61b4c2c079ed189314e")
        console.log(DFX.LEDGER_FILL_BALANCE(correctBytes.toString().replaceAll(',', ';')))
        console.log(DFX.LEDGER_FILL_BALANCE(correctBytes.toString().replaceAll(',', ';')))

        let wasm_bytes = readWasmFile("test/vault_repo/vault_001.wasm");
        let actor = await getActor(vault_canister_id, identity, idlFactory);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            description: [],
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: "0.0.1"
        }
        await actor.add_version(wasm);

        console.log(execute(`./test/resource/vault_manager.sh`))
    });

    after(() => {
        DFX.STOP();
    });

    it('Get config test', async function () {
        canister_id = DFX.GET_CANISTER_ID("vault_manager");
        let config = await getConfig(canister_id, identity);
        expect(config.origins.length).eq(3);
        expect(config.initial_cycles_balance).eq(500000000000n);
        expect(config.destination_address).eq("4918c656ea851d74504c84fe61581ef7cc00b282d44aa61b4c2c079ed189314e");
        expect(config.icp_price).eq(100000000n);
    });


    it("Create Vault from the VaultManagerCanister", async function () {
        canister = await createCanister(canister_id, identity, BigInt(1));
        let vaultManager = new VaultManager()
        await vaultManager.init(canister, identity, true)
        let state = await vaultManager.getState()
        expect(state.members.length).eq(1);
        let address = principalToAddress(identity.getPrincipal() as any)
        expect(state.members[0].userId).eq(address);
        let canisters = await getCanisters(canister_id, identity);
        expect(canisters.length).eq(1);
        expect(canisters[0].canister_id.toText()).eq(canister.toText());
    });


    it("Create Vault from the VaultManagerCanister Rejected because of payment", async function () {
        let incorrectBytes = fromHexString("6eee6eb5aeb5b94688a1f1831b246560797db6b0c80d8a004f64a0498519d632")
        console.log(DFX.LEDGER_FILL_BALANCE(incorrectBytes.toString().replaceAll(',', ';')))
        try {
            await createCanister(canister_id, identity, BigInt(3));
            fail("Should throw error")
        } catch (e) {
            expect(e.message).contains("Incorrect destination");
        }
        let correctBytes = fromHexString("4918c656ea851d74504c84fe61581ef7cc00b282d44aa61b4c2c079ed189314e")
        console.log(call(`dfx canister call ledger transfer "(record { to=vec { ${correctBytes.toString().replaceAll(',', ';')} };
          amount=record { e8s=50_000_000 }; fee=record { e8s=10_000 : nat64 }; memo=0:nat64; } )"`))
        try {
            await createCanister(canister_id, identity, BigInt(4));
            fail("Should throw error")
        } catch (e) {
            expect(e.message).contains("Incorrect amount");
        }
        try {
            await createCanister(canister_id, identity, BigInt(1));
            fail("Should throw error")
        } catch (e) {
            expect(e.message).contains("Block already used");
        }
    });


    it("Create Vault from the VaultManagerCanister with newer version by default", async function () {
        let wasm_bytes = readWasmFile("test/vault_repo/vault_002.wasm");
        let actor = await getActor(vault_canister_id, identity, idlFactory);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            description: [],
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: "0.0.2"
        }
        await actor.add_version(wasm);

        canister = await createCanister(canister_id, identity, BigInt(2));

        let vaultManager = new VaultManager()
        await vaultManager.init(canister, identity, true)

        let canisters = await getCanisters(canister_id, identity);
        expect(canisters.length).eq(2);
    });


    it("Get all canisters", async function () {
        let canisters: [VaultCanister] = await getCanisters(canister_id, identity);
        canisters.sort();
        expect(canisters.length).eq(2);
        expect(canisters.map(l => l.canister_id.toText()).find(l => l === canister.toText())).not.eq(undefined);
        expect(canisters[0].initiator.toText()).eq(identity.getPrincipal().toText());
    });


    it("Get all canisters after upgrade", async function () {
        DFX.UPGRADE_FORCE("vault_manager")
        let canisters: [VaultCanister] = await getCanisters(canister_id, identity);
        expect(canisters.length).eq(2);
        let actor = await getActor(canister_id, identity, idlFactory);
        let origins = await actor.get_trusted_origins() as Array<String>
        expect(origins.length).eq(3);
    });

})


