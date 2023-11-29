use candid::CandidType;
use candid::types::Serializer;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::transaction::member::member_create_transaction::{MemberCreateTransactionBuilder, MemberCreateTransactionRequest};
use crate::transaction::member::member_remove_transaction::{MemberRemoveTransactionBuilder, MemberRemoveTransactionRequest};
use crate::transaction::member::member_update_name_transaction::{MemberUpdateNameTransactionBuilder, MemberUpdateNameTransactionRequest};
use crate::transaction::member::member_update_role_transaction::{MemberUpdateRoleTransactionBuilder, MemberUpdateRoleTransactionRequest};
use crate::transaction::policy::policy_create_transaction::{PolicyCreateTransactionBuilder, PolicyCreateTransactionRequest};
use crate::transaction::policy::policy_remove_transaction::{PolicyRemoveTransactionBuilder, PolicyRemoveTransactionRequest};
use crate::transaction::policy::policy_update_transaction::{PolicyUpdateTransactionBuilder, PolicyUpdateTransactionRequest};
use crate::transaction::quorum::quorum_transaction::{QuorumUpdateTransactionBuilder, QuorumUpdateTransactionRequest};
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transactions_service::store_transaction;
use crate::transaction::wallet::wallet::generate_address;
use crate::transaction::wallet::wallet_create_transaction::{WalletCreateTransactionBuilder, WalletCreateTransactionRequest};
use crate::transaction::wallet::wallet_update_name_transaction::{WalletUpdateNameTransactionBuilder, WalletUpdateNameTransactionRequest};
use crate::transaction_service::Approve;
use crate::util::caller_to_address;

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionRequest {
    MemberCreateTransactionRequestV(MemberCreateTransactionRequest),
    MemberUpdateNameTransactionRequestV(MemberUpdateNameTransactionRequest),
    MemberUpdateRoleTransactionRequestV(MemberUpdateRoleTransactionRequest),
    MemberRemoveTransactionRequestV(MemberRemoveTransactionRequest),
    WalletUpdateNameTransactionRequestV(WalletUpdateNameTransactionRequest),
    WalletCreateTransactionRequestV(WalletCreateTransactionRequest),
    QuorumUpdateTransactionRequestV(QuorumUpdateTransactionRequest),
    PolicyCreateTransactionRequestV(PolicyCreateTransactionRequest),
    PolicyUpdateTransactionRequestV(PolicyUpdateTransactionRequest),
    PolicyRemoveTransactionRequestV(PolicyRemoveTransactionRequest),
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
        TransactionRequest::QuorumUpdateTransactionRequestV(r) => {
            QuorumUpdateTransactionBuilder::init(r).build()
        }
        TransactionRequest::WalletCreateTransactionRequestV(trs) => {
            //only one async call in builders - no reason to make trait async ???
            let generated_random_address = generate_address().await;
            WalletCreateTransactionBuilder::init(trs, generated_random_address).build()
        }
        TransactionRequest::WalletUpdateNameTransactionRequestV(request) => {
            WalletUpdateNameTransactionBuilder::init(request).build()
        }
        TransactionRequest::MemberRemoveTransactionRequestV(request) => {
            MemberRemoveTransactionBuilder::init(request).build()
        }
        TransactionRequest::PolicyCreateTransactionRequestV(request) => {
            let generated_random_uid = generate_address().await;
            PolicyCreateTransactionBuilder::init(request, generated_random_uid).build()
        }
        TransactionRequest::PolicyUpdateTransactionRequestV(request) => {
            PolicyUpdateTransactionBuilder::init(request).build()
        }
        TransactionRequest::PolicyRemoveTransactionRequestV(request) => {
            PolicyRemoveTransactionBuilder::init(request).build()
        }
    };

    let approve = Approve {
        signer: caller_to_address(),
        created_date: time(),
        status: TransactionState::Approved,
    };
    trs.handle_approve(approve);
    store_transaction(trs.clone());
    trs.to_candid()
}
