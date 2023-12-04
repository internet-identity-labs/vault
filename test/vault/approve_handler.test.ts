import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {ApproveRequest, Network, TransactionState, VaultManager, VaultRole} from "./sdk_prototype/vault_manager";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {getTransactionByIdFromGetAllTrs, requestCreateMemberTransaction} from "./member_transactions.test";
import {requestUpdateQuorumTransaction} from "./quorum_transactions.test";
import {requestCreateWalletTransaction} from "./wallet_transactions.test";
import {expect} from "chai";

require('./bigintExtension.js');

describe("Approve Handler Transactions", () => {
    let canister_id;
    let admin_identity1 = getIdentity("87654321876543218765432187654321")
    let admin_identity2 = getIdentity("87654321876543218765432187654322")
    let admin_identity3 = getIdentity("87654321876543218765432187654323")
    let member_identity4 = getIdentity("87654321876543218765432187654325")
    let manager1: VaultManager;
    let manager2: VaultManager;
    let manager3: VaultManager;
    let memberManager4: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager1 = new VaultManager();
        manager2 = new VaultManager();
        manager3 = new VaultManager();
        memberManager4 = new VaultManager();
        await manager1.init(canister_id, admin_identity1, true);
        await manager2.init(canister_id, admin_identity2, true);
        await manager3.init(canister_id, admin_identity3, true);
        await memberManager4.init(canister_id, member_identity4, true);
        await requestCreateMemberTransaction(manager1, principalToAddress(admin_identity2.getPrincipal() as any), "a2", VaultRole.ADMIN);
        await requestCreateMemberTransaction(manager1, principalToAddress(admin_identity3.getPrincipal() as any), "a3", VaultRole.ADMIN);
        await requestCreateMemberTransaction(manager1, principalToAddress(member_identity4.getPrincipal() as any), "m3", VaultRole.MEMBER);
        await requestUpdateQuorumTransaction(manager1, 2);
        await manager1.execute()
    });

    after(() => {
        DFX.STOP();
    });


    it("Trs approved by 1 and rejected by 2", async function () {
        let tr_id = (await requestUpdateQuorumTransaction(manager1, 3))[0].id
        let reject: ApproveRequest = {
            tr_id,
            state: TransactionState.Rejected
        }
        await manager2.approveTransaction([reject])
        let tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.approves.length).eq(2);
        expect(tr.state).eq(TransactionState.Pending);

        await manager3.approveTransaction([reject])
        tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.approves.length).eq(3);
        expect(tr.state).eq(TransactionState.Rejected);

        await manager1.execute();
        tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.state).eq(TransactionState.Rejected);
    });

    it("Trs approved by 2 and rejected by 1", async function () {
        let tr_id = (await requestUpdateQuorumTransaction(manager1, 3))[0].id
        let approve: ApproveRequest = {
            tr_id,
            state: TransactionState.Approved
        }
        await manager2.approveTransaction([approve])
        let tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.approves.length).eq(2);
        expect(tr.state).eq(TransactionState.Approved);

        await manager1.execute();
        tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.state).eq(TransactionState.Executed);
    });

    it("Trs approved by 3 and executed", async function () {
        let tr_id = (await requestCreateWalletTransaction(manager1, "2", Network.IC))[0].id
        let approve: ApproveRequest = {
            tr_id,
            state: TransactionState.Approved
        }
        await manager2.approveTransaction([approve])
        let tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.approves.length).eq(2);
        expect(tr.state).eq(TransactionState.Pending);

        try {
            await memberManager4.approveTransaction([approve])
        } catch (e) {
            expect(e.message).contains("Not permitted")
        }

        try {
            await manager1.approveTransaction([approve])
        } catch (e) {
            expect(e.message).contains("Already approved")
        }
        try {
            await manager1.approveTransaction([{
                tr_id,
                state: TransactionState.Rejected
            }])
        } catch (e) {
            expect(e.message).contains("Already approved")
        }

        await manager3.approveTransaction([approve])
        tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.approves.length).eq(3);
        expect(tr.state).eq(TransactionState.Approved);

        await manager1.execute();
        tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.state).eq(TransactionState.Executed);
    });

    it("Trs rejected by 1 from 3 and rejected", async function () {
        let tr_id = (await requestCreateWalletTransaction(manager1, "1", Network.IC))[0].id
        let approve: ApproveRequest = {
            tr_id,
            state: TransactionState.Rejected
        }
        await manager2.approveTransaction([approve])
        let tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.approves.length).eq(2);
        expect(tr.state).eq(TransactionState.Rejected);

        await manager1.execute();
        tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr.state).eq(TransactionState.Rejected);

        try {
            await manager2.approveTransaction([approve])
        } catch (e) {
            expect(e.message).contains("Transaction is immutable")
        }
    });

    it("Trs blocked and then executed", async function () {
        let tr_id = (await requestCreateMemberTransaction(manager1, "1", "1", VaultRole.MEMBER))[0].id
        let tr_id2 = (await requestCreateMemberTransaction(manager1, "2", "2", VaultRole.MEMBER))[0].id
        await manager1.execute();
        let tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id2));
        expect(tr.state).eq(TransactionState.Blocked);
        let approve: ApproveRequest = {
            tr_id,
            state: TransactionState.Approved
        }
        await manager2.approveTransaction([approve])
        await manager3.approveTransaction([approve])
        await manager1.execute();
        tr = (await getTransactionByIdFromGetAllTrs(manager1, tr_id2));
        let tr1 = (await getTransactionByIdFromGetAllTrs(manager1, tr_id));
        expect(tr1.state).eq(TransactionState.Executed);
        expect(tr.state).eq(TransactionState.Pending);
    });


})

