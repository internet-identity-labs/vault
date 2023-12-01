import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {
    MemberCreateTransactionRequest,
    QuorumTransactionRequest, TransactionState,
    VaultManager,
    VaultRole
} from "./sdk_prototype/vault_manager";
import {execute} from "../util/call.util";
import {getTransactionByIdFromGetAllTrs} from "./member_transactions.test";
import {expect} from "chai";

require('./bigintExtension.js');

describe("Batch Transactions", () => {
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

    it("Request batch transaction - rejected", async function () {
        let memberCreate =  new MemberCreateTransactionRequest("memberAddress", "memberName", VaultRole.ADMIN);
        let quorumTransactionRequest = new QuorumTransactionRequest(3);
        let batchUid = "someRandomGeneratedUID_1"
        memberCreate.batch_uid = batchUid
        quorumTransactionRequest.batch_uid = batchUid
        let res = await manager.requestTransaction([memberCreate, quorumTransactionRequest])
        await manager.execute()
        let state = await manager.redefineState();
        let tr1 = await getTransactionByIdFromGetAllTrs(manager, res[0].id)
        let tr2 = await getTransactionByIdFromGetAllTrs(manager, res[1].id)
        expect(state.members.length).eq(1)
        expect(state.quorum.quorum).eq(1)
        expect(tr1.state).eq(TransactionState.Rejected)
        expect(tr2.state).eq(TransactionState.Rejected)
    });

    it("Request batch transaction - executed", async function () {
        let memberCreate =  new MemberCreateTransactionRequest("memberAddress", "memberName", VaultRole.ADMIN);
        let quorumTransactionRequest = new QuorumTransactionRequest(2);
        let batchUid = "someRandomGeneratedUID_2"
        memberCreate.batch_uid = batchUid
        quorumTransactionRequest.batch_uid = batchUid
        let res = await manager.requestTransaction([memberCreate, quorumTransactionRequest])
        await manager.execute()
        let state = await manager.redefineState();
        let tr1 = await getTransactionByIdFromGetAllTrs(manager, res[0].id)
        let tr2 = await getTransactionByIdFromGetAllTrs(manager, res[1].id)
        expect(state.members.length).eq(2)
        expect(state.quorum).eq(2)
        expect(tr1.state).eq(TransactionState.Executed)
        expect(tr2.state).eq(TransactionState.Executed)
    });


})

