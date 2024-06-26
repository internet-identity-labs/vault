import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {expect} from "chai";
import {
    getTransactionByIdFromGetAllTrs,
    requestCreatePolicyTransaction,
    requestCreateWalletTransaction,
    requestRemovePolicyTransaction,
    requestUpdatePolicyTransaction,
    verifyTransaction
} from "./helper";
import {
    Approve,
    Currency,
    Network,
    TransactionState,
    TransactionType,
    VaultManager
} from "@nfid/vaults";
import {WalletCreateTransaction} from "@nfid/vaults";
import {PolicyCreateTransaction, PolicyCreateTransactionRequest} from "@nfid/vaults";
import {PolicyUpdateTransaction} from "@nfid/vaults";
import {PolicyRemoveTransaction} from "@nfid/vaults";
import {hasOwnProperty} from "@nfid/vaults";

require('./bigintextension.js');

describe("Policy Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))
        const admin = getIdentity("87654321876543218765432187654321");
        console.log(admin.getPrincipal().toText())
        const member = getIdentity("87654321876543218765432187654320");
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager(canister_id, admin_identity);
        await manager.resetToLocalEnv();
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
        let expected = buildExpectedPolicyCreateTransaction(policyCreateResponse[0], TransactionState.Approved)
        expected.wallets = [walletUid_1]
        expected.member_threshold = 2
        expected.amount_threshold = 2n
        verifyPolicyCreateTransaction(expected, policyCreateResponse[0] as PolicyCreateTransaction)
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id)
        expected.state = TransactionState.Executed
        verifyPolicyCreateTransaction(expected, tr)
        let state = await manager.getState()
        expect(state.policies.length).eq(1)
        expect(state.policies[0].amountThreshold).eq(2n)
        expect(state.policies[0].memberThreshold).eq(2)
        expect(state.policies[0].wallets.length).eq(1)
        expect(state.policies[0].wallets[0]).eq(walletUid_1)
    });

    it("CreatePolicy failed because of nonexistent wallet", async function () {
        let policyCreateResponse = await requestCreatePolicyTransaction(manager, 3, 3n, ["test_unex_uid"])
        await manager.execute()
        let expected = buildExpectedPolicyCreateTransaction(policyCreateResponse[0], TransactionState.Failed)
        expected.wallets = ["test_unex_uid"]
        expected.member_threshold = 3
        expected.amount_threshold = 3n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id)
        verifyPolicyCreateTransaction(expected, tr)
        expect(hasOwnProperty(tr.error, "WalletNotExists")).eq(true)
        let state = await manager.getState()
        expect(state.policies.length).eq(1)
    });

    it("CreatePolicy failed because threshold already exists wallet", async function () {
        let policyCreateResponse = await requestCreatePolicyTransaction(manager, 3, 2n, [walletUid_1])
        await manager.execute()
        let expected = buildExpectedPolicyCreateTransaction(policyCreateResponse[0], TransactionState.Failed)
        expected.wallets = [walletUid_1]
        expected.member_threshold = 3
        expected.amount_threshold = 2n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id) as PolicyCreateTransaction
        verifyPolicyCreateTransaction(expected, tr)
        expect(hasOwnProperty(tr.error, "ThresholdAlreadyExists")).eq(true)
        let state = await manager.getState()
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
        let state = await manager.getState()
        expect(state.policies.length).eq(1)
        expect(state.policies[0].amountThreshold).eq(3n)
        expect(state.policies[0].memberThreshold).eq(3)
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
        let state = await manager.getState()
        expect(state.policies.length).eq(1)
        expect(state.policies[0].amountThreshold).eq(3n)
        expect(state.policies[0].memberThreshold).eq(4)
        expect(state.policies[0].wallets.length).eq(1)
        expect(state.policies[0].wallets[0]).eq(walletUid_1)
    });

    it("UpdatePolicy not exist failed", async function () {
        let policyCreateResponse = await requestUpdatePolicyTransaction(manager, 4, 4n, "policyUidNE")
        await manager.execute()
        let expected = buildExpectedPoliceUpdateTransaction(policyCreateResponse[0], TransactionState.Failed)
        expected.member_threshold = 4
        expected.amount_threshold = 4n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyCreateResponse[0].id)
        verifyPolicyUpdateTransaction(expected, tr)
        let state = await manager.getState()
        expect(state.policies.length).eq(1)
    });

    it("UpdatePolicy amount exist - failed", async function () {
        let resp = await requestCreateWalletTransaction(manager, walletName_2, Network.IC);
        walletUid_2 = (resp[0] as WalletCreateTransaction).uid
        let policyCreateResponsePredefined = await requestCreatePolicyTransaction(manager, 3, 3n, [walletUid_2])
        let policyCreateResponse = await requestCreatePolicyTransaction(manager, 4, 4n, [walletUid_2])
        policyUid2 = (policyCreateResponse[0] as WalletCreateTransaction).uid
        await manager.execute()
        let policyUpdateResponse = await requestUpdatePolicyTransaction(manager, 3, 3n, policyUid2)
        await manager.execute()

        let expected = buildExpectedPoliceUpdateTransaction(policyUpdateResponse[0], TransactionState.Failed)
        expected.member_threshold = 3
        expected.amount_threshold = 3n
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyUpdateResponse[0].id)
        verifyPolicyUpdateTransaction(expected, tr)
        let state = await manager.getState()
        expect(state.policies.length).eq(3)
        expect(state.policies[2].amountThreshold).eq(4n)
        expect(state.policies[2].memberThreshold).eq(4)
        expect(state.policies[2].wallets.length).eq(1)
        expect(state.policies[2].wallets[0]).eq(walletUid_2)
    });

    it("RemovePolicy not exist failed", async function () {
        let policyRemoveResponse = await requestRemovePolicyTransaction(manager, "nonexistentUID")
        await manager.execute()
        let expected = buildExpectedPolicyRemoveTransaction(TransactionState.Failed)
        expected.uid = "nonexistentUID"
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyRemoveResponse[0].id)
        verifyPolicyRemoveTransaction(expected, tr)
        let state = await manager.getState()
        expect(state.policies.length).eq(3)
    });

    it("RemovePolicy executed", async function () {
        let policyRemoveResponse = await requestRemovePolicyTransaction(manager, policyUid2)
        await manager.execute()
        let expected = buildExpectedPolicyRemoveTransaction(TransactionState.Executed)
        expected.uid = policyUid2
        let tr = await getTransactionByIdFromGetAllTrs(manager, policyRemoveResponse[0].id)
        verifyPolicyRemoveTransaction(expected, tr)
        let state = await manager.getState()
        expect(state.policies.length).eq(2)
        expect(state.policies[0]).not.eq(policyUid2)
    });

    it("Request 2 Policy With The Same Uid failed", async function () {
        let request1 = new PolicyCreateTransactionRequest("uniqueId", 2, 10n, [walletUid_1]);
        let response1 = await manager.requestTransaction([request1])
        await manager.execute()
        let transaction1 = await getTransactionByIdFromGetAllTrs(manager, response1[0].id) as PolicyCreateTransaction
        expect(transaction1.state).eq(TransactionState.Executed)
        let request2 = new PolicyCreateTransactionRequest("uniqueId", 2, 15n, [walletUid_1]);
        let response2 = await manager.requestTransaction([request2])
        await manager.execute()
        let transaction2 = await getTransactionByIdFromGetAllTrs(manager, response2[0].id) as PolicyCreateTransaction
        expect(transaction2.state).eq(TransactionState.Failed)
        expect(hasOwnProperty(transaction2.error, "UIDAlreadyExists")).eq(true)
    });

    function buildExpectedPolicyCreateTransaction(actualTr, state) {
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


    function buildExpectedPolicyRemoveTransaction(state) {
        let expectedApprove: Approve = {
            createdDate: 0n,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }

        let expectedTrs: PolicyRemoveTransaction = {
            modifiedDate: 0n,
            uid: "",
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: 0n,
            id: 0n,
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


