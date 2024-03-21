import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {ActorMethod} from "@dfinity/agent";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {expect} from "chai";
import {
    getTransactionByIdFromGetAllTrs,
    requestCreateWalletTransaction,
    requestUpdateWalletNameTransaction,
    verifyTransaction
} from "./helper";
import {Approve, Network, TransactionState, TransactionType, VaultManager} from "./sdk";
import {WalletCreateTransaction, WalletCreateTransactionRequest} from "./sdk/transaction/wallet/wallet_create";
import {WalletUpdateNameTransaction} from "./sdk/transaction/wallet/wallet_update_name";
import { hasOwnProperty } from "./sdk/util/helper";

require('./bigintextension.js');

describe("Wallet Transactions", () => {
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
        manager = new VaultManager(canister_id, admin_identity);
        await manager.resetToLocalEnv();
    });

    after(() => {
        DFX.STOP();
    });

    const walletName = "testWallet"
    const walletName_2 = "testWallet2"
    let uid;
    it("CreateWallet approved and executed", async function () {
        let trRequestResponse = await requestCreateWalletTransaction(manager, walletName, Network.IC);
        let expected = buildExpectedWalletCreateTransaction(TransactionState.Approved)
        verifyWalletCreateTransaction(expected, trRequestResponse[0] as WalletCreateTransaction)
        let trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        verifyWalletCreateTransaction(expected, trFromAll);
        await manager.execute()
        trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        expected = buildExpectedWalletCreateTransaction(TransactionState.Executed)
        verifyWalletCreateTransaction(expected, trFromAll)

        let state = await manager.getState()
        let walletActual = state.wallets.find(w => w.name === walletName)
        uid = walletActual.uid;
        expect(walletActual.uid).not.undefined
        expect(walletActual.network).eq(Network.IC)
    });

    it("UpdateWalletName approved executed", async function () {
        let trRequestResponse = await requestUpdateWalletNameTransaction(manager, uid, walletName_2);
        let expected = buildExpectedWalletCreateTransaction(TransactionState.Approved)
        expected.name = walletName_2
        verifyWalletUpdateTransaction(expected, trRequestResponse[0] as WalletCreateTransaction)
        let trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        verifyWalletUpdateTransaction(expected, trFromAll);
        await manager.execute()
        trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        expected = buildExpectedWalletCreateTransaction(TransactionState.Executed)
        expected.name = walletName_2
        verifyWalletUpdateTransaction(expected, trFromAll)

        let state = await manager.getState()
        let walletActual = state.wallets.find(w => w.uid === uid)
        expect(walletActual.name).eq(walletName_2)
    });

    it("UpdateWalletName  rejected no such wallet", async function () {
        let trRequestResponse = await requestUpdateWalletNameTransaction(manager, "uidUnexistene", walletName);
        await manager.execute()
        let trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        let expected = buildExpectedWalletCreateTransaction(TransactionState.Rejected)
        verifyWalletUpdateTransaction(expected, trFromAll)
    });


    it("Request 2 Wallets With The Same Uid rejected", async function () {
        let request1 = new WalletCreateTransactionRequest("uniqueId", "11112123123", Network.IC);
        let response1 = await manager.requestTransaction([request1])
        await manager.execute()
        let transaction1 = await getTransactionByIdFromGetAllTrs(manager, response1[0].id) as WalletCreateTransaction
        expect(transaction1.state).eq(TransactionState.Executed)
        let request2 = new WalletCreateTransactionRequest("uniqueId", "345346456", Network.IC);
        let response2 = await manager.requestTransaction([request2])
        await manager.execute()
        let transaction2 = await getTransactionByIdFromGetAllTrs(manager, response2[0].id) as WalletCreateTransaction
        expect(transaction2.state).eq(TransactionState.Rejected)
        expect(hasOwnProperty(transaction2.error, "UIDAlreadyExists")).eq(true)
    });


    function buildExpectedWalletCreateTransaction(state) {
        let expectedApprove: Approve = {
            createdDate: 0n,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }

        let expectedTrs: WalletCreateTransaction = {
            modifiedDate: 0n,
            uid: "",
            name: walletName, network: Network.IC,
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: 0n,
            id: 0n,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            state,
            transactionType: TransactionType.WalletCreate
        }
        return expectedTrs
    }

})

export function verifyWalletCreateTransaction(expected: WalletCreateTransaction, actual: WalletCreateTransaction) {
    expect(expected.name).eq(actual.name)
    expect(expected.network).eq(actual.network)
    expect(actual.uid).not.eq("")
    expect(actual.uid).not.eq(undefined)
    verifyTransaction(expected, actual, TransactionType.WalletCreate)
}


export function verifyWalletUpdateTransaction(expected: WalletUpdateNameTransaction, actual: WalletUpdateNameTransaction) {
    expect(expected.name).eq(actual.name)
    expect(actual.uid).eq(actual.uid)
    verifyTransaction(expected, actual, TransactionType.WalletCreate)
}