import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {VaultManager, VaultRole} from "./sdk_prototype/vault_manager";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {requestCreateMemberTransaction} from "./member_transactions.test";
import {expect} from "chai";
import {requestUpdateQuorumTransaction} from "./quorum_transactions.test";

require('./bigintExtension.js');

describe("State Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager();
        await manager.init(canister_id, admin_identity, true);
    });

    after(() => {
        DFX.STOP();
    });

    it("Request transactions - redefine state", async function () {
        let tr1 = await requestCreateMemberTransaction(manager, "memberAddress1", "memberName", VaultRole.MEMBER)
        let tr2 = await requestCreateMemberTransaction(manager, "memberAddress2", "memberName", VaultRole.MEMBER)
        let tr3 = await requestCreateMemberTransaction(manager, "memberAddress3", "memberName", VaultRole.MEMBER)
        let tr4 = await requestCreateMemberTransaction(manager, "memberAddress4", "memberName", VaultRole.MEMBER)
        let tr5 = await requestCreateMemberTransaction(manager, "memberAddress5", "memberName", VaultRole.MEMBER)
        await manager.execute()

        let currentState = await manager.redefineState();

        expect(currentState.members.length).eq(6)

        let state1trs = await manager.redefineState(1n)

        expect(state1trs.members.length).eq(1)
        expect(state1trs.members[0].userId).eq(principalToAddress(admin_identity.getPrincipal() as any))

        let state2trs = await manager.redefineState(tr3[0].id)

        expect(state2trs.members.length).eq(4)
    });

    it("Request transactions - redefine state after reject", async function () {
        let tr1 = await requestUpdateQuorumTransaction(manager, 3)
        let tr2 = await requestUpdateQuorumTransaction(manager, 5)
        let tr3 = await requestCreateMemberTransaction(manager, "memberAddress6", "memberName", VaultRole.MEMBER)
        let tr4 = await requestCreateMemberTransaction(manager, "memberAddress7", "memberName", VaultRole.MEMBER)
        await manager.execute()

        let state2trs = await manager.redefineState(tr2[0].id)

        expect(state2trs.quorum.quorum).eq(1)
        expect(state2trs.members.length).eq(6)
    });

    it("Request transactions - redefine state after blocked", async function () {
        let tr3 = await requestCreateMemberTransaction(manager, "memberAddress8", "memberName", VaultRole.MEMBER)
        let tr4 = await requestCreateMemberTransaction(manager, "memberAddress9", "memberName", VaultRole.MEMBER)

        let state2trs = await manager.redefineState(tr4[0].id)

        expect(state2trs.quorum.quorum).eq(1)
        //7 from previous + 1 admin and 2 in blocked (not executed)
        expect(state2trs.members.length).eq(8)
    });


})

