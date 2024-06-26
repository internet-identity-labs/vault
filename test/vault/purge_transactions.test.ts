import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {execute, sleep} from "../util/call.util";
import {
    getTransactionByIdFromGetAllTrs,
    requestCreateMemberTransaction,
    requestPurgeTransaction,
    requestUpdateQuorumTransaction
} from "./helper";
import {ApproveRequest, TransactionState, VaultManager, VaultRole} from "@nfid/vaults";
import {PurgeTransaction} from "@nfid/vaults";
import {MemberCreateTransaction} from "@nfid/vaults";

require('./bigintextension.js');

describe("Purge Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    const member = getIdentity("87654321876543218765432187654320");
    let manager: VaultManager;
    let manager2: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager(canister_id, admin_identity);
        manager2 = new VaultManager(canister_id, member);
        await manager.resetToLocalEnv();
        await manager2.resetToLocalEnv();
    });

    after(() => {
        DFX.STOP();
    });

    it("Purge transactions", async function () {
        await requestCreateMemberTransaction(manager, principalToAddress(member.getPrincipal() as any), "memberName", VaultRole.ADMIN)
        await manager.execute()
        await requestUpdateQuorumTransaction(manager, 2)
        await sleep(2);
        let tr2 = await requestCreateMemberTransaction(manager, "memberAddress3", "memberName", VaultRole.ADMIN)
        let tr3 = await requestCreateMemberTransaction(manager, "memberAddress4", "memberName", VaultRole.ADMIN)
        await sleep(2);
        let tr = await requestPurgeTransaction(manager);
        let tr4 = await requestCreateMemberTransaction(manager, "memberAddress5", "memberName", VaultRole.ADMIN)
        let tr5 = await requestCreateMemberTransaction(manager, "memberAddress6", "memberName", VaultRole.ADMIN)
        await sleep(2);
        let approveRequest: ApproveRequest = {
            trId: tr[0].id,
            state: TransactionState.Approved
        }
        await manager2.approveTransaction([approveRequest])
        await manager2.execute();
        let purgeTr = await getTransactionByIdFromGetAllTrs(manager, tr[0].id) as PurgeTransaction
        expect(purgeTr.state).eq(TransactionState.Executed)
        let purgedMemberTransaction1 = await getTransactionByIdFromGetAllTrs(manager, tr2[0].id) as MemberCreateTransaction
        let purgedMemberTransaction2 = await getTransactionByIdFromGetAllTrs(manager, tr3[0].id) as MemberCreateTransaction
        let purgedMemberTransaction3 = await getTransactionByIdFromGetAllTrs(manager, tr4[0].id) as MemberCreateTransaction
        let purgedMemberTransaction4 = await getTransactionByIdFromGetAllTrs(manager, tr5[0].id) as MemberCreateTransaction
        expect(purgedMemberTransaction1.state).eq(TransactionState.Purged)
        expect(purgedMemberTransaction2.state).eq(TransactionState.Purged)
        expect(purgedMemberTransaction3.state).eq(TransactionState.Pending)
        expect(purgedMemberTransaction4.state).eq(TransactionState.Blocked)
        let state = await manager.getState();
        expect(state.members.length).eq(2)
    });

})

