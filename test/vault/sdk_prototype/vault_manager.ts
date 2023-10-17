import {
    Approve as ApproveCandid,
    Member,
    MemberCreateTransaction as MemberCreateTransactionCandid,
    MemberRemoveTransaction as MemberRemoveTransactionCandid,
    MemberUpdateNameTransaction as MemberUpdateNameTransactionCandid,
    MemberUpdateRoleTransaction as MemberUpdateRoleTransactionCandid,
    Network as NetworkCandid,
    ObjectState as ObjectStateCandid,
    Policy as PolicyCandid,
    PolicyCreateTransaction as PolicyCreateTransactionCandid,
    PolicyRemoveTransaction as PolicyRemoveTransactionCandid,
    PolicyUpdateTransaction as PolicyUpdateTransactionCandid,
    QuorumUpdateTransaction as QuorumUpdateTransactionCandid,
    TopUpTransaction as TopUpTransactionCandid,
    TransactionApproveRequest,
    TransactionCandid,
    TransactionRequest as TransactionRequestCandid,
    TransactionState as TransactionStateCandid,
    TransferTransaction as TransferTransactionCandid,
    TrType,
    VaultNamingUpdateTransaction as VaultNamingUpdateTransactionCandid,
    VaultRole as VaultRoleCandid,
    VaultState,
    VersionUpgradeTransaction as VersionUpgradeCandid,
    Wallet as WalletCandid,
    WalletCreateTransaction as WalletCreateTransactionCandid,
    WalletUpdateNameTransaction as WalletUpdateNameTransactionCandid
} from "./service_vault";
import {idlFactory} from "./idl";

import {Actor, ActorMethod, HttpAgent, Identity} from "@dfinity/agent";
import {IDL} from "@dfinity/candid";

export interface VaultManagerI {
    getTransactions(): Promise<Array<Transaction>>;

    getState(): Vault

    redefineState(id?: BigInt): Promise<Vault>

    requestTransaction(requests: Array<TransactionRequest>): Promise<Array<Transaction>>

    approveTransaction(approve: Array<ApproveRequest>): Promise<Array<Transaction>>

    execute();

    canisterBalance(): Promise<bigint>;
}


export class VaultManager implements VaultManagerI {
    actor: Record<string, ActorMethod>;
    state: Vault;
    canisterId: String;

    public async init(canisterId: string, identity: Identity, isLocalNetwork?: boolean): Promise<VaultManager> {
        this.actor = await this.getActor(canisterId, identity, idlFactory, isLocalNetwork);
        this.canisterId = canisterId;
        let state = await this.actor.get_state([]) as VaultState;
        this.state = vaultCandidToVault(state)
        return this;
    }

    async getTransactions(): Promise<Array<Transaction>> {
        let trs = await this.actor.get_transactions_all() as Array<TransactionCandid>
        let res: Array<Transaction> = trs.map(transactionCandidToTransaction);
        return res
    }

    async getVersion(): Promise<string> {
        return await this.actor.get_version() as string
    }

    async requestTransaction(request: Array<TransactionRequest>): Promise<Array<Transaction>> {
        let b: Array<TransactionRequestCandid> = request.map(l => l.toCandid())
        let a = await this.actor.request_transaction(b) as [TransactionCandid];
        return a.map(transactionCandidToTransaction)
    }

    getState(): Vault {
        return this.state
    }

    async canisterBalance(): Promise<bigint> {
        return await this.actor.canister_balance() as bigint
    }

    async redefineState(id?: BigInt): Promise<Vault> {
        let param = id === undefined ? [] : [id]
        let state = await this.actor.get_state(param) as VaultState;
        this.state = vaultCandidToVault(state)
        return this.state
    }

    async approveTransaction(approves: Array<ApproveRequest>): Promise<Array<Transaction>> {
        let a: Array<TransactionApproveRequest> = approves.map(approveToCandid)
        let b = await this.actor.approve(a) as Array<TransactionCandid>;
        let xx: Array<Transaction> = b.map(transactionCandidToTransaction)
        return xx
    }

    private getActor = async (
        imCanisterId: string,
        identity: Identity,
        idl: IDL.InterfaceFactory,
        isTest?: boolean
    ): Promise<Record<string, ActorMethod>> => {
        let agent: HttpAgent;
        if (isTest) {
            agent = new HttpAgent({host: "http://127.0.0.1:8000", identity: identity});
            await agent.fetchRootKey();
        } else {
            agent = new HttpAgent({host: "https://ic0.app", identity: identity});
        }
        return Actor.createActor(idl, {agent, canisterId: imCanisterId});
    };

    async execute() {
        await this.actor.execute()
    }

}

export enum TransactionType {
    WalletUpdateName = 'WalletUpdateName',
    MemberCreate = 'MemberCreate',
    PolicyRemove = 'PolicyRemove',
    WalletCreate = 'WalletCreate',
    MemberRemove = 'MemberRemove',
    PolicyCreate = 'PolicyCreate',
    PolicyUpdate = 'PolicyUpdate',
    MemberUpdateName = 'MemberUpdateName',
    MemberUpdateRole = 'MemberUpdateRole',
    QuorumUpdate = 'QuorumUpdate',
    VaultNamingUpdate = 'VaultNamingUpdate',
    Transfer = 'Transfer',
    TopUp = 'TopUp',
    VersionUpgrade = 'VersionUpgrade',
}

export enum Currency {
    ICP = "ICP",
}

export enum Network {
    IC = "IC",
}

export enum TransactionState {
    Approved = "Approved",
    Pending = "Pending",
    Cancelled = "Cancelled",
    Rejected = "Rejected",
    Executed = "Executed",
    Blocked = "Blocked",
}

export enum ObjectState {
    ARCHIVED = "Archived",
    ACTIVE = "Active",
}


export enum VaultRole {
    ADMIN = "Admin",
    MEMBER = "Member",
}

export interface Quorum {
    quorum: number,
    modifiedDate: bigint
}

export interface VaultMember {
    userId: string
    name: string | undefined
    role: VaultRole
    state: ObjectState
    modifiedDate: bigint
    createdDate: bigint
}

export interface Wallet {
    'uid': string,
    'modifiedDate': bigint,
    'name': string,
    'network': Network,
    'createdDate': bigint,
}

export interface Policy {
    'uid': string,
    'member_threshold': number,
    'modified_date': bigint,
    'amount_threshold': bigint,
    'wallets': Array<string>,
    'currency': Currency,
    'created_date': bigint,
}

export class Vault {
    members: Array<VaultMember>
    quorum: Quorum
    wallets: Array<Wallet>
    policies: Array<Policy>
    name?: string
    description?: string
}


export interface Transaction {
    id: bigint;
    transactionType: TransactionType;
    initiator: string;
    modifiedDate: bigint;
    memo?: string;
    state: TransactionState;
    approves: Approve[];
    isVaultState: boolean;
    createdDate: bigint;
    batchUid: string;
    threshold: number | undefined
}

export interface MemberCreateTransaction extends Transaction {
    memberId: string;
    name: string;
    role: VaultRole;
}


export interface MemberUpdateNameTransaction extends Transaction {
    memberId: string;
    name: string;
}


export interface MemberUpdateRoleTransaction extends Transaction {
    memberId: string;
    role: VaultRole;
}

export interface MemberRemoveTransaction extends Transaction {
    memberId: string;
}

export interface QuorumUpdateTransaction extends Transaction {
    quorum: number
}

export interface VersionUpgradeTransaction extends Transaction {
    version: string
}

export interface VaultUpdateNamingTransaction extends Transaction {
    name?: string
    description?: string
}

export interface WalletCreateTransaction extends Transaction {
    name: string,
    network: Network,
    uid: string
}

export interface PolicyCreateTransaction extends Transaction {
    uid: string,
    member_threshold: number,
    amount_threshold: bigint,
    wallets: Array<string>,
    currency: Currency,
}

export interface PolicyUpdateTransaction extends Transaction {
    uid: string,
    member_threshold: number,
    amount_threshold: bigint,
}

export interface PolicyRemoveTransaction extends Transaction {
    uid: string,
}

export interface TransferTransaction extends Transaction {
    currency: Currency,
    address: string,
    wallet: string,
    amount: bigint,
    policy: string | undefined
}

export interface TopUpTransaction extends Transaction {
    currency: Currency,
    wallet: string,
    amount: bigint,
}

export interface WalletUpdateNameTransaction extends Transaction {
    name: string,
    uid: string
}

function vaultCandidToVault(vaultCandid: VaultState): Vault {
    let members: Array<VaultMember> = vaultCandid.members.map(mapMember)
    let quorum: Quorum = {
        modifiedDate: vaultCandid.quorum.modified_date,
        quorum: vaultCandid.quorum.quorum
    }
    let wallets: Array<Wallet> = vaultCandid.wallets.map(mapWallet)
    let policies: Array<Policy> = vaultCandid.policies.map(mapPolicy)
    let name = vaultCandid.name.length === 0 ? undefined : vaultCandid.name[0]
    let description = vaultCandid.description.length === 0 ? undefined : vaultCandid.description[0]
    let v: Vault = {
        members: members, quorum: quorum, wallets, policies, name, description
    }

    return v
}


function mapMember(mr: Member): VaultMember {
    return {
        createdDate: mr.created_date,
        modifiedDate: mr.modified_date,
        name: mr.name,
        role: candidToRole(mr.role),
        state: candidToObjectState(mr.state),
        userId: mr.member_id
    }
}


function mapWallet(mr: WalletCandid): Wallet {
    return {
        network: mapNetworkFromCandid(mr.network), uid: mr.uid,
        createdDate: mr.created_date,
        modifiedDate: mr.modified_date,
        name: mr.name
    }
}


function mapPolicy(mr: PolicyCandid): Policy {
    return {
        amount_threshold: mr.amount_threshold,
        created_date: mr.created_date,
        currency: Currency.ICP, //TODO
        member_threshold: mr.member_threshold,
        modified_date: mr.modified_date,
        uid: mr.uid,
        wallets: mr.wallets
    }
}


function candidToRole(response: VaultRoleCandid): VaultRole {
    if (hasOwnProperty(response, "Admin")) {
        return VaultRole.ADMIN
    }
    if (hasOwnProperty(response, "Member")) {
        return VaultRole.MEMBER
    }
    throw Error("Unexpected enum value")
}


function candidToObjectState(response: ObjectStateCandid): ObjectState {
    if (hasOwnProperty(response, "Active")) {
        return ObjectState.ACTIVE
    }
    if (hasOwnProperty(response, "Archived")) {
        return ObjectState.ARCHIVED
    }
    throw Error("Unexpected enum value")
}


function transactionCandidToTransaction(trs: TransactionCandid): Transaction {
    if (hasOwnProperty(trs, "QuorumUpdateTransactionV")) {
        let mmm = trs.QuorumUpdateTransactionV as QuorumUpdateTransactionCandid
        let t: QuorumUpdateTransaction = {
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            quorum: mmm.quorum,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
        }
        return t;
    }
    if (hasOwnProperty(trs, "UpgradeTransactionV")) {
        let mmm = trs.UpgradeTransactionV as VersionUpgradeCandid
        let t: VersionUpgradeTransaction = {
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            version: mmm.version,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
        }
        return t;
    }
    if (hasOwnProperty(trs, "VaultNamingUpdateTransactionV")) {
        let mmm = trs.VaultNamingUpdateTransactionV as VaultNamingUpdateTransactionCandid
        let t: VaultUpdateNamingTransaction = {
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            name: mmm.name.length === 0 ? undefined : mmm.name[0],
            description: mmm.description.length === 0 ? undefined : mmm.description[0],
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
        }
        return t;
    }
    if (hasOwnProperty(trs, "MemberCreateTransactionV")) {
        let mmm = trs.MemberCreateTransactionV as MemberCreateTransactionCandid
        let t: MemberCreateTransaction = {
            memberId: mmm.member_id,
            name: mmm.name,
            role: candidToRole(mmm.role),
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
        }
        return t;
    }
    if (hasOwnProperty(trs, "MemberUpdateNameTransactionV")) {
        let mmm = trs.MemberUpdateNameTransactionV as MemberUpdateNameTransactionCandid
        let t: MemberUpdateNameTransaction = {
            memberId: mmm.member_id,
            name: mmm.name,
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
        }
        return t;
    }
    if (hasOwnProperty(trs, "MemberUpdateRoleTransactionV")) {
        let mmm = trs.MemberUpdateRoleTransactionV as MemberUpdateRoleTransactionCandid
        let t: MemberUpdateRoleTransaction = {
            memberId: mmm.member_id,
            role: candidToRole(mmm.role),
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
        }
        return t;
    }
    if (hasOwnProperty(trs, "MemberRemoveTransactionV")) {
        let mmm = trs.MemberRemoveTransactionV as MemberRemoveTransactionCandid
        let t: MemberRemoveTransaction = {
            memberId: mmm.member_id,
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
        }
        return t;
    }
    if (hasOwnProperty(trs, "WalletCreateTransactionV")) {
        let mmm = trs.WalletCreateTransactionV as WalletCreateTransactionCandid
        let t: WalletCreateTransaction = {
            name: mmm.name,
            uid: mmm.uid,
            network: mapNetworkFromCandid(mmm.network),
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0]
        }
        return t;
    }
    if (hasOwnProperty(trs, "WalletUpdateNameTransactionV")) {
        let mmm = trs.WalletUpdateNameTransactionV as WalletUpdateNameTransactionCandid
        let t: WalletUpdateNameTransaction = {
            name: mmm.name,
            uid: mmm.uid,
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0]
        }
        return t;
    }
    if (hasOwnProperty(trs, "PolicyCreateTransactionV")) {
        let mmm = trs.PolicyCreateTransactionV as PolicyCreateTransactionCandid
        let t: PolicyCreateTransaction = {
            amount_threshold: mmm.amount_threshold,
            currency: Currency.ICP, //TODO
            member_threshold: mmm.member_threshold,
            wallets: mmm.wallets,
            uid: mmm.uid,
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
            memo: mmm.common.memo.length === 0 ? undefined : mmm.common.memo[0]
        }
        return t;
    }
    if (hasOwnProperty(trs, "PolicyUpdateTransactionV")) {
        let mmm = trs.PolicyUpdateTransactionV as PolicyUpdateTransactionCandid
        let t: PolicyUpdateTransaction = {
            amount_threshold: mmm.amount_threshold,
            member_threshold: mmm.member_threshold,
            uid: mmm.uid,
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0]
        }
        return t;
    }
    if (hasOwnProperty(trs, "PolicyRemoveTransactionV")) {
        let mmm = trs.PolicyRemoveTransactionV as PolicyRemoveTransactionCandid
        let t: PolicyRemoveTransaction = {
            uid: mmm.uid,
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0]
        }
        return t;
    }
    if (hasOwnProperty(trs, "TransferTransactionV")) {
        let mmm = trs.TransferTransactionV as TransferTransactionCandid
        let t: TransferTransaction = {
            address: mmm.address,
            amount: mmm.amount,
            currency: Currency.ICP, //TODO
            wallet: mmm.wallet,
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
            memo: mmm.common.memo.length === 0 ? undefined : mmm.common.memo[0],
            policy: mmm.policy.length === 0 ? undefined : mmm.policy[0]
        }
        return t;
    }
    if (hasOwnProperty(trs, "TopUpTransactionV")) {
        let mmm = trs.TopUpTransactionV as TopUpTransactionCandid
        let t: TopUpTransaction = {
            amount: mmm.amount,
            currency: Currency.ICP, //TODO
            wallet: mmm.wallet,
            approves: mmm.common.approves.map(candidToApprove),
            batchUid: mmm.common.batch_uid.length === 0 ? undefined : mmm.common.batch_uid[0],
            createdDate: mmm.common.created_date,
            id: mmm.common.id,
            initiator: mmm.common.initiator,
            isVaultState: mmm.common.is_vault_state,
            modifiedDate: mmm.common.modified_date,
            state: candidToTransactionState(mmm.common.state),
            transactionType: mapTrTypeToTransactionType(mmm.common.transaction_type),
            threshold: mmm.common.threshold.length === 0 ? undefined : mmm.common.threshold[0],
            memo: mmm.common.memo.length === 0 ? undefined : mmm.common.memo[0]
        }
        return t;
    }
    throw Error("Unexpected enum value")
}


export function candidToTransactionState(trType: TransactionStateCandid): TransactionState {
    const transactionTypeKeys = Object.values(TransactionState);
    for (const key of transactionTypeKeys) {
        if (hasOwnProperty(trType, key)) {
            return TransactionState[key];
        }
    }
    throw Error("Invalid transaction state");
}


export function mapTrTypeToTransactionType(trType: TrType): TransactionType {
    const transactionTypeKeys = Object.values(TransactionType);
    for (const key of transactionTypeKeys) {
        if (hasOwnProperty(trType, key)) {
            return TransactionType[key];
        }
    }
    throw Error();
}


export function mapNetworkFromCandid(n: NetworkCandid): Network {
    const networks = Object.values(Network);
    for (const key of networks) {
        if (hasOwnProperty(n, key)) {
            return Network[key];
        }
    }
    throw Error();
}


// A `hasOwnProperty` that produces evidence for the typechecker
export function hasOwnProperty<
    X extends Record<string, unknown>,
    Y extends PropertyKey,
>(obj: X, prop: Y): obj is X & Record<Y, unknown> {
    return Object.prototype.hasOwnProperty.call(obj, prop)
}


abstract class TransactionRequest {
    abstract toCandid(): TransactionRequestCandid
}

export class QuorumTransactionRequest implements TransactionRequest {
    quorum: number
    batch_uid: string | undefined


    constructor(quorum: number, batch_uid?: string) {
        this.quorum = quorum
        this.batch_uid = batch_uid
    }

    toCandid(): TransactionRequestCandid {
        return {
            QuorumUpdateTransactionRequestV: {
                quorum: this.quorum,
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []
            }
        }
    }
}

export class VaultNamingTransactionRequest implements TransactionRequest {
    name: string | undefined
    description: string | undefined
    batch_uid: string | undefined


    constructor(name?: string, description?: string, batch_uid?: string) {
        this.name = name
        this.description = description
        this.batch_uid = batch_uid
    }

    toCandid(): TransactionRequestCandid {
        return {
            VaultNamingUpdateTransactionRequestV: {
                name: this.name !== undefined ? [this.name] : [],
                description: this.description !== undefined ? [this.description] : [],
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []
            }
        }
    }
}

export class TransferTransactionRequest implements TransactionRequest {
    'currency': Currency;
    'address': string;
    'wallet': string;
    'amount': bigint;


    constructor(currency: Currency, address: string, wallet: string, amount: bigint) {
        this.currency = currency
        this.address = address
        this.wallet = wallet
        this.amount = amount
    }

    toCandid(): TransactionRequestCandid {
        return {
            TransferTransactionRequestV: {
                //TODO
                currency: {'ICP': null},
                address: this.address,
                wallet: this.wallet,
                amount: this.amount
            }
        }
    }
}

export class TopUpTransactionRequest implements TransactionRequest {
    'currency': Currency;
    'wallet': string;
    'amount': bigint;


    constructor(currency: Currency, wallet: string, amount: bigint) {
        this.currency = currency
        this.wallet = wallet
        this.amount = amount
    }

    toCandid(): TransactionRequestCandid {
        return {
            TopUpTransactionRequestV: {
                currency: {'ICP': null},
                wallet: this.wallet,
                amount: this.amount
            }
        }
    }
}

export class MemberCreateTransactionRequest implements TransactionRequest {
    member_id: string
    name: string
    role: VaultRole
    batch_uid: string | undefined

    constructor(member: string, name: string, role: VaultRole, batch_uid?: string) {
        this.member_id = member
        this.name = name
        this.role = role
        this.batch_uid = batch_uid
    }

    toCandid(): TransactionRequestCandid {
        return {
            MemberCreateTransactionRequestV: {
                member_id: this.member_id, name: this.name, role: roleToCandid(this.role),
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []
            }
        }
    }
}

export class WalletCreateTransactionRequest implements TransactionRequest {
    network: Network
    name: string
    batch_uid: string | undefined

    constructor(name: string, network: Network, batch_uid?: string) {
        this.network = network
        this.name = name
        this.batch_uid = batch_uid

    }

    toCandid(): TransactionRequestCandid {
        return {
            WalletCreateTransactionRequestV: {
                name: this.name,
                network: networkToCandid(this.network),
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []
            }
        }
    }
}

export class WalletUpdateNameTransactionRequest implements TransactionRequest {
    uid: string
    name: string
    batch_uid: string | undefined

    constructor(name: string, uid: string, batch_uid?: string) {
        this.uid = uid
        this.name = name
        this.batch_uid = batch_uid

    }

    toCandid(): TransactionRequestCandid {
        return {
            WalletUpdateNameTransactionRequestV: {
                name: this.name,
                uid: this.uid,
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []
            }
        }
    }
}

export class MemberUpdateNameTransactionRequest implements TransactionRequest {
    member_id: string
    name: string
    batch_uid: string | undefined

    constructor(member: string, name: string, batch_uid?: string) {
        this.member_id = member
        this.name = name
        this.batch_uid = batch_uid

    }

    toCandid(): TransactionRequestCandid {
        return {
            MemberUpdateNameTransactionRequestV: {
                member_id: this.member_id, name: this.name,
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []

            }
        }
    }
}

export class MemberUpdateRoleTransactionRequest implements TransactionRequest {
    member_id: string
    role: VaultRole
    batch_uid: string | undefined

    constructor(member_id: string, role: VaultRole, batch_uid?: string) {
        this.member_id = member_id
        this.role = role
        this.batch_uid = batch_uid

    }

    toCandid(): TransactionRequestCandid {
        return {
            MemberUpdateRoleTransactionRequestV: {
                member_id: this.member_id, role: roleToCandid(this.role),
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []
            }
        }
    }
}

export class MemberRemoveTransactionRequest implements TransactionRequest {
    member_id: string
    batch_uid: string | undefined

    constructor(member_id: string, batch_uid?: string) {
        this.member_id = member_id
        this.batch_uid = batch_uid

    }

    toCandid(): TransactionRequestCandid {
        return {
            MemberRemoveTransactionRequestV: {
                member_id: this.member_id,
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []
            }
        }
    }
}

export class VersionUpgradeTransactionRequest implements TransactionRequest {
    version: string

    constructor(version: string) {
        this.version = version
    }

    toCandid(): TransactionRequestCandid {
        return {
            VersionUpgradeTransactionRequestV: {
                version: this.version,
            }
        }
    }
}

export class PolicyCreateTransactionRequest implements TransactionRequest {
    'member_threshold': number;
    'amount_threshold': bigint;
    'wallets': Array<string>;
    batch_uid: string | undefined

    constructor(member_threshold: number, amount_threshold: bigint, wallets: Array<string>, batch_uid?: string) {
        this.member_threshold = member_threshold
        this.amount_threshold = amount_threshold
        this.wallets = wallets
        this.batch_uid = batch_uid

    }

    toCandid(): TransactionRequestCandid {
        return {
            PolicyCreateTransactionRequestV: {
                'member_threshold': this.member_threshold,
                'amount_threshold': this.amount_threshold,
                'wallets': this.wallets,
                'currency': {'ICP': null}, //TODO
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []
            }
        }
    }
}

export class PolicyUpdateTransactionRequest implements TransactionRequest {
    'uid': string;
    'member_threshold': number;
    'amount_threshold': bigint;
    batch_uid: string | undefined

    constructor(uid: string, member_threshold: number, amount_threshold: bigint, batch_uid?: string) {
        this.uid = uid
        this.member_threshold = member_threshold
        this.amount_threshold = amount_threshold
        this.batch_uid = batch_uid

    }

    toCandid(): TransactionRequestCandid {
        return {
            PolicyUpdateTransactionRequestV: {
                'uid': this.uid,
                'member_threshold': this.member_threshold,
                'amount_threshold': this.amount_threshold,
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []

            }
        }
    }
}

export class PolicyRemoveTransactionRequest implements TransactionRequest {
    'uid': string;
    batch_uid: string | undefined

    constructor(uid: string, batch_uid?: string) {
        this.uid = uid
        this.batch_uid = batch_uid
    }

    toCandid(): TransactionRequestCandid {
        return {
            PolicyRemoveTransactionRequestV: {
                'uid': this.uid,
                batch_uid: this.batch_uid !== undefined ? [this.batch_uid] : []

            }
        }
    }
}


export function roleToCandid(response: VaultRole): VaultRoleCandid {
    if (response === VaultRole.ADMIN) {
        return {Admin: null} as VaultRoleCandid
    }
    if (response === VaultRole.MEMBER) {
        return {Member: null} as VaultRoleCandid
    }
    throw Error("Unexpected enum value")
}

export function networkToCandid(network: Network): NetworkCandid {
    if (network === Network.IC) {
        return {IC: null} as NetworkCandid
    }
    throw Error("Unexpected enum value")
}


export interface Approve {
    status: TransactionState
    signer: string
    createdDate: bigint
}


export function candidToApprove(response: ApproveCandid): Approve {
    return {
        createdDate: response.created_date,
        signer: response.signer,
        status: candidToTransactionState(response.status)
    }
}


export function transactionStateToCandid(
    state: TransactionState,
): TransactionStateCandid {
    if (state === TransactionState.Approved) {
        return {Approved: null}
    }
    if (state === TransactionState.Rejected) {
        return {Rejected: null}
    }
    throw Error("Unexpected enum value")
}

export interface ApproveRequest {
    tr_id: bigint
    state: TransactionState
}

export function approveToCandid(a: ApproveRequest): TransactionApproveRequest {
    return {
        state: transactionStateToCandid(a.state),
        transaction_id: a.tr_id,
    }
}