import {
    MemberCreateTransactionRequest,
    MemberRemoveTransactionRequest,
    MemberUpdateNameTransactionRequest,
    MemberUpdateRoleTransactionRequest,
    PolicyCreateTransactionRequest,
    PolicyRemoveTransactionRequest,
    PolicyUpdateTransactionRequest, PurgeTransactionRequest,
    QuorumTransactionRequest,
    TopUpTransactionRequest,
    TransferTransactionRequest,
    VaultNamingTransactionRequest,
    VersionUpgradeTransactionRequest,
    WalletCreateTransactionRequest,
    WalletUpdateNameTransactionRequest
} from "./sdk_prototype/transaction_requests";
import {expect} from "chai";
import {Transaction} from "./sdk_prototype/transactions";
import {Currency, TransactionState} from "./sdk_prototype/enums";
import {Approve} from "./sdk_prototype/approve";
import {generateRandomString} from "./sdk_prototype/helper";


export function verifyTransaction(expected: Transaction, actual: Transaction, trType) {
    expect(expected.state).eq(actual.state)
    expect(expected.batchUid).eq(actual.batchUid)
    expect(expected.initiator).eq(actual.initiator)
    expect(expected.isVaultState).eq(actual.isVaultState)
    expect(expected.transactionType).eq(trType)
    expect(expected.memo).eq(actual.memo)
    expect(expected.approves.length).eq(actual.approves.length)
    if (expected.state === TransactionState.Blocked) {
        expect(undefined).eq(actual.threshold)
    } else {
        expect(expected.threshold).eq(actual.threshold)
    }
    for (const app of expected.approves) {
        const found = actual.approves.find((l) => l.signer === app.signer);
        verifyApprove(app, found)
    }
}

export async function requestCreatePolicyTransaction(manager, membersTr, amountTr, wallets): Promise<Array<Transaction>> {
    const uniqueId: string = generateRandomString();
    let transactionRequest = new PolicyCreateTransactionRequest(uniqueId, membersTr, amountTr, wallets);
    return await manager.requestTransaction([transactionRequest])
}

export async function requestRemovePolicyTransaction(manager, uid): Promise<Array<Transaction>> {
    let transactionRequest = new PolicyRemoveTransactionRequest(uid);
    return await manager.requestTransaction([transactionRequest])
}

export async function requestUpdatePolicyTransaction(manager, membersTr, amountTr, uid): Promise<Array<Transaction>> {
    let transactionRequest = new PolicyUpdateTransactionRequest(uid, membersTr, amountTr);
    return await manager.requestTransaction([transactionRequest])
}

export async function requestCreateMemberTransaction(manager, memberAddress, memberName, memberRole): Promise<Array<Transaction>> {
    let transactionRequest = new MemberCreateTransactionRequest(memberAddress, memberName, memberRole);
    return await manager.requestTransaction([transactionRequest])
}

export async function requestUpdateMemberNameTransaction(manager, memberAddress, memberName): Promise<Array<Transaction>> {
    let transactionRequest = new MemberUpdateNameTransactionRequest(memberAddress, memberName);
    return await manager.requestTransaction([transactionRequest])
}

export async function requestUpdateMemberRoleTransaction(manager, memberAddress, transactionRequestole): Promise<Array<Transaction>> {
    let transactionRequest = new MemberUpdateRoleTransactionRequest(memberAddress, transactionRequestole);
    return await manager.requestTransaction([transactionRequest])
}

export async function requestRemoveMemberTransaction(manager, memberAddress): Promise<Array<Transaction>> {
    let transactionRequest = new MemberRemoveTransactionRequest(memberAddress);
    return await manager.requestTransaction([transactionRequest])
}

export async function getTransactionByIdFromGetAllTrs(manager, trId) {
    let transactions = await manager.getTransactions();
    let tr = transactions.find(l => l.id === trId);
    return tr;
}

export async function requestUpdateQuorumTransaction(manager, quorum): Promise<Array<Transaction>> {
    let memberR = new QuorumTransactionRequest(quorum);
    return await manager.requestTransaction([memberR])
}

export async function requestPurgeTransaction(manager): Promise<Array<Transaction>> {
    let memberR = new PurgeTransactionRequest();
    return await manager.requestTransaction([memberR])
}

export async function requestUpdateVaultNamingTransaction(manager, name?, description?): Promise<Array<Transaction>> {
    let memberR = new VaultNamingTransactionRequest(name, description);
    return await manager.requestTransaction([memberR])
}


export async function requestUpdateWalletNameTransaction(manager, uid, walletName): Promise<Array<Transaction>> {
    let memberR = new WalletUpdateNameTransactionRequest(walletName, uid);
    return await manager.requestTransaction([memberR])
}

export async function requestCreateWalletTransaction(manager, walletName, network): Promise<Array<Transaction>> {
    const uniqueId: string = generateRandomString();
    let memberR = new WalletCreateTransactionRequest(uniqueId, walletName, network);
    return await manager.requestTransaction([memberR])
}

export async function requestVersionUpgradeTransaction(manager, version): Promise<Array<Transaction>> {
    let memberR = new VersionUpgradeTransactionRequest(version);
    return await manager.requestTransaction([memberR])
}

export async function requestTransferTransaction(manager, address, wallet, amount): Promise<Array<Transaction>> {
    let memberR = new TransferTransactionRequest(Currency.ICP, address, wallet, amount);
    return await manager.requestTransaction([memberR])
}

export async function requestTopUpTransaction(manager, wallet, amount): Promise<Array<Transaction>> {
    let memberR = new TopUpTransactionRequest(Currency.ICP, wallet, amount);
    return await manager.requestTransaction([memberR])
}

export function verifyApprove(expected: Approve, actual: Approve) {
    expect(expected.status).eq(actual.status)
    expect(expected.signer).eq(actual.signer)
}
