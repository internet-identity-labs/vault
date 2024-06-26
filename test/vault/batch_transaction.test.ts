import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {execute, sleep} from "../util/call.util";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {getTransactionByIdFromGetAllTrs} from "./helper";
import {
    ApproveRequest,
    MemberCreateTransactionRequest, MemberRemoveTransactionRequest,
    QuorumTransactionRequest,
    TransactionState,
    VaultManager,
    VaultRole
} from "@nfid/vaults";
import {fail} from "assert";

require('./bigintextension.js');

describe("Batch Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let admin_identity2 = getIdentity("87654321876543218765432187654322")
    let admin_identity3 = getIdentity("87654321876543218765432187654323")
    let admin_identity4 = getIdentity("87654321876543218765432187654324")
    let manager: VaultManager;
    let manager2: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager(canister_id, admin_identity);
        manager2 = new VaultManager(canister_id, admin_identity2);
        await manager.resetToLocalEnv();
        await manager2.resetToLocalEnv();
    });

    after(() => {
        DFX.STOP();
    });

    let trs;

    it("Request batch transaction incorrect order - rejected", async function () {
        let memberCreate = new MemberCreateTransactionRequest("memberAddress", "memberName", VaultRole.ADMIN);
        let quorumTransactionRequest = new QuorumTransactionRequest(2);
        let batchUid = "someRandomGeneratedUID_1"
        memberCreate.batch_uid = batchUid
        quorumTransactionRequest.batch_uid = batchUid
        let res = await manager.requestTransaction([quorumTransactionRequest, memberCreate])
        await sleep(2)
        let state = await manager.getState();
        let tr1 = await getTransactionByIdFromGetAllTrs(manager, res[0].id)
        let tr2 = await getTransactionByIdFromGetAllTrs(manager, res[1].id)
        expect(state.members.length).eq(1)
        expect(state.quorum.quorum).eq(1)
        expect(tr1.state).eq(TransactionState.Failed)
        expect(tr2.state).eq(TransactionState.Failed)
        let allTransactions = await manager.getTransactions()
        trs = allTransactions.length
    });

    it("Request batch transaction with different batch UIDs - rejected", async function () {
        let memberCreate = new MemberCreateTransactionRequest("memberAddress", "memberName", VaultRole.ADMIN);
        let quorumTransactionRequest = new QuorumTransactionRequest(2);
        let batchUid = "someRandomGeneratedUID_1"
        memberCreate.batch_uid = batchUid
        try {
            await manager.requestTransaction([memberCreate, quorumTransactionRequest])
            fail("Should throw error")
        } catch (e) {
            expect(e.message).contains("All objects should have the same batch UID, or do not have it at all")
        }
        let state = await manager.getState();
        expect(state.quorum.quorum).eq(1)
        expect(state.members.length).eq(1)
        quorumTransactionRequest.batch_uid = "someRandomGeneratedUID_2"
        try {
            await manager.requestTransaction([memberCreate, quorumTransactionRequest])
            fail("Should throw error")
        } catch (e) {
            expect(e.message).contains("All objects should have the same batch UID, or do not have it at all")
        }
        state = await manager.getState();
        expect(state.quorum.quorum).eq(1)
        expect(state.members.length).eq(1)
        let allTransactions = await manager.getTransactions()
        expect(allTransactions.length).eq(trs)
    });


    it("Request batch transaction - with member recreate", async function () {
        let memberCreate = new MemberCreateTransactionRequest(principalToAddress(admin_identity4.getPrincipal() as any), "memberName", VaultRole.ADMIN);
        let quorumTransactionRequest = new QuorumTransactionRequest(0);
        let batchUid = "someRandomGeneratedUID_3"
        memberCreate.batch_uid = batchUid
        quorumTransactionRequest.batch_uid = batchUid
        let res = await manager.requestTransaction([memberCreate, quorumTransactionRequest])
        await sleep(2)
        let tr1 = await getTransactionByIdFromGetAllTrs(manager, res[0].id)
        let tr2 = await getTransactionByIdFromGetAllTrs(manager, res[1].id)

        let state = await manager.getState();
        expect(state.members.length).eq(1)
        expect(state.quorum.quorum).eq(1)
        expect(tr1.state).eq(TransactionState.Failed)
        expect(tr2.state).eq(TransactionState.Failed)
        let memberReCreate = new MemberCreateTransactionRequest(principalToAddress(admin_identity4.getPrincipal() as any), "memberName", VaultRole.ADMIN);
        res = await manager.requestTransaction([memberReCreate])
        await sleep(2)
        tr1 = await getTransactionByIdFromGetAllTrs(manager, res[0].id)
        state = await manager.getState();
        expect(state.members.length).eq(2)
        expect(tr1.state).eq(TransactionState.Executed)
        let memberRemove = new MemberRemoveTransactionRequest(principalToAddress(admin_identity4.getPrincipal() as any));
        await manager.requestTransaction([memberRemove])
        await sleep(2)
        state = await manager.getState();
        expect(state.members.length).eq(1)
    });

    it("Request batch transaction - rejected", async function () {
        let memberCreate = new MemberCreateTransactionRequest("memberAddress", "memberName", VaultRole.ADMIN);
        let quorumTransactionRequest = new QuorumTransactionRequest(3);
        let batchUid = "someRandomGeneratedUID_12"
        memberCreate.batch_uid = batchUid
        quorumTransactionRequest.batch_uid = batchUid
        let res = await manager.requestTransaction([memberCreate, quorumTransactionRequest])
        await sleep(2)
        let state = await manager.getState();
        let tr1 = await getTransactionByIdFromGetAllTrs(manager, res[0].id)
        let tr2 = await getTransactionByIdFromGetAllTrs(manager, res[1].id)
        expect(state.members.length).eq(1)
        expect(state.quorum.quorum).eq(1)
        expect(tr1.state).eq(TransactionState.Failed)
        expect(tr2.state).eq(TransactionState.Failed)
    });

    it("Request batch transaction - executed", async function () {
        let memberCreate = new MemberCreateTransactionRequest(principalToAddress(admin_identity2.getPrincipal() as any), "memberName", VaultRole.ADMIN);
        let quorumTransactionRequest = new QuorumTransactionRequest(2);
        let batchUid = "someRandomGeneratedUID_2"
        memberCreate.batch_uid = batchUid
        quorumTransactionRequest.batch_uid = batchUid
        let res = await manager.requestTransaction([memberCreate, quorumTransactionRequest])
        await sleep(2)
        let state = await manager.getState();
        let tr1 = await getTransactionByIdFromGetAllTrs(manager, res[0].id)
        let tr2 = await getTransactionByIdFromGetAllTrs(manager, res[1].id)
        expect(state.members.length).eq(2)
        expect(state.quorum.quorum).eq(2)
        expect(tr1.state).eq(TransactionState.Executed)
        expect(tr2.state).eq(TransactionState.Executed)
    });


    it("Request batch transaction - with approves executed", async function () {
        let memberCreate = new MemberCreateTransactionRequest(principalToAddress(admin_identity3.getPrincipal() as any), "memberName", VaultRole.ADMIN);
        let quorumTransactionRequest = new QuorumTransactionRequest(3);
        let batchUid = "someRandomGeneratedUID_3"
        memberCreate.batch_uid = batchUid
        quorumTransactionRequest.batch_uid = batchUid
        let res = await manager.requestTransaction([memberCreate, quorumTransactionRequest])
        await sleep(2)
        let tr1 = await getTransactionByIdFromGetAllTrs(manager, res[0].id)
        let tr2 = await getTransactionByIdFromGetAllTrs(manager, res[1].id)
        expect(tr1.state).eq(TransactionState.Pending)
        expect(tr2.state).eq(TransactionState.Blocked)
        let approve1: ApproveRequest = {
            trId: tr1.id,
            state: TransactionState.Approved
        }
        let approve2: ApproveRequest = {
            trId: tr2.id,
            state: TransactionState.Approved
        }
        await manager2.approveTransaction([approve1, approve2])
        await sleep(2)
        tr1 = await getTransactionByIdFromGetAllTrs(manager, res[0].id)
        tr2 = await getTransactionByIdFromGetAllTrs(manager, res[1].id)

        let state = await manager.getState();
        expect(state.members.length).eq(3)
        expect(state.quorum.quorum).eq(3)
        expect(tr1.state).eq(TransactionState.Executed)
        expect(tr2.state).eq(TransactionState.Executed)
    });
})

