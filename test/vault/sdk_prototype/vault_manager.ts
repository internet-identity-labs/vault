import {
    Approve as ApproveCandid,
    Member,
    MemberCreateTransaction as MemberCreateTransactionCandid,
    MemberUpdateNameTransaction as MemberUpdateNameTransactionCandid,
    MemberUpdateRoleTransaction as MemberUpdateRoleTransactionCandid,
    ObjectState as ObjectStateCandid,
    QuorumUpdateTransaction as QuorumUpdateTransactionCandid,
    TransactionApproveRequest,
    TransactionCandid,
    TransactionRequest as TransactionRequestCandid,
    TransactionState as TransactionStateCandid,
    TrType,
    VaultRole as VaultRoleCandid,
    VaultState
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

    async requestTransaction(request: Array<TransactionRequest>): Promise<Array<Transaction>> {
        let b: Array<TransactionRequestCandid> = request.map(l => l.toCandid())
        let a = await this.actor.request_transaction(b) as [TransactionCandid];
        return a.map(transactionCandidToTransaction)
    }

    getState(): Vault {
        return this.state
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
    MemberUnarchive = 'MemberUnarchive',
    WalletCreate = 'WalletCreate',
    MemberArchive = 'MemberArchive',
    PolicyCreate = 'PolicyCreate',
    PolicyUpdate = 'PolicyUpdate',
    MemberUpdateName = 'MemberUpdateName',
    MemberUpdateRole = 'MemberUpdateRole',
    QuorumUpdate = 'QuorumUpdate',
    Transfer = 'Transfer'
}

export enum Currency {
    ICP = "ICP",
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


export class Vault {
    members: Array<VaultMember>
    quorum: Quorum
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
    member_id: string;
    name: string;
    role: VaultRole;
}


export interface MemberUpdateNameTransaction extends Transaction {
    member_id: string;
    name: string;
}


export interface MemberUpdateRoleTransaction extends Transaction {
    member_id: string;
    role: VaultRole;
}

export interface QuorumUpdateTransaction extends Transaction {
    quorum: number
}

function vaultCandidToVault(vaultCandid: VaultState): Vault {
    let members: Array<VaultMember> = vaultCandid.members.map(mapMember)
    let quorum: Quorum = {
        modifiedDate: vaultCandid.quorum.modified_date,
        quorum: vaultCandid.quorum.quorum
    }
    let v: Vault = {
        members: members, quorum: quorum
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
    if (hasOwnProperty(trs, "MemberCreateTransactionV")) {
        let mmm = trs.MemberCreateTransactionV as MemberCreateTransactionCandid
        let t: MemberCreateTransaction = {
            member_id: mmm.member_id,
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
            member_id: mmm.member_id,
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
            member_id: mmm.member_id,
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


// A `hasOwnProperty` that produces evidence for the typechecker
export function hasOwnProperty<
    X extends Record<string, unknown>,
    Y extends PropertyKey,
>(obj: X, prop: Y): obj is X & Record<Y, unknown> {
    return Object.prototype.hasOwnProperty.call(obj, prop)
}


interface TransactionRequest {
    toCandid(): TransactionRequestCandid
}

export class QuorumTransactionRequest implements TransactionRequest {
    quorum: number

    constructor(quorum: number) {
        this.quorum = quorum
    }

    toCandid(): TransactionRequestCandid {
        return {
            QuorumUpdateTransactionRequestV: {
                quorum: this.quorum
            }
        }
    }
}

export class MemberCreateTransactionRequest implements TransactionRequest {
    member_id: string
    name: string
    role: VaultRole


    constructor(member: string, name: string, role: VaultRole) {
        this.member_id = member
        this.name = name
        this.role = role
    }

    toCandid(): TransactionRequestCandid {
        return {
            MemberCreateTransactionRequestV: {
                member_id: this.member_id, name: this.name, role: roleToCandid(this.role)
            }
        }
    }
}

export class MemberUpdateNameTransactionRequest implements TransactionRequest {
    member_id: string
    name: string


    constructor(member: string, name: string) {
        this.member_id = member
        this.name = name
    }

    toCandid(): TransactionRequestCandid {
        return {
            MemberUpdateNameTransactionRequestV: {
                member_id: this.member_id, name: this.name
            }
        }
    }
}

export class MemberUpdateRoleTransactionRequest implements TransactionRequest {
    member_id: string
    role: VaultRole


    constructor(member_id: string, role: VaultRole) {
        this.member_id = member_id
        this.role = role
    }

    toCandid(): TransactionRequestCandid {
        return {
            MemberUpdateRoleTransactionRequestV: {
                member_id: this.member_id, role: roleToCandid(this.role)
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