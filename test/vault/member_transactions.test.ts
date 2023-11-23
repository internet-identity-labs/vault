import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {idlFactory} from "./sdk_prototype/idl";
import {ActorMethod} from "@dfinity/agent";
import {
    Approve,
    MemberCreateTransaction,
    MemberCreateTransactionRequest,
    MemberUpdateNameTransaction,
    MemberUpdateNameTransactionRequest, MemberUpdateRoleTransaction,
    MemberUpdateRoleTransactionRequest,
    Transaction,
    TransactionState,
    TransactionType,
    VaultManager,
    VaultRole
} from "./sdk_prototype/vault_manager";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";

require('./bigintExtension.js');

describe("Member Transactions", () => {
    let admin_actor_1: Record<string, ActorMethod>;
    let member_actor_1: Record<string, ActorMethod>;
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        // await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))
        const admin = getIdentity("87654321876543218765432187654321");
        const member = getIdentity("87654321876543218765432187654320");
        canister_id = DFX.GET_CANISTER_ID("vault");
        admin_actor_1 = await getActor(canister_id, admin, idlFactory);
        member_actor_1 = await getActor(canister_id, member, idlFactory);
        manager = new VaultManager();
        await manager.init(canister_id, admin_identity, true);
    });

    after(() => {
        DFX.STOP();
    });

    let memberAddress = "testMemberAddress_1";
    let memberAddress2 = "testMemberAddress_2";
    let memberAddress3 = "testMemberAddress_3";
    let memberName = "name_1";
    let memberName2 = "name_2";
    let memberName3 = "name_3";
    let memberRole = VaultRole.MEMBER;

    it("CreateMemberTransaction approved + executed", async function () {
        let trReqResp: Array<Transaction> = await requestCreateMemberTransaction(manager, memberAddress, memberName, memberRole)
        let trId = trReqResp[0].id

        let tr = await getTransactionByIdFromGetAllTrs(manager, trId)
        //build transaction with expected fields but copy id and dates
        let expectedTrs: MemberCreateTransaction = buildExpectedCreateMemberTransaction(tr, TransactionState.Approved)
        //verify transaction from the response
        verifyCreateMemberTransaction(tr as MemberCreateTransaction, trReqResp[0] as MemberCreateTransaction)
        //verify transaction from the getAll
        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)

        await manager.execute();
        let state = await manager.redefineState();
        let member = state.members.find(m => m.userId)

        expect(member.userId).eq(memberAddress)
        expect(member.name).eq(memberName)
        expect(member.role).eq(VaultRole.MEMBER)

        tr = await getTransactionByIdFromGetAllTrs(manager, trId)
        expectedTrs = buildExpectedCreateMemberTransaction(tr, TransactionState.Executed)
        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)
    });


    it("CreateMemberTransaction rejected because of same member", async function () {
        let trReqResp: Array<Transaction> = await requestCreateMemberTransaction(manager, memberAddress, memberName, memberRole)
        let trId = trReqResp[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        verifyCreateMemberTransaction(tr as MemberCreateTransaction, trReqResp[0] as MemberCreateTransaction)

        await manager.execute()

        tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedTrs: MemberCreateTransaction = buildExpectedCreateMemberTransaction(tr, TransactionState.Rejected)

        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)

        let state = await manager.redefineState();
        let members = state.members.filter(m => m.userId).length
        expect(members).eq(1)
    });

    it("CreateMemberTransaction Blocked, then execute and verify second member", async function () {
        let approvedButNotExecuted = await requestCreateMemberTransaction(manager, memberAddress2, memberName, VaultRole.MEMBER)

        let trReqRespBlocked: Array<Transaction> = await requestCreateMemberTransaction(manager, memberAddress, memberName, memberRole)
        let trId = trReqRespBlocked[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);

        verifyCreateMemberTransaction(tr as MemberCreateTransaction, trReqRespBlocked[0] as MemberCreateTransaction)
        let expectedTrs: MemberCreateTransaction = buildExpectedCreateMemberTransaction(tr, TransactionState.Blocked)
        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)

        tr = await getTransactionByIdFromGetAllTrs(manager, trId)
        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)
        let state = await manager.redefineState();
        let members = state.members.length
        expect(members).eq(1)

        await manager.execute();
        tr = await getTransactionByIdFromGetAllTrs(manager,  approvedButNotExecuted[0].id);
        expectedTrs = buildExpectedCreateMemberTransaction(tr, TransactionState.Executed)

        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)
        state = await manager.redefineState();
        members = state.members.filter(m => m.userId).length
        expect(members).eq(2)

        let member = state.members.find(m => m.userId === memberAddress2)

        expect(member.userId).eq(memberAddress2)
        expect(member.name).eq(memberName)
        expect(member.role).eq(VaultRole.MEMBER)
    });


    it("UpdateMemberNameTransaction approved + executed", async function () {
        let updateNameTrResponse: Array<Transaction> = await requestUpdateMemberNameTransaction(manager, memberAddress, memberName2);
        let state = await manager.redefineState();

        let member = state.members.find(m => m.userId === memberAddress)

        expect(member.name).eq(memberName)

        let trId = updateNameTrResponse[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedUpdateNameTransaction(tr, TransactionState.Approved, memberName2)

        verifyUpdateMemberNameTransaction(expectedUpdTrs, tr)
        await manager.execute()

        state = await manager.redefineState();
        member = state.members.find(m => m.userId === memberAddress)
        expect(member.name).eq(memberName2)
    });

    it("UpdateMemberNameTransaction rejected because of no such member", async function () {
        let updateNameTrResponse: Array<Transaction> = await requestUpdateMemberNameTransaction(manager, memberAddress3, memberName3);
        let trId = updateNameTrResponse[0].id
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedUpdateNameTransaction(tr, TransactionState.Rejected, memberName3)
        verifyUpdateMemberNameTransaction(expectedUpdTrs, tr)
    });


    it("UpdateMemberRoleTransaction approved + executed", async function () {
        let updateRoleTrResponse: Array<Transaction> = await requestUpdateMemberRoleTransaction(manager, memberAddress, VaultRole.ADMIN);
        let state = await manager.redefineState();

        let member = state.members.find(m => m.userId === memberAddress)

        expect(member.role).eq(VaultRole.MEMBER)

        let trId = updateRoleTrResponse[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedUpdateRoleTransaction(tr, TransactionState.Approved, VaultRole.ADMIN)

        verifyUpdateMemberRoleTransaction(expectedUpdTrs, tr)
        await manager.execute()

        state = await manager.redefineState();
        member = state.members.find(m => m.userId === memberAddress)
        expect(member.role).eq(VaultRole.ADMIN)
    });

    it("UpdateMemberRoleTransaction rejected because of no such member", async function () {
        let updateNameTrResponse: Array<Transaction> = await requestUpdateMemberRoleTransaction(manager, memberAddress3, VaultRole.MEMBER);
        let trId = updateNameTrResponse[0].id
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedUpdateRoleTransaction(tr, TransactionState.Rejected, VaultRole.MEMBER)
        verifyUpdateMemberRoleTransaction(expectedUpdTrs, tr)
    });



    function buildExpectedUpdateNameTransaction(actualTr, state, name) {
        let expectedApprove: Approve = {
            createdDate: actualTr.approves[0].createdDate,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }
        let expectedTrs: MemberUpdateNameTransaction = {
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: actualTr.createdDate,
            id: actualTr.id,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            member_id: memberAddress,
            modifiedDate: actualTr.modifiedDate,
            name,
            state,
            transactionType: TransactionType.MemberCreate
        }
        return expectedTrs
    }

    function buildExpectedUpdateRoleTransaction(actualTr, state, role) {
        let expectedApprove: Approve = {
            createdDate: actualTr.approves[0].createdDate,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }
        let expectedTrs: MemberUpdateRoleTransaction = {
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: actualTr.createdDate,
            id: actualTr.id,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            member_id: memberAddress,
            modifiedDate: actualTr.modifiedDate,
            role,
            state,
            transactionType: TransactionType.MemberCreate
        }
        return expectedTrs
    }



    function buildExpectedCreateMemberTransaction(actualTr, state) {
        let expectedApprove: Approve = {
            createdDate: actualTr.approves[0].createdDate,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }
        let expectedTrs: MemberCreateTransaction = {
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: actualTr.createdDate,
            id: actualTr.id,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            member_id: memberAddress,
            modifiedDate: actualTr.modifiedDate,
            name: memberName,
            role: memberRole,
            state,
            transactionType: TransactionType.MemberCreate
        }
        return expectedTrs
    }

})


async function requestUpdateMemberNameTransaction(manager, memberAddress, memberName): Promise<Array<Transaction>> {
    let memberR = new MemberUpdateNameTransactionRequest(memberAddress, memberName);
    return await manager.requestTransaction([memberR])
}


async function requestUpdateMemberRoleTransaction(manager, memberAddress, memberRole): Promise<Array<Transaction>> {
    let memberR = new MemberUpdateRoleTransactionRequest(memberAddress, memberRole);
    return await manager.requestTransaction([memberR])
}


function verifyUpdateMemberNameTransaction(expected: MemberUpdateNameTransaction, actual: MemberUpdateNameTransaction) {
    expect(expected.name).eq(actual.name)
    expect(expected.id).eq(actual.id)
    expect(expected.state).eq(actual.state)
    expect(expected.batchUid).eq(actual.batchUid)
    expect(expected.initiator).eq(actual.initiator)
    expect(expected.createdDate).eq(actual.createdDate)
    expect(expected.modifiedDate).eq(actual.modifiedDate)
    expect(expected.isVaultState).eq(true)
    expect(expected.transactionType).eq(TransactionType.MemberCreate)
    expect(expected.memo).eq(actual.memo)
    expect(expected.approves.length).eq(actual.approves.length)
    for (const app of expected.approves) {
        const found = actual.approves.find((l) => l.signer === app.signer);
        verifyApprove(app, found)
    }
}

function verifyUpdateMemberRoleTransaction(expected: MemberUpdateRoleTransaction, actual: MemberUpdateRoleTransaction) {
    expect(expected.role).eq(actual.role)
    expect(expected.id).eq(actual.id)
    expect(expected.state).eq(actual.state)
    expect(expected.batchUid).eq(actual.batchUid)
    expect(expected.initiator).eq(actual.initiator)
    expect(expected.createdDate).eq(actual.createdDate)
    expect(expected.modifiedDate).eq(actual.modifiedDate)
    expect(expected.isVaultState).eq(true)
    expect(expected.transactionType).eq(TransactionType.MemberCreate)
    expect(expected.memo).eq(actual.memo)
    expect(expected.approves.length).eq(actual.approves.length)
    for (const app of expected.approves) {
        const found = actual.approves.find((l) => l.signer === app.signer);
        verifyApprove(app, found)
    }
}

export async function getTransactionByIdFromGetAllTrs(manager, trId) {
    let transactions = await manager.getTransactions();
    let tr = transactions.find(l => l.id === trId);
    return tr;
}

export async function requestCreateMemberTransaction(manager, memberAddress, memberName, memberRole): Promise<Array<Transaction>> {
    let memberR = new MemberCreateTransactionRequest(memberAddress, memberName, memberRole);
    return await manager.requestTransaction([memberR])
}


function verifyCreateMemberTransaction(expected: MemberCreateTransaction, actual: MemberCreateTransaction) {
    expect(expected.name).eq(actual.name)
    expect(expected.id).eq(actual.id)
    expect(expected.state).eq(actual.state)
    expect(expected.batchUid).eq(actual.batchUid)
    expect(expected.initiator).eq(actual.initiator)
    expect(expected.createdDate).eq(actual.createdDate)
    expect(expected.modifiedDate).eq(actual.modifiedDate)
    expect(expected.isVaultState).eq(true)
    expect(expected.transactionType).eq(TransactionType.MemberCreate)
    expect(expected.memo).eq(actual.memo)
    expect(expected.approves.length).eq(actual.approves.length)
    expect(expected.role).eq(actual.role)
    for (const app of expected.approves) {
        const found = actual.approves.find((l) => l.signer === app.signer);
        verifyApprove(app, found)
    }
}



function verifyApprove(expected: Approve, actual: Approve) {
    expect(expected.status).eq(actual.status)
    expect(expected.signer).eq(actual.signer)
    expect(expected.createdDate).eq(actual.createdDate)
}
