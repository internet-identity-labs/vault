use candid::CandidType;
use candid::types::Serializer;
use ic_cdk::api::time;
use ic_cdk::trap;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::policy_service::Currency;
use crate::transaction::transaction::TransactionNew;
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transactions_service::store_transaction;
use crate::transaction::wallet::wallet_update_transaction::WalletUpdateNameTransactionBuilder;
use crate::transaction_service::Approve;
use crate::util::caller_to_address;
use crate::vault_service::VaultRole;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct
QuorumTransactionRequest {
    pub amount: u8,
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

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UpdateWalletNameTransactionRequest {
    pub uid: String,
    pub name: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ArchiveWalletTransactionRequest {
    pub uid: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UnArchiveWalletTransactionRequest {
    pub uid: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CreateWalletTransactionRequest {
    pub currency: Currency,
    pub name: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionRequestType {
    MemberCreateTransactionRequest(CreateMemberTransactionRequest),
    MemberUpdateNameTransactionRequest(UpdateMemberNameTransactionRequest),
    MemberUpdateRoleTransactionRequest(UpdateMemberRoleTransactionRequest),
    MemberArchiveTransactionRequest(ArchiveMemberTransactionRequest),
    MemberUnArchiveTransactionRequest(UnArchiveMemberTransactionRequest),
    WalletUpdateNameTransactionRequest(UpdateWalletNameTransactionRequest),
    WalletArchiveTransactionRequest(ArchiveWalletTransactionRequest),
    WalletUnArchiveTransactionRequest(UnArchiveWalletTransactionRequest),
    WalletCreateTransactionRequest(CreateWalletTransactionRequest),
    QuorumStateTransaction(QuorumTransactionRequest),
}

pub async fn handle_transaction_request(trr: TransactionRequestType) {
    let mut trs: Box<dyn TransactionNew> = match trr {
        // TransactionRequestType::QuorumStateTransaction(x) => {
        //     QuorumTransactionBuilder::init(x)
        //         .build()
        // }
        // TransactionRequestType::MemberCreateTransactionRequest(request) => {
        //     let new_member = Member::new(request.member, request.role, request.name);
        //     MemberTransactionBuilder::init(TrType::MemberCreate, new_member)
        //         .build()
        // }
        // TransactionRequestType::MemberUpdateNameTransactionRequest(request) => {
        //     let mut member = get_member_by_id(&request.member);
        //     member.name = request.name;
        //     MemberTransactionBuilder::init(TrType::MemberUpdateName, member)
        //         .build()
        // }
        // TransactionRequestType::MemberArchiveTransactionRequest(request) => {
        //     let mut member = get_member_by_id(&request.member);
        //     member.state = ObjectState::Archived;
        //     MemberTransactionBuilder::init(TrType::MemberArchive, member)
        //         .build()
        // }
        // TransactionRequestType::MemberUnArchiveTransactionRequest(request) => {
        //     let mut member = get_member_by_id(&request.member);
        //     member.state = ObjectState::Active;
        //     MemberTransactionBuilder::init(TrType::MemberUnArchive, member)
        //         .build()
        // }
        // TransactionRequestType::MemberUpdateRoleTransactionRequest(request) => {
        //     let mut member = get_member_by_id(&request.member);
        //     member.role = request.role;
        //     MemberTransactionBuilder::init(TrType::MemberUpdateRole, member)
        //         .build()
        // }
        TransactionRequestType::WalletUpdateNameTransactionRequest(request) => {
            WalletUpdateNameTransactionBuilder::init(request.uid, request.name).build()
        }
        // TransactionRequestType::WalletCreateTransactionRequest(request) => {
        //     let wallet = Wallet {
        //         uid: generate_address().await, //TODO
        //         name: request.name,
        //         currency: Currency::ICP,
        //         state: Active,
        //         modified_date: time(),
        //         created_date: time(),
        //     };
        //     WalletTransactionBuilder::init(WalletCreate, wallet).build()
        // }
        _ => {
            trap("".clone())
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
