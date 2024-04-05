import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {
    getTransactionByIdFromGetAllTrs,
    requestCreateMemberTransaction,
    requestRemoveMemberTransaction,
    requestUpdateMemberNameTransaction,
    requestUpdateMemberRoleTransaction,
    verifyTransaction
} from "./helper";
import {
    Approve,
    MemberCreateTransaction, MemberRemoveTransaction, MemberUpdateNameTransaction, MemberUpdateRoleTransaction,
    Transaction,
    TransactionState, TransactionType,
    VaultManager,
    VaultRole
} from "@nfid/vaults";

require('./bigintextension.js');

describe("Member Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))

        const member = getIdentity("87654321876543218765432187654320");
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager(canister_id, admin_identity);
        await manager.resetToLocalEnv();
    });

    after(() => {
        DFX.STOP();
    });

    let adminAddress = principalToAddress(admin_identity.getPrincipal() as any);

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
        let expectedTrs: MemberCreateTransaction = buildExpectedCreateMemberTransaction(TransactionState.Approved)
        //verify transaction from the response
        verifyCreateMemberTransaction(tr as MemberCreateTransaction, trReqResp[0] as MemberCreateTransaction)
        //verify transaction from the getAll
        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)

        await manager.execute();
        let state = await manager.getState();
        let member = state.members.find(m => m.userId === memberAddress)

        expect(member.name).eq(memberName)
        expect(member.role).eq(VaultRole.MEMBER)

        tr = await getTransactionByIdFromGetAllTrs(manager, trId)
        expectedTrs = buildExpectedCreateMemberTransaction(TransactionState.Executed)
        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)
    });


    it("CreateMemberTransaction rejected because of same member", async function () {
        let trReqResp: Array<Transaction> = await requestCreateMemberTransaction(manager, memberAddress, memberName, memberRole)
        let trId = trReqResp[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        verifyCreateMemberTransaction(tr as MemberCreateTransaction, trReqResp[0] as MemberCreateTransaction)

        await manager.execute()

        tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedTrs: MemberCreateTransaction = buildExpectedCreateMemberTransaction(TransactionState.Rejected)
        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)

        let state = await manager.getState();
        let members = state.members.filter(m => m.userId).length
        expect(members).eq(2)
    });

    it("CreateMemberTransaction Blocked, then execute and verify second member", async function () {
        let approvedButNotExecuted = await requestCreateMemberTransaction(manager, memberAddress2, memberName, VaultRole.MEMBER)

        let trReqRespBlocked: Array<Transaction> = await requestCreateMemberTransaction(manager, memberAddress, memberName, memberRole)
        let trId = trReqRespBlocked[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        verifyCreateMemberTransaction(tr as MemberCreateTransaction, trReqRespBlocked[0] as MemberCreateTransaction)

        let expectedTrs: MemberCreateTransaction = buildExpectedCreateMemberTransaction(TransactionState.Blocked)
        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)

        tr = await getTransactionByIdFromGetAllTrs(manager, trId)
        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)
        let state = await manager.getState();
        let members = state.members.length
        expect(members).eq(2)

        await manager.execute();
        tr = await getTransactionByIdFromGetAllTrs(manager, approvedButNotExecuted[0].id);
        expectedTrs = buildExpectedCreateMemberTransaction(TransactionState.Executed)
        expectedTrs.memberId = memberAddress2

        verifyCreateMemberTransaction(expectedTrs, tr as MemberCreateTransaction)
        state = await manager.getState();
        members = state.members.filter(m => m.userId).length
        expect(members).eq(3)

        let member = state.members.find(m => m.userId === memberAddress2)

        expect(member.userId).eq(memberAddress2)
        expect(member.name).eq(memberName)
        expect(member.role).eq(VaultRole.MEMBER)
    });


    it("UpdateMemberNameTransaction approved + executed", async function () {
        let updateNameTrResponse: Array<Transaction> = await requestUpdateMemberNameTransaction(manager, memberAddress, memberName2);
        let state = await manager.getState();

        let member = state.members.find(m => m.userId === memberAddress)

        expect(member.name).eq(memberName)

        let trId = updateNameTrResponse[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedUpdateNameTransaction(tr, TransactionState.Approved, memberName2)

        verifyUpdateMemberNameTransaction(expectedUpdTrs, tr)
        await manager.execute()

        state = await manager.getState();
        member = state.members.find(m => m.userId === memberAddress)
        expect(member.name).eq(memberName2)
    });

    it("UpdateMemberNameTransaction rejected because of no such member", async function () {
        let updateNameTrResponse: Array<Transaction> = await requestUpdateMemberNameTransaction(manager, memberAddress3, memberName3);
        let trId = updateNameTrResponse[0].id
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedUpdateNameTransaction(tr, TransactionState.Rejected, memberName3)
        expectedUpdTrs.memberId = memberAddress3
        verifyUpdateMemberNameTransaction(expectedUpdTrs, tr)
    });


    it("UpdateMemberRoleTransaction approved + executed", async function () {
        let updateRoleTrResponse: Array<Transaction> = await requestUpdateMemberRoleTransaction(manager, memberAddress, VaultRole.ADMIN);
        let state = await manager.getState();

        let member = state.members.find(m => m.userId === memberAddress)

        expect(member.role).eq(VaultRole.MEMBER)

        let trId = updateRoleTrResponse[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedUpdateRoleTransaction(tr, TransactionState.Approved, VaultRole.ADMIN)

        verifyUpdateMemberRoleTransaction(expectedUpdTrs, tr)
        await manager.execute()

        state = await manager.getState();
        member = state.members.find(m => m.userId === memberAddress)
        expect(member.role).eq(VaultRole.ADMIN)
    });

    it("UpdateMemberRoleTransaction rejected because of no such member", async function () {
        let updateNameTrResponse: Array<Transaction> = await requestUpdateMemberRoleTransaction(manager, memberAddress3, VaultRole.MEMBER);
        let trId = updateNameTrResponse[0].id
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedUpdateRoleTransaction(tr, TransactionState.Rejected, VaultRole.MEMBER)
        expectedUpdTrs.memberId = memberAddress3
        verifyUpdateMemberRoleTransaction(expectedUpdTrs, tr)
    });

    it("RemoveMember approved + executed", async function () {
        let state = await manager.getState();
        let member = state.members.find(m => m.userId === memberAddress2)
        expect(member).not.eq(undefined)

        let updateNameTrResponse: Array<Transaction> = await requestRemoveMemberTransaction(manager, memberAddress);
        let trId = updateNameTrResponse[0].id
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);

        let expectedUpdTrs = buildExpectedRemoveMemberTransaction(tr, TransactionState.Approved)
        expectedUpdTrs.memberId = memberAddress;
        verifyRemoveMemberTransaction(expectedUpdTrs, tr)

        await manager.execute()

        tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        expectedUpdTrs = buildExpectedRemoveMemberTransaction(tr, TransactionState.Executed)
        expectedUpdTrs.memberId = memberAddress;
        verifyRemoveMemberTransaction(expectedUpdTrs, tr)

        state = await manager.getState();
        member = state.members.find(m => m.userId === memberAddress)
        expect(member).eq(undefined)
    });


    it("UpdateMemberRoleTransaction rejected because of less than 1 admin", async function () {
        let updateNameTrResponse: Array<Transaction> = await requestUpdateMemberRoleTransaction(manager, adminAddress, VaultRole.MEMBER);
        let trId = updateNameTrResponse[0].id
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedUpdateRoleTransaction(tr, TransactionState.Rejected, VaultRole.MEMBER)
        expectedUpdTrs.memberId = adminAddress
        verifyUpdateMemberRoleTransaction(expectedUpdTrs, tr)
    });


    it("RemoveMember rejected because 1 admin", async function () {
        let state = await manager.getState();
        let member = state.members.find(m => m.userId === adminAddress)
        expect(member.role).eq(VaultRole.ADMIN)

        let updateNameTrResponse: Array<Transaction> = await requestRemoveMemberTransaction(manager, adminAddress);
        let trId = updateNameTrResponse[0].id
        await manager.execute()

        let tr = await getTransactionByIdFromGetAllTrs(manager, trId);
        let expectedUpdTrs = buildExpectedRemoveMemberTransaction(tr, TransactionState.Rejected)
        expectedUpdTrs.memberId = adminAddress;
        verifyRemoveMemberTransaction(expectedUpdTrs, tr)

        state = await manager.getState();
        member = state.members.find(m => m.userId === adminAddress)
        expect(member.role).eq(VaultRole.ADMIN)
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
            memberId: memberAddress,
            modifiedDate: actualTr.modifiedDate,
            name,
            state,
            transactionType: TransactionType.MemberUpdateName
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
            memberId: memberAddress,
            modifiedDate: actualTr.modifiedDate,
            role,
            state,
            transactionType: TransactionType.MemberUpdateRole
        }
        return expectedTrs
    }

    function buildExpectedRemoveMemberTransaction(actualTr, state) {
        let expectedApprove: Approve = {
            createdDate: actualTr.approves[0].createdDate,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }
        let expectedTrs: MemberRemoveTransaction = {
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: actualTr.createdDate,
            id: actualTr.id,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            memberId: memberAddress,
            modifiedDate: actualTr.modifiedDate,
            state,
            transactionType: TransactionType.MemberRemove
        }
        return expectedTrs
    }


    function buildExpectedCreateMemberTransaction(state) {
        let expectedApprove: Approve = {
            createdDate: 0n,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }
        let expectedTrs: MemberCreateTransaction = {
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: 0n,
            id: 0n,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            memberId: memberAddress,
            modifiedDate: 0n,
            name: memberName,
            role: memberRole,
            state,
            transactionType: TransactionType.MemberCreate
        }
        return expectedTrs
    }

})

function verifyUpdateMemberNameTransaction(expected: MemberUpdateNameTransaction, actual: MemberUpdateNameTransaction) {
    expect(expected.name).eq(actual.name)
    expect(expected.memberId).eq(actual.memberId)
    verifyTransaction(expected, actual, TransactionType.MemberUpdateName)
}

function verifyRemoveMemberTransaction(expected: MemberRemoveTransaction, actual: MemberRemoveTransaction) {
    expect(expected.memberId).eq(actual.memberId)
    verifyTransaction(expected, actual, TransactionType.MemberRemove)
}

function verifyUpdateMemberRoleTransaction(expected: MemberUpdateRoleTransaction, actual: MemberUpdateRoleTransaction) {
    expect(expected.role).eq(actual.role)
    expect(expected.memberId).eq(actual.memberId)
    verifyTransaction(expected, actual, TransactionType.MemberUpdateRole)
}

function verifyCreateMemberTransaction(expected: MemberCreateTransaction, actual: MemberCreateTransaction) {
    expect(expected.name).eq(actual.name)
    expect(expected.role).eq(actual.role)
    expect(expected.memberId).eq(actual.memberId)
    verifyTransaction(expected, actual, TransactionType.MemberCreate)
}
