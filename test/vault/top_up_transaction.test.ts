import {getIdentity} from "../util/deployment.util";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {
    getTransactionByIdFromGetAllTrs, requestCreateMemberTransaction, requestCreatePolicyTransaction,
    requestCreateWalletTransaction, requestTopUpQuorumTransaction,
    requestTopUpTransaction,
    verifyTransaction
} from "./helper";
import {sleep} from "../util/call.util";
import {Approve,
    ApproveRequest, Currency, WalletCreateTransaction, Network, TopUpTransaction, TransactionState, TransactionType, VaultManager, VaultRole} from "@nfid/vaults";

require('./bigintextension.js');

describe("TopUp Transactions", () => {
    //predefined wallet address to fill with ICP manually
    const walletAddress = "706ab8c2d9585942dc4bdc5ed73188d7f56f97374a36b63b08ca45456ae699e3";
    const walletUid = "ba7f3c8953f15ae2e66f8def7d3a7c388e5af7f35e9c74f7d95aa3faa4b20c22";
    let canisterId = "hygn6-wiaaa-aaaal-qcr7a-cai";
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let member_identity = getIdentity("87654321876543218765432187654322")
    console.log(admin_identity.getPrincipal().toText())
    let manager: VaultManager;
    let member_manager: VaultManager;
    before(async () => {
        manager = new VaultManager(canisterId, admin_identity);
        await requestCreateMemberTransaction(manager, principalToAddress(member_identity.getPrincipal() as any), "DummeMember", VaultRole.MEMBER);
        member_manager = new VaultManager(canisterId, member_identity);
    });

    it("Trs approved and transferred without Policy", async function () {
        let cycleBalance = await manager.canisterBalance()
        await manager.execute()
        let trRequestResponse = await requestTopUpQuorumTransaction(manager, walletUid, 100000)
        let tr = trRequestResponse[0] as TopUpTransaction
        await manager.execute()
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TopUpTransaction
        let expectedTrs = buildExpectedTopUpTransaction(TransactionState.Executed)
        verifyTopUpTransaction(expectedTrs, tr)
        //need to wait for cycles minter to mint cycles
        await sleep(20);
        let cycleBalance2 = await manager.canisterBalance()
        expect(cycleBalance2 > cycleBalance).eq(true)
    });

    it("Trs Rejected without Policy", async function () {
        await manager.execute()
        let trRequestResponse = await requestTopUpQuorumTransaction(manager, walletUid, 999999999)
        let tr = trRequestResponse[0] as TopUpTransaction
        await manager.execute()
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TopUpTransaction
        expect(tr.state).eq(TransactionState.Failed)
        expect(JSON.stringify(tr.error)).contains("ledger transfer error: InsufficientFunds")
    });

    it("Trs Rejected with Policy", async function () {
        let wallet = await requestCreateWalletTransaction(manager, "walletUid", Network.IC)
        await manager.execute()
        let uid = (wallet[0] as WalletCreateTransaction).uid
        await requestCreatePolicyTransaction(manager, 2, 100000, [uid])
        await manager.execute()
        let trs = await manager.getTransactions()
        let trRequestResponse = await requestTopUpTransaction(manager, uid, 999999999)
        let tr = trRequestResponse[0] as TopUpTransaction
        let approve: ApproveRequest = {
            trId: tr.id,
            state: TransactionState.Approved
        }
        await member_manager.approveTransaction([approve])
        await manager.execute()
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TopUpTransaction
        expect(tr.state).eq(TransactionState.Failed)
        expect(tr.threshold).eq(2)
        expect(JSON.stringify(tr.error)).contains("ledger transfer error: InsufficientFunds")
    });


    function buildExpectedTopUpTransaction(state: TransactionState): TopUpTransaction {
        let expectedApprove: Approve = {
            createdDate: 0n,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }
        let expectedTrs: TopUpTransaction = {
            amount: 100000n,
            currency: Currency.ICP,
            wallet: walletUid,
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: 0n,
            id: 0n,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: false,
            modifiedDate: 0n,
            state,
            transactionType: TransactionType.Transfer
        }
        return expectedTrs
    }

})

function verifyTopUpTransaction(expected: TopUpTransaction, actual: TopUpTransaction) {
    expect(expected.wallet).eq(actual.wallet)
    expect(expected.amount).eq(actual.amount)
    expect(expected.currency).eq(actual.currency)
    verifyTransaction(expected, actual, TransactionType.Transfer)
}
