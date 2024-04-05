import {getIdentity} from "../util/deployment.util";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {getTransactionByIdFromGetAllTrs, requestTopUpTransaction, verifyTransaction} from "./helper";
import {sleep} from "../util/call.util";
import {Approve, Currency, TransactionState, TransactionType, VaultManager} from "@nfid/vaults";
import {TopUpTransaction} from "@nfid/vaults";

require('./bigintextension.js');

describe.skip("TopUp Transactions", () => {
    //predefined wallet address to fill with ICP manually
    const walletAddress = "706ab8c2d9585942dc4bdc5ed73188d7f56f97374a36b63b08ca45456ae699e3";
    const walletUid = "ba7f3c8953f15ae2e66f8def7d3a7c388e5af7f35e9c74f7d95aa3faa4b20c22";
    let canisterId = "hygn6-wiaaa-aaaal-qcr7a-cai";
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    before(async () => {
        manager = new VaultManager(canisterId, admin_identity);
        await manager.resetToLocalEnv();
    });

    it("Trs approved and transferred", async function () {
        let cycleBalance = await manager.canisterBalance()
        await manager.execute()
        let trRequestResponse = await requestTopUpTransaction(manager, walletUid, 100000)
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

    it("Trs Rejected", async function () {
        await manager.execute()
        let trRequestResponse = await requestTopUpTransaction(manager, walletUid, 999999999)
        let tr = trRequestResponse[0] as TopUpTransaction
        await manager.execute()
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TopUpTransaction
        expect(tr.state).eq(TransactionState.Rejected)
        expect(tr.memo).contains("ledger transfer error: InsufficientFunds")
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
