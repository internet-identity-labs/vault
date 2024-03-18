import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {ActorMethod} from "@dfinity/agent";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {
    getTransactionByIdFromGetAllTrs,
    requestCreateMemberTransaction,
    requestUpdateQuorumTransaction, verifyTransaction
} from "./helper";
import {Approve, idlFactory, TransactionState, TransactionType, VaultManager, VaultRole} from "./sdk";
import {Transaction} from "./sdk/transaction/transaction";
import {QuorumUpdateTransaction} from "./sdk/transaction/config/quorum_update";

require('./bigintextension.js');

describe("Quorum Transactions", () => {
    let admin_actor_1: Record<string, ActorMethod>;
    let member_actor_1: Record<string, ActorMethod>;
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))
        const admin = getIdentity("87654321876543218765432187654321");
        const member = getIdentity("87654321876543218765432187654320");
        canister_id = DFX.GET_CANISTER_ID("vault");
        admin_actor_1 = await getActor(canister_id, admin, idlFactory);
        member_actor_1 = await getActor(canister_id, member, idlFactory);
        manager = new VaultManager(canister_id, admin_identity);
        await manager.resetToLocalEnv();
    });

    after(() => {
        DFX.STOP();
    });

    it("UpdateQuorum ADMIN rejected because of amount of admins less than quorum", async function () {
        let trReqResp: Array<Transaction> = await requestUpdateQuorumTransaction(manager, 2)
        let trId = trReqResp[0].id
        await manager.execute();
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId)
        let expectedTrs: QuorumUpdateTransaction = buildExpectedQuorumTransaction(TransactionState.Rejected, 2)
        verifyQuorumUpdateTransaction(expectedTrs, tr as QuorumUpdateTransaction)
        let state = await manager.getState();
        expect(state.quorum.quorum).eq(1)
    });

    it("UpdateQuorum ADMIN approved + executed", async function () {
        await requestCreateMemberTransaction(manager, "memberAddress2", "memberName", VaultRole.ADMIN)
        await manager.execute()
        let trReqResp: Array<Transaction> = await requestUpdateQuorumTransaction(manager, 2)
        let trId = trReqResp[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId)
        let expectedTrs: QuorumUpdateTransaction = buildExpectedQuorumTransaction(TransactionState.Approved, 2)
        verifyQuorumUpdateTransaction(tr as QuorumUpdateTransaction, trReqResp[0] as QuorumUpdateTransaction)
        verifyQuorumUpdateTransaction(expectedTrs, tr as QuorumUpdateTransaction)
        await manager.execute();
        let state = await manager.getState();
        expect(state.quorum.quorum).eq(2)
    });


    function buildExpectedQuorumTransaction(state, quorum) {
        let expectedApprove: Approve = {
            createdDate: 0n,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }
        let expectedTrs: QuorumUpdateTransaction = {
            modifiedDate: 0n,
            quorum: quorum,
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: 0n,
            id: 0n,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            state,
            transactionType: TransactionType.QuorumUpdate
        }
        return expectedTrs
    }

})

function verifyQuorumUpdateTransaction(expected: QuorumUpdateTransaction, actual: QuorumUpdateTransaction) {
    expect(expected.quorum).eq(actual.quorum)
    verifyTransaction(expected, actual, TransactionType.QuorumUpdate)
}

