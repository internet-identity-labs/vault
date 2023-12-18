import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {createCanister, getCanisters} from "./sdk/ochestrator";
import {VaultManager} from "../vault/sdk_prototype/vault_manager";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {VaultCanister} from "./sdk/vm";


describe("VM Test", () => {
    let canister;
    let identity = getIdentity("87654321876543218765432187654321")
    let canister_id
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`dfx deploy vault_manager`))
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

    it("Get all canisters", async function () {
        let canisters: [VaultCanister] = await getCanisters(canister_id, identity);
        expect(canisters.length).eq(1);
        expect(canisters[0].canister_id.toText()).eq(canister.toText());
        expect(canisters[0].initiator.toText()).eq(identity.getPrincipal().toText());
    });
})


