use candid::CandidType;
use candid::types::Serializer;
use ic_cdk::api::time;
use ic_cdk::trap;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::transaction::member::member_archive_transaction::MemberArchiveTransactionBuilder;
use crate::transaction::member::member_create_transaction::{MemberCreateTransactionBuilder, MemberCreateTransactionRequest};
use crate::transaction::member::member_unarchive_transaction::MemberUnarchiveTransactionBuilder;
use crate::transaction::member::member_update_name_transaction::{MemberUpdateNameTransactionBuilder, MemberUpdateNameTransactionRequest};
use crate::transaction::member::member_update_role_transaction::{MemberUpdateRoleTransactionBuilder, MemberUpdateRoleTransactionRequest};
use crate::transaction::quorum::quorum_transaction::{QuorumUpdateTransactionBuilder, QuorumUpdateTransactionRequest};
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transactions_service::store_transaction;
use crate::transaction::wallet::wallet_create_transaction::WalletCreateTransactionRequest;
use crate::transaction::wallet::wallet_update_transaction::{WalletUpdateNameTransactionBuilder, WalletUpdateNameTransactionRequest};
use crate::transaction_service::Approve;
use crate::util::caller_to_address;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberArchiveTransactionRequest {
    pub member: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberUnArchiveTransactionRequest {
    pub member: String,
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ArchiveWalletTransactionRequest {
    pub uid: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UnArchiveWalletTransactionRequest {
    pub uid: String,
}


#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionRequest {
    MemberCreateTransactionRequestV(MemberCreateTransactionRequest),
    MemberUpdateNameTransactionRequestV(MemberUpdateNameTransactionRequest),
    MemberUpdateRoleTransactionRequestV(MemberUpdateRoleTransactionRequest),
    MemberArchiveTransactionRequestV(MemberArchiveTransactionRequest),
    MemberUnArchiveTransactionRequestV(MemberUnArchiveTransactionRequest),
    WalletUpdateNameTransactionRequestV(WalletUpdateNameTransactionRequest),
    WalletCreateTransactionRequestV(WalletCreateTransactionRequest),
    QuorumUpdateTransactionRequestV(QuorumUpdateTransactionRequest),
}

pub async fn handle_transaction_request(trr: TransactionRequest) -> TransactionCandid {
    let mut trs: Box<dyn ITransaction> = match trr {
        TransactionRequest::MemberCreateTransactionRequestV(request) => {
            MemberCreateTransactionBuilder::init(request).build()
        }
        TransactionRequest::MemberUpdateNameTransactionRequestV(r) => {
            MemberUpdateNameTransactionBuilder::init(r).build()
        }
        TransactionRequest::MemberUpdateRoleTransactionRequestV(r) => {
            MemberUpdateRoleTransactionBuilder::init(r).build()
        }
        TransactionRequest::MemberArchiveTransactionRequestV(r) => {
            MemberArchiveTransactionBuilder::init(r.member).build()
        }
        TransactionRequest::MemberUnArchiveTransactionRequestV(r) => {
            MemberUnarchiveTransactionBuilder::init(r.member).build()
        }
        TransactionRequest::QuorumUpdateTransactionRequestV(r) => {
            QuorumUpdateTransactionBuilder::init(r).build()
        }
        TransactionRequest::WalletCreateTransactionRequestV(trs) => {
            trap("");
            // QuorumUpdateTransactionBuilder::init(trs).build()
        }
        TransactionRequest::WalletUpdateNameTransactionRequestV(request) => {
            WalletUpdateNameTransactionBuilder::init(request).build()
        }
    };

    let approve = Approve {
        signer: caller_to_address(),
        created_date: time(),
        status: TransactionState::Approved,
    };
    trs.handle_approve(approve);
    trs.define_state();
    store_transaction(trs.clone());
    trs.to_candid()
}
