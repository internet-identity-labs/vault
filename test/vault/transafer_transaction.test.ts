import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {
    Approve,
    Currency,
    Network,
    TransactionState,
    TransactionType,
    TransferTransaction,
    VaultManager,
    VaultRole,
    WalletCreateTransaction
} from "./sdk_prototype/vault_manager";
import {execute} from "../util/call.util";
import {expect} from "chai";
import {fromHexString, principalToAddress, principalToAddressBytes} from "ictool";
import {Principal} from "@dfinity/principal";
import {
    getTransactionByIdFromGetAllTrs,
    requestCreateMemberTransaction,
    requestCreatePolicyTransaction,
    requestCreateWalletTransaction,
    requestTransferTransaction, requestUpdateQuorumTransaction,
    verifyTransaction
} from "./helper";

require('./bigintextension.js');

describe("Transfer Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let member_identity = getIdentity("87654321876543218765432187654322")
    let manager: VaultManager;
    let manager_member: VaultManager;
    let address = principalToAddress(admin_identity.getPrincipal() as any)
    let member_address = principalToAddress(member_identity.getPrincipal() as any)
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager();
        manager_member = new VaultManager();
        await manager.init(canister_id, admin_identity, true);
        await manager_member.init(canister_id, member_identity, true);
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
        await manager.execute()
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TransferTransaction
        expected = buildExpectedTransferTransaction(TransactionState.Executed)
        verifyTransferTransaction(expected, tr)
    });

    it("Trs approved and rejected by system", async function () {
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId, 300_000_000)
        let tr = trRequestResponse[0] as TransferTransaction
        let expected = buildExpectedTransferTransaction(TransactionState.Approved)
        expected.amount = 300_000_000n
        verifyTransferTransaction(expected, trRequestResponse[0] as TransferTransaction)
        await manager.execute()
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TransferTransaction
        expected.state = TransactionState.Rejected
        expected.memo = "ledger transfer error: InsufficientFunds { balance: Tokens { e8s: 199989900 } }"
        verifyTransferTransaction(expected, tr)
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
        await manager.execute()
        tr = await getTransactionByIdFromGetAllTrs(manager, tr.id) as TransferTransaction
        expected.state = TransactionState.Executed
        verifyTransferTransaction(expected, tr)
    });

    it("Trs approved and rejected by system", async function () {
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId, 10)
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id)
        let expected = buildExpectedTransferTransaction(TransactionState.Rejected)
        expected.amount = 10n
        expected.threshold = undefined
        expected.memo = "No suitable policy"
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
        let trs = await manager.getTransactions()
        expected.amount = 200n
        expected.threshold = 2
        verifyTransferTransaction(expected, tr as TransferTransaction)
    });

    it("Trs pending because from another wallet", async function () {
        let createWallet = await requestCreateWalletTransaction(manager, "testWallet2", Network.IC) as Array<WalletCreateTransaction>
        let walletUId2 = createWallet[0].uid
        await requestCreatePolicyTransaction(manager, 2, 10, [walletUId2])
        await manager.execute()
        let trRequestResponse = await requestTransferTransaction(manager, address, walletUId2, 100)
        await manager.execute()
        let tr = await getTransactionByIdFromGetAllTrs(manager, trRequestResponse[0].id) as TransferTransaction
        expect(tr.state).eq(TransactionState.Pending)
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
            isVaultState: true,
            modifiedDate: 0n,
            state,
            transactionType: TransactionType.Transfer
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
