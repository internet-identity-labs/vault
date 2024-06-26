import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {execute, sleep} from "../util/call.util";
import {expect} from "chai";
import {fromHexString, principalToAddress, principalToAddressBytes} from "ictool";
import {Principal} from "@dfinity/principal";
import {
    getTransactionByIdFromGetAllTrs,
    requestCreateMemberTransaction,
    requestCreatePolicyTransaction,
    requestCreateWalletTransaction,
    requestICRC1TransferTransaction,
    requestPurgeTransaction,
    requestQuorumTransferTransaction,
    requestTransferTransaction,
    requestUpdateQuorumTransaction,
    verifyTransaction
} from "./helper";
import {
    Approve,
    ApproveRequest,
    Currency,
    hasOwnProperty,
    Network,
    TransactionState,
    TransactionType,
    TransferQuorumTransaction,
    TransferTransaction,
    VaultManager,
    VaultRole,
    WalletCreateTransaction,
    WalletCreateTransactionRequest
} from "@nfid/vaults";

require('./bigintextension.js');
const DEFAULT_SUB_ACCOUNT =
    "0000000000000000000000000000000000000000000000000000000000000000"
describe("Transfer Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let member_identity = getIdentity("87654321876543218765432187654322")
    let manager: VaultManager;
    let manager_member: VaultManager;
    let address = principalToAddress(admin_identity.getPrincipal() as any)
    let member_address = principalToAddress(member_identity.getPrincipal() as any).toUpperCase()
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager(canister_id, admin_identity);
        manager_member = new VaultManager(canister_id, member_identity);
        await manager.resetToLocalEnv();
        await manager_member.resetToLocalEnv();
    });

    after(() => {
        DFX.STOP();
    });

    let walletUId;
    it("Trs approved and transferred", async function () {
        let createWallet = await requestCreateWalletTransaction(manager, "testWallet", Network.IC) as Array<WalletCreateTransaction>
        walletUId = createWallet[0].uid
        let walBytes = principalToAddressBytes(Principal.fromText(canister_id) as any, fromHexString(walletUId))
        await console.log(DFX.LEDGER_FILL_BALANCE(walBytes.toString().replaceAll(',', ';')))
        let createPolicy = await requestCreatePolicyTransaction(manager, 1, 10, [createWallet[0].uid])
        await manager.execute()
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId, 100)
        let tr = trRequestResponse[0] as TransferTransaction
        let expected = buildExpectedTransferTransaction(TransactionState.Approved)
        verifyTransferTransaction(expected, trRequestResponse[0] as TransferTransaction)
        await sleep(5)
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TransferTransaction
        expected = buildExpectedTransferTransaction(TransactionState.Executed)
        verifyTransferTransaction(expected, tr)
        expect(tr.blockIndex).eq(2n)
    });

    it("Trs approved and rejected - wallet not hex", async function () {
        let memberR = new WalletCreateTransactionRequest("uniqueId", "walletName_incorrect_UID", Network.IC);
        let createWallet = await manager.requestTransaction([memberR]) as WalletCreateTransaction[]
        await requestCreatePolicyTransaction(manager, 1, 10, [createWallet[0].uid])
        await manager.execute()
        let trRequestResponse = await requestTransferTransaction(manager, address, createWallet[0].uid, 100)
        let tr = trRequestResponse[0] as TransferTransaction
        await sleep(5)
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TransferTransaction
        expect(tr.state).eq(TransactionState.Failed)
    });

    it("Trs approved and failed ", async function () {
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId, 300_000_000)
        let tr = trRequestResponse[0] as TransferTransaction
        let expected = buildExpectedTransferTransaction(TransactionState.Approved)
        expected.amount = 300_000_000n
        verifyTransferTransaction(expected, trRequestResponse[0] as TransferTransaction)
        await manager.execute()
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TransferTransaction
        expected.state = TransactionState.Failed
        verifyTransferTransaction(expected, tr)
        // @ts-ignore
        expect(tr.error.CanisterReject.message).eq("ledger transfer error: InsufficientFunds { balance: Tokens { e8s: 99989900 } }")
    });

    it("Trs approved and executed from member", async function () {
        await requestCreateMemberTransaction(manager, member_address, "memberName", VaultRole.MEMBER)
        await manager.execute()
        let trRequestResponse = await requestTransferTransaction(manager_member, address, walletUId, 100)
        let tr = trRequestResponse[0] as TransferTransaction
        let expected = buildExpectedTransferTransaction(TransactionState.Approved)
        expected.initiator = member_address
        expected.approves[0].signer = member_address
        verifyTransferTransaction(expected, trRequestResponse[0] as TransferTransaction)
        await sleep(5)
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TransferTransaction
        expected.state = TransactionState.Executed
        verifyTransferTransaction(expected, tr)
    });

    it("Trs approved and failed from member", async function () {
        await requestCreateMemberTransaction(manager, member_address, "memberName", VaultRole.MEMBER)
        await manager.execute()
        let trRequestResponse = await requestTransferTransaction(manager_member, "address", walletUId, 100)
        let tr = trRequestResponse[0] as TransferTransaction
        let expected = buildExpectedTransferTransaction(TransactionState.Approved)
        expected.initiator = member_address
        expected.approves[0].signer = member_address
        await sleep(5)
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TransferTransaction
        expected.state = TransactionState.Failed
        expect(hasOwnProperty(tr.error, "CanisterReject")).eq(true)
    });

    it("Trs approved and failed by system ", async function () {
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId, 10)
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        let expected = buildExpectedTransferTransaction(TransactionState.Failed)
        expected.amount = 10n
        expected.threshold = undefined
        expect(hasOwnProperty(tr.error, "CouldNotDefinePolicy")).eq(true)
        verifyTransferTransaction(expected, tr as TransferTransaction)
    });

    it("Trs pending and correct threshold from the highest policy", async function () {
        await requestCreatePolicyTransaction(manager, 2, 100, [walletUId])
        await manager.execute()
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId, 150)
        let tr = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        let expected = buildExpectedTransferTransaction(TransactionState.Pending)
        expected.amount = 150n
        expected.threshold = 2
        verifyTransferTransaction(expected, tr as TransferTransaction)
    });

    it("Trs blocked and correct threshold from the highest policy", async function () {
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId, 200)
        let tr = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        let expected = buildExpectedTransferTransaction(TransactionState.Blocked)
        expected.amount = 200n
        expected.threshold = 2
        verifyTransferTransaction(expected, tr as TransferTransaction)
    });

    it("Trs pending because from another wallet", async function () {
        let trs = await manager.getTransactions()
        let createWallet = await requestCreateWalletTransaction(manager, "testWallet2", Network.IC) as Array<WalletCreateTransaction>
        let walletTransaction = await getTransactionByIdFromGetAllTrs(manager, createWallet[0].id)
        //have to be blocked by previous transfer transactions
        expect(walletTransaction.state).eq(TransactionState.Blocked)
        let reject1: ApproveRequest = {
            trId:  trs[trs.length - 2].id,
            state: TransactionState.Rejected
        }
        let reject2: ApproveRequest = {
            trId:  trs[trs.length - 1].id,
            state: TransactionState.Rejected
        }
        await manager_member.approveTransaction([reject1, reject2])
        await manager_member.execute()
        let walletUId2 = createWallet[0].uid
        await requestCreatePolicyTransaction(manager, 2, 10, [walletUId2])
        await manager.execute()
        //request transfer from existing wallet
        await requestTransferTransaction(manager, address, walletUId, 200)
        await manager.execute()
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId2, 100)
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id) as TransferTransaction
        expect(tr.state).eq(TransactionState.Pending)
    });

    let walletUId4
    it("Request quorum policy transfer transaction", async function () {
        await requestPurgeTransaction(manager)
        await manager.execute()
        let createWallet = await requestCreateWalletTransaction(manager, "testWallet5", Network.IC) as Array<WalletCreateTransaction>
        walletUId4 = createWallet[0].uid
        await manager.execute()
        let trRequestResponse = await requestQuorumTransferTransaction(manager, address, walletUId4, 100)
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id) as TransferQuorumTransaction
        expect(tr.state).eq(TransactionState.Failed)
        expect(tr.threshold).eq(1)
        // @ts-ignore
        expect(tr.error.CanisterReject.message).eq("ledger transfer error: InsufficientFunds { balance: Tokens { e8s: 0 } }")
    });

    it("Request quorum ICRC1 transfer transaction failed", async function () {
        await requestPurgeTransaction(manager)
        await manager.execute()
        let trRequestResponse = await requestICRC1TransferTransaction(manager,
            Principal.fromText("sculj-2sjuf-dxqlm-dcv5y-hin5x-zfyvr-tzngf-bt5b5-dwhcc-zbsqf-rae"),
            undefined,
            Principal.fromText("ryjl3-tyaaa-aaaaa-aaaba-cai"),
            walletUId4,
            100n,
            "memo")
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id) as TransferQuorumTransaction
        expect(tr.state).eq(TransactionState.Failed)
        expect(tr.threshold).eq(1)
        // @ts-ignore
        expect(tr.error.CanisterReject.message).eq("the debit account doesn't have enough funds to complete the transaction, current balance: 0")
    });

    it("Request quorum ICRC1 transfer transaction Executed + default wallet", async function () {
        await requestPurgeTransaction(manager)
        await manager.execute()
        await manager.execute()
        let memberR = new WalletCreateTransactionRequest(DEFAULT_SUB_ACCOUNT, "defaultWallet", Network.IC);
        let walBytes = principalToAddressBytes(Principal.fromText(canister_id) as any, fromHexString(DEFAULT_SUB_ACCOUNT))
        await console.log(DFX.LEDGER_FILL_BALANCE(walBytes.toString().replaceAll(',', ';')))
        await manager.requestTransaction([memberR])
        let trRequestResponse = await requestICRC1TransferTransaction(manager,
            Principal.fromText("sculj-2sjuf-dxqlm-dcv5y-hin5x-zfyvr-tzngf-bt5b5-dwhcc-zbsqf-rae"),
            undefined,
            Principal.fromText("ryjl3-tyaaa-aaaaa-aaaba-cai"),
            DEFAULT_SUB_ACCOUNT,
            100n,
            "memo")
        await sleep(5)
        let tr = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id) as TransferQuorumTransaction
        expect(tr.state).eq(TransactionState.Executed)
        expect(tr.threshold).eq(1)
        expect(tr.blockIndex).eq(5n)
    });

    it("Trs blocked because of vault trs", async function () {
        let createWallet = await requestCreateWalletTransaction(manager, "testWallet3", Network.IC) as Array<WalletCreateTransaction>
        let walletUId = createWallet[0].uid
        await requestCreatePolicyTransaction(manager, 2, 10, [walletUId])
        await manager.execute()
        await requestCreateMemberTransaction(manager, "member_address", "memberName", VaultRole.ADMIN)
        await requestUpdateQuorumTransaction(manager, 2)
        await manager.execute()
        await requestCreateMemberTransaction(manager, "member_address2", "memberName", VaultRole.ADMIN)
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId, 100)
        expect(trRequestResponse[0].state).eq(TransactionState.Blocked)
    });

    function buildExpectedTransferTransaction(state: TransactionState): TransferTransaction {
        let expectedApprove: Approve = {
            createdDate: 0n,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }
        let expectedTrs: TransferTransaction = {
            policy: undefined,
            address: address,
            amount: 100n,
            currency: Currency.ICP,
            wallet: walletUId,
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: 0n,
            id: 0n,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: false,
            modifiedDate: 0n,
            state,
            transactionType: TransactionType.Transfer,
            blockIndex: undefined
        }
        return expectedTrs
    }

})

function verifyTransferTransaction(expected: TransferTransaction, actual: TransferTransaction) {
    expect(expected.wallet).eq(actual.wallet)
    expect(expected.amount).eq(actual.amount)
    expect(expected.address).eq(actual.address)
    expect(expected.currency).eq(actual.currency)
    verifyTransaction(expected, actual, TransactionType.Transfer)
}
