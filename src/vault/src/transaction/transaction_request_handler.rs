use candid::CandidType;
use candid::types::{Serializer, Type, TypeId};
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::{ObjectState, TransactionState};
use crate::transaction::member_transaction::MemberTransactionBuilder;
use crate::transaction::members::{get_member_by_id, Member};
use crate::transaction::quorum_transaction::QuorumTransactionBuilder;
use crate::transaction::transaction::{ TransactionNew, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transactions_service::store_transaction;
use crate::transaction_service::Approve;
use crate::util::caller_to_address;
use crate::vault_service::VaultRole;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct
QuorumTransactionRequest {
    pub amount: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CreateMemberTransactionRequest {
    pub member: String,
    pub name: String,
    pub role: VaultRole,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UpdateMemberNameTransactionRequest {
    pub member: String,
    pub name: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UpdateMemberRoleTransactionRequest {
    pub member: String,
    pub role: VaultRole,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ArchiveMemberTransactionRequest {
    pub member: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UnArchiveMemberTransactionRequest {
    pub member: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionRequestType {
    MemberCreateTransactionRequest(CreateMemberTransactionRequest),
    MemberUpdateNameTransactionRequest(UpdateMemberNameTransactionRequest),
    MemberUpdateRoleTransactionRequest(UpdateMemberRoleTransactionRequest),
    MemberArchiveTransactionRequest(ArchiveMemberTransactionRequest),
    MemberUnArchiveTransactionRequest(UnArchiveMemberTransactionRequest),
    QuorumStateTransaction(QuorumTransactionRequest),
}

pub fn handle_transaction_request(trr: TransactionRequestType) {
    let mut trs: Box<dyn TransactionNew> = match trr {
        TransactionRequestType::QuorumStateTransaction(x) => {
            QuorumTransactionBuilder::init(x)
                .build()
        }
        TransactionRequestType::MemberCreateTransactionRequest(request) => {
            let new_member = Member::new(request.member, request.role, request.name);
            MemberTransactionBuilder::init(TrType::MemberCreate, new_member)
                .build()
        }
        TransactionRequestType::MemberUpdateNameTransactionRequest(request) => {
            let mut member = get_member_by_id(&request.member);
            member.name = request.name;
            MemberTransactionBuilder::init(TrType::MemberUpdateName, member)
                .build()
        }
        TransactionRequestType::MemberArchiveTransactionRequest(request) => {
            let mut member = get_member_by_id(&request.member);
            member.state = ObjectState::Archived;
            MemberTransactionBuilder::init(TrType::MemberArchive, member)
                .build()
        }
        TransactionRequestType::MemberUnArchiveTransactionRequest(request) => {
            let mut member = get_member_by_id(&request.member);
            member.state = ObjectState::Active;
            MemberTransactionBuilder::init(TrType::MemberUnArchive, member)
                .build()
        }
        TransactionRequestType::MemberUpdateRoleTransactionRequest(request) => {
            let mut member = get_member_by_id(&request.member);
            member.role = request.role;
            MemberTransactionBuilder::init(TrType::MemberUpdateRole, member)
                .build()
        }
    };

    let approve = Approve {
        signer: caller_to_address(),
        created_date: time(),
        status: TransactionState::Approved,
    };
    trs.handle_approve(approve);
    trs.define_state();
    store_transaction(trs);
}
