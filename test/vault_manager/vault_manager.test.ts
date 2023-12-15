import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {createCanister} from "./sdk/ochestrator";
import {VaultManager} from "../vault/sdk_prototype/vault_manager";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";


describe("VM Test", () => {

    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`dfx deploy vault_manager`))
    });

    after(() => {
        DFX.STOP();
    });

    it("Create Vault from the VaultManagerCanister", async function () {
        let canister_id = DFX.GET_CANISTER_ID("vault_manager");
        let identity = getIdentity("87654321876543218765432187654321")
        let canister = await createCanister(canister_id, identity, BigInt(0));
        let vaultManager = new VaultManager()
        await vaultManager.init(canister, identity, true)
        let state = await vaultManager.redefineState()
        expect(state.members.length).eq(1);
        let address = principalToAddress(identity.getPrincipal() as any)
        expect(state.members[0].userId).eq(address);
    });


})


