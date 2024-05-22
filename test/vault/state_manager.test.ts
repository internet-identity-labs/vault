import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {expect} from "chai";
import {requestCreateMemberTransaction, requestUpdateQuorumTransaction} from "./helper";
import {VaultManager, VaultRole} from "@nfid/vaults";
import { Principal } from "@dfinity/principal";

require('./bigintextension.js');

const vaultInitString = "vault --argument '(principal \"3ekng-5nqql-esu4u-64sla-pcm5o-hjatn-hwjo7-vk7ya-ianug-zqqyy-iae\", record { origins = vec {}; repo_canister = \"7jlkn-paaaa-aaaap-abvpa-cai\" })'"

describe("State Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager(canister_id, admin_identity);
        await manager.resetToLocalEnv();
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
        let tr4 = await requestCreateMemberTransaction(manager, "memberAddress7", "memberName", VaultRole.ADMIN)
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

    it( "ICRC1 Add" , async function () {
        let state = await manager.addICRC1Canister(Principal.fromText("6jq2j-daaaa-aaaap-absuq-cai"), Principal.fromText(canister_id))
        expect(state.icrc1_canisters.length).eq(1)
        expect(state.icrc1_canisters[0].ledger.toText()).eq("6jq2j-daaaa-aaaap-absuq-cai")
        expect(state.icrc1_canisters[0].index.toText()).eq(canister_id)
    });

    it( "ICRC1 Remove" , async function () {
        let icrc1 = Principal.fromText("6jq2j-daaaa-aaaap-absuq-cai")
        let state = await manager.removeICRC1Canister(icrc1)
        expect(state.icrc1_canisters.length).eq(0)
    });

})

