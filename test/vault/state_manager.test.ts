import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {principalToAddress} from "ictool";
import {execute, sleep} from "../util/call.util";
import {expect} from "chai";
import {requestCreateMemberTransaction, requestUpdateQuorumTransaction} from "./helper";
import {
    ICRC1CanistersAddTransactionRequest,
    ICRC1CanistersRemoveTransactionRequest,
    VaultManager,
    VaultRole
} from "@nfid/vaults";
import {Principal} from "@dfinity/principal";

require('./bigintextension.js');

describe("State Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let admin_identity2 = getIdentity("87654321876543218765432187654322")
    let manager: VaultManager;
    let manager2: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager(canister_id, admin_identity);
        await manager.resetToLocalEnv();
        manager2 = new VaultManager(canister_id, admin_identity2);
        await manager2.resetToLocalEnv();
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

        let currentState = await manager.getState();

        expect(currentState.members.length).eq(6)

        let state1trs = await manager.getState(1n)

        expect(state1trs.members.length).eq(1)
        expect(state1trs.members[0].userId).eq(principalToAddress(admin_identity.getPrincipal() as any))

        let state2trs = await manager.getState(tr3[0].id)

        expect(state2trs.members.length).eq(4)
    });

    it("Request transactions - redefine state after reject", async function () {
        let tr1 = await requestUpdateQuorumTransaction(manager, 3)
        let tr2 = await requestUpdateQuorumTransaction(manager, 5)
        let tr3 = await requestCreateMemberTransaction(manager, "memberAddress6", "memberName", VaultRole.MEMBER)
        let tr4 = await requestCreateMemberTransaction(manager, principalToAddress(admin_identity2.getPrincipal() as any), "memberName", VaultRole.ADMIN)
        await manager.execute()

        let state2trs = await manager.getState(tr2[0].id)

        expect(state2trs.quorum.quorum).eq(1)
        expect(state2trs.members.length).eq(6)
    });

    it("Request transactions - redefine state after blocked", async function () {
        let tr1 = await requestUpdateQuorumTransaction(manager, 2)
        await manager.execute()
        let tr3 = await requestCreateMemberTransaction(manager, "memberAddress8", "memberName", VaultRole.ADMIN)
        let tr4 = await requestCreateMemberTransaction(manager, "memberAddress9", "memberName", VaultRole.MEMBER)
        let state2trs = await manager.getState(tr4[0].id)

        expect(state2trs.quorum.quorum).eq(2)
        //7 from previous + 1 admin and 2 in blocked (not executed)
        expect(state2trs.members.length).eq(8)
    });

    it("ICRC1 Add", async function () {
        let addICRC1Transaction = new ICRC1CanistersAddTransactionRequest(Principal.fromText("6jq2j-daaaa-aaaap-absuq-cai"), Principal.fromText(canister_id))
        await manager.requestTransaction([addICRC1Transaction])
        await sleep(2)
        await manager.execute()
        let state = await manager.getState()
        expect(state.icrc1_canisters.length).eq(1)
        expect(state.icrc1_canisters[0].ledger.toText()).eq("6jq2j-daaaa-aaaap-absuq-cai")
        expect(state.icrc1_canisters[0].index.toText()).eq(canister_id)
    });

    it("ICRC1 Remove", async function () {
        let removceICRC1 = new ICRC1CanistersRemoveTransactionRequest(Principal.fromText("6jq2j-daaaa-aaaap-absuq-cai"))
        await manager.requestTransaction([removceICRC1])
        await manager.execute()
        let state = await manager.getState()
        expect(state.icrc1_canisters.length).eq(0)
    });

})

