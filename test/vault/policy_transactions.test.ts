import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {idlFactory} from "./sdk_prototype/idl";
import {ActorMethod} from "@dfinity/agent";
import {
    Approve,
    Currency,
    Network,
    PolicyCreateTransaction,
    PolicyCreateTransactionRequest,
    PolicyRemoveTransaction,
    PolicyRemoveTransactionRequest,
    PolicyUpdateTransaction,
    PolicyUpdateTransactionRequest,
    Transaction,
    TransactionState,
    TransactionType,
    VaultManager,
    WalletCreateTransaction
} from "./sdk_prototype/vault_manager";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {getTransactionByIdFromGetAllTrs, verifyTransaction} from "./member_transactions.test";
import {requestCreateWalletTransaction} from "./wallet_transactions.test";
import {expect} from "chai";

require('./bigintExtension.js');

describe("Policy Transactions", () => {
    let admin_actor_1: Record<string, ActorMethod>;
    let member_actor_1: Record<string, ActorMethod>;
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
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

    const walletName = "testWallet"
    const walletName_2 = "testWallet2"
    let walletUid_1;
    let walletUid_2;
    let policyUid;
    let policyUid2;
    it("CreatePolicy approved and executed", async function () {
        let walletResponse = await requestCreateWalletTransaction(manager, walletName, Network.IC);
        walletUid_1 = (walletResponse[0] as WalletCreateTransaction).uid
        await manager.execute()
        let policyCreateResponse = await requestCreatePolicyTransaction(manager, 2, 2n, [walletUid_1])
        policyUid = (policyCreateResponse[0] as PolicyCreateTransaction).uid
        let expected = buildExpectedPoliceCreateTransaction(policyCreateResponse[0], TransactionState.Approved)
        expected.wallets = [walletUid_1]
        expected.member_threshold = 2
        expected.amount_threshold = 2n
        verifyPolicyCreateTransaction(expected, policyCreateResponse[0] as PolicyCreateTransaction)
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id)
        expected.state = TransactionState.Executed
        verifyPolicyCreateTransaction(expected, tr)
        let state = await manager.redefineState()
        expect(state.policies.length).eq(1)
        expect(state.policies[0].amount_threshold).eq(2n)
        expect(state.policies[0].member_threshold).eq(2)
        expect(state.policies[0].wallets.length).eq(1)
        expect(state.policies[0].wallets[0]).eq(walletUid_1)
    });

    it("CreatePolicy rejected because of unexist wallet", async function () {
        let policyCreateResponse = await requestCreatePolicyTransaction(manager, 3, 3n, ["test_unex_uid"])
        await manager.execute()
        let expected = buildExpectedPoliceCreateTransaction(policyCreateResponse[0], TransactionState.Rejected)
        expected.wallets = ["test_unex_uid"]
        expected.member_threshold = 3
        expected.amount_threshold = 3n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id)
        verifyPolicyCreateTransaction(expected, tr)
        let state = await manager.redefineState()
        expect(state.policies.length).eq(1)
    });

    it("CreatePolicy rejected because of non-existent wallet", async function () {
        let policyCreateResponse = await requestCreatePolicyTransaction(manager, 3, 2n, [walletUid_1])
        await manager.execute()
        let expected = buildExpectedPoliceCreateTransaction(policyCreateResponse[0], TransactionState.Rejected)
        expected.wallets = [walletUid_1]
        expected.member_threshold = 3
        expected.amount_threshold = 2n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id)
        verifyPolicyCreateTransaction(expected, tr)
        let state = await manager.redefineState()
        expect(state.policies.length).eq(1)
    });

    it("UpdatePolicy executed", async function () {
        let policyCreateResponse = await requestUpdatePolicyTransaction(manager, 3, 3n, policyUid)
        await manager.execute()
        let expected = buildExpectedPoliceUpdateTransaction(policyCreateResponse[0], TransactionState.Executed)
        expected.member_threshold = 3
        expected.amount_threshold = 3n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id)
        verifyPolicyUpdateTransaction(expected, tr)
        let state = await manager.redefineState()
        expect(state.policies.length).eq(1)
        expect(state.policies[0].amount_threshold).eq(3n)
        expect(state.policies[0].member_threshold).eq(3)
        expect(state.policies[0].wallets.length).eq(1)
        expect(state.policies[0].wallets[0]).eq(walletUid_1)
    });

    it("UpdatePolicy only amount executed", async function () {
        let policyCreateResponse = await requestUpdatePolicyTransaction(manager, 4, 3n, policyUid)
        await manager.execute()
        let expected = buildExpectedPoliceUpdateTransaction(policyCreateResponse[0], TransactionState.Executed)
        expected.member_threshold = 4
        expected.amount_threshold = 3n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id)
        verifyPolicyUpdateTransaction(expected, tr)
        let state = await manager.redefineState()
        expect(state.policies.length).eq(1)
        expect(state.policies[0].amount_threshold).eq(3n)
        expect(state.policies[0].member_threshold).eq(4)
        expect(state.policies[0].wallets.length).eq(1)
        expect(state.policies[0].wallets[0]).eq(walletUid_1)
    });

    it("UpdatePolicy not exist rejected", async function () {
        let policyCreateResponse = await requestUpdatePolicyTransaction(manager, 4, 4n, "policyUidNE")
        await manager.execute()
        let expected = buildExpectedPoliceUpdateTransaction(policyCreateResponse[0], TransactionState.Rejected)
        expected.member_threshold = 4
        expected.amount_threshold = 4n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id)
        verifyPolicyUpdateTransaction(expected, tr)
        let state = await manager.redefineState()
        expect(state.policies.length).eq(1)
    });

    it("UpdatePolicy amount exist - rejected", async function () {
        let resp = await requestCreateWalletTransaction(manager, walletName_2, Network.IC);
        walletUid_2 = (resp[0] as WalletCreateTransaction).uid
        let policyCreateResponse = await requestCreatePolicyTransaction(manager, 4, 4n, [walletUid_2])
        policyUid2 = (policyCreateResponse[0] as WalletCreateTransaction).uid
        await manager.execute()
        let policyUpdateResponse = await requestUpdatePolicyTransaction(manager, 3, 3n, policyUid2)
        await manager.execute()

        let expected = buildExpectedPoliceUpdateTransaction(policyUpdateResponse[0], TransactionState.Rejected)
        expected.member_threshold = 3
        expected.amount_threshold = 3n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyUpdateResponse[0].id)
        verifyPolicyUpdateTransaction(expected, tr)
        let state = await manager.redefineState()
        expect(state.policies.length).eq(2)
        expect(state.policies[1].amount_threshold).eq(4n)
        expect(state.policies[1].member_threshold).eq(4)
        expect(state.policies[1].wallets.length).eq(1)
        expect(state.policies[1].wallets[0]).eq(walletUid_2)
    });

    it("RemovePolicy not exist rejected", async function () {
        let policyRemoveResponse = await requestRemovePolicyTransaction(manager, "nonexistentUID")
        await manager.execute()
        let expected = buildExpectedPolicyRemoveTransaction(policyRemoveResponse[0], TransactionState.Rejected)
        expected.uid = "nonexistentUID"
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyRemoveResponse[0].id)
        verifyPolicyRemoveTransaction(expected, tr)
        let state = await manager.redefineState()
        expect(state.policies.length).eq(2)
    });

    it("RemovePolicy executed", async function () {
        let policyRemoveResponse = await requestRemovePolicyTransaction(manager, policyUid2)
        await manager.execute()
        let expected = buildExpectedPolicyRemoveTransaction(policyRemoveResponse[0], TransactionState.Executed)
        expected.uid = policyUid2
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyRemoveResponse[0].id)
        verifyPolicyRemoveTransaction(expected, tr)
        let state = await manager.redefineState()
        expect(state.policies.length).eq(1)
    });


    function buildExpectedPoliceCreateTransaction(actualTr, state) {
        let expectedApprove: Approve = {
            createdDate: actualTr.approves[0].createdDate,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }

        let expectedTrs: PolicyCreateTransaction = {
            amount_threshold: 2n,
            currency: Currency.ICP,
            member_threshold: 2,
            wallets: [],
            modifiedDate: actualTr.modifiedDate,
            uid: actualTr.uid,
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: actualTr.createdDate,
            id: actualTr.id,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            state,
            transactionType: TransactionType.PolicyCreate
        }
        return expectedTrs
    }

    function buildExpectedPoliceUpdateTransaction(actualTr, state) {
        let expectedApprove: Approve = {
            createdDate: actualTr.approves[0].createdDate,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }

        let expectedTrs: PolicyUpdateTransaction = {
            amount_threshold: 3n,
            member_threshold: 3,
            modifiedDate: actualTr.modifiedDate,
            uid: actualTr.uid,
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: actualTr.createdDate,
            id: actualTr.id,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            state,
            transactionType: TransactionType.PolicyUpdate
        }
        return expectedTrs
    }


    function buildExpectedPolicyRemoveTransaction(actualTr, state) {
        let expectedApprove: Approve = {
            createdDate: actualTr.approves[0].createdDate,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }

        let expectedTrs: PolicyRemoveTransaction = {
            modifiedDate: actualTr.modifiedDate,
            uid: actualTr.uid,
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: actualTr.createdDate,
            id: actualTr.id,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            state,
            transactionType: TransactionType.PolicyUpdate
        }
        return expectedTrs
    }

})


export function verifyPolicyCreateTransaction(expected: PolicyCreateTransaction, actual: PolicyCreateTransaction) {
    expect(expected.amount_threshold).eq(actual.amount_threshold)
    expect(expected.currency).eq(actual.currency)
    expect(expected.member_threshold).eq(actual.member_threshold)
    for (const a of actual.wallets) {
        let b = expected.wallets.find(x => x === a)
        expect(b).not.eq(undefined)
    }
    expect(actual.uid).not.eq("")
    expect(actual.uid).not.eq(undefined)
    verifyTransaction(expected, actual, TransactionType.PolicyCreate)
}


export function verifyPolicyUpdateTransaction(expected: PolicyUpdateTransaction, actual: PolicyUpdateTransaction) {
    expect(expected.amount_threshold).eq(actual.amount_threshold)
    expect(expected.member_threshold).eq(actual.member_threshold)
    expect(actual.uid).not.eq("")
    expect(actual.uid).not.eq(undefined)
    verifyTransaction(expected, actual, TransactionType.PolicyUpdate)
}

export function verifyPolicyRemoveTransaction(expected: PolicyRemoveTransaction, actual: PolicyRemoveTransaction) {
    expect(expected.uid).eq(actual.uid)
    verifyTransaction(expected, actual, TransactionType.PolicyUpdate)
}

export async function requestCreatePolicyTransaction(manager, membersTr, amountTr, wallets): Promise<Array<Transaction>> {
    let memberR = new PolicyCreateTransactionRequest(membersTr, amountTr, wallets);
    return await manager.requestTransaction([memberR])
}

export async function requestRemovePolicyTransaction(manager, uid): Promise<Array<Transaction>> {
    let memberR = new PolicyRemoveTransactionRequest(uid);
    return await manager.requestTransaction([memberR])
}


export async function requestUpdatePolicyTransaction(manager, membersTr, amountTr, uid): Promise<Array<Transaction>> {
    let memberR = new PolicyUpdateTransactionRequest(uid, membersTr, amountTr);
    return await manager.requestTransaction([memberR])
}


