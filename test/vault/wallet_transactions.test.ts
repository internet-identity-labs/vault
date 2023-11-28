import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {idlFactory} from "./sdk_prototype/idl";
import {ActorMethod} from "@dfinity/agent";
import {
    Approve,
    Network,
    Transaction,
    TransactionState,
    TransactionType,
    VaultManager,
    WalletCreateTransaction,
    WalletCreateTransactionRequest,
    WalletUpdateNameTransaction,
    WalletUpdateNameTransactionRequest
} from "./sdk_prototype/vault_manager";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {getTransactionByIdFromGetAllTrs, verifyTransaction} from "./member_transactions.test";
import {expect} from "chai";

require('./bigintExtension.js');

describe("Wallet Transactions", () => {
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
    let uid;
    it("CreateWallet approved and executed", async function () {
        let trRequestResponse = await requestCreateWalletTransaction(manager, walletName, Network.IC);
        let expected = buildExpectedWalletCreateTransaction(trRequestResponse[0], TransactionState.Approved)
        verifyWalletCreateTransaction(expected, trRequestResponse[0] as WalletCreateTransaction)
        let trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        verifyWalletCreateTransaction(expected, trFromAll);
        await manager.execute()
        trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        expected = buildExpectedWalletCreateTransaction(trRequestResponse[0], TransactionState.Executed)
        verifyWalletCreateTransaction(expected, trFromAll)

        let state = await manager.redefineState()
        let walletActual = state.wallets.find(w => w.name === walletName)
        uid = walletActual.uid;
        expect(walletActual.uid).not.undefined
        expect(walletActual.network).eq(Network.IC)
    });

    it("UpdateWalletName approved executed", async function () {
        let trRequestResponse = await requestUpdateWalletNameTransaction(manager, uid, walletName_2);
        let expected = buildExpectedWalletCreateTransaction(trRequestResponse[0], TransactionState.Approved)
        expected.name = walletName_2
        verifyWalletUpdateTransaction(expected, trRequestResponse[0] as WalletCreateTransaction)
        let trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        verifyWalletUpdateTransaction(expected, trFromAll);
        await manager.execute()
        trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        expected = buildExpectedWalletCreateTransaction(trRequestResponse[0], TransactionState.Executed)
        expected.name = walletName_2
        verifyWalletUpdateTransaction(expected, trFromAll)

        let state = await manager.redefineState()
        let walletActual = state.wallets.find(w => w.uid === uid)
        expect(walletActual.name).eq(walletName_2)
    });

    it("UpdateWalletName  rejected no such wallet", async function () {
        let trRequestResponse = await requestUpdateWalletNameTransaction(manager, "uidUnexistene", walletName);
        await manager.execute()
        let trFromAll = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        let expected = buildExpectedWalletCreateTransaction(trRequestResponse[0], TransactionState.Rejected)
        verifyWalletUpdateTransaction(expected, trFromAll)
    });


    function buildExpectedWalletCreateTransaction(actualTr, state) {
        let expectedApprove: Approve = {
            createdDate: actualTr.approves[0].createdDate,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }

        let expectedTrs: WalletCreateTransaction = {
            modifiedDate: actualTr.modifiedDate,
            uid: actualTr.uid,
            name: walletName, network: Network.IC,
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: actualTr.createdDate,
            id: actualTr.id,
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

export async function requestUpdateWalletNameTransaction(manager, uid, walletName): Promise<Array<Transaction>> {
    let memberR = new WalletUpdateNameTransactionRequest(walletName, uid);
    return await manager.requestTransaction([memberR])
}

export async function requestCreateWalletTransaction(manager, walletName, network): Promise<Array<Transaction>> {
    let memberR = new WalletCreateTransactionRequest(walletName, network);
    return await manager.requestTransaction([memberR])
}
