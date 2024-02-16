use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::security_service::verify_caller;
use crate::transaction::member::member_create_transaction::{MemberCreateTransactionBuilder, MemberCreateTransactionRequest};
use crate::transaction::member::member_remove_transaction::{MemberRemoveTransactionBuilder, MemberRemoveTransactionRequest};
use crate::transaction::member::member_update_name_transaction::{MemberUpdateNameTransactionBuilder, MemberUpdateNameTransactionRequest};
use crate::transaction::member::member_update_role_transaction::{MemberUpdateRoleTransactionBuilder, MemberUpdateRoleTransactionRequest};
use crate::transaction::policy::policy_create_transaction::{PolicyCreateTransactionBuilder, PolicyCreateTransactionRequest};
use crate::transaction::policy::policy_remove_transaction::{PolicyRemoveTransactionBuilder, PolicyRemoveTransactionRequest};
use crate::transaction::policy::policy_update_transaction::{PolicyUpdateTransactionBuilder, PolicyUpdateTransactionRequest};
use crate::transaction::purge::purge_transaction::{PurgeTransactionBuilder, PurgeTransactionRequest};
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_approve_handler::Approve;
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transaction_service::store_transaction;
use crate::transaction::transfer::top_up_transaction::{TopUpTransactionBuilder, TopUpTransactionRequest};
use crate::transaction::transfer::transfer_transaction::{TransferTransactionBuilder, TransferTransactionRequest};
use crate::transaction::upgrade::upgrade_transaction::{VersionUpgradeTransactionBuilder, VersionUpgradeTransactionRequest};
use crate::transaction::vault::controllers_transaction::{ControllersUpdateTransactionBuilder, ControllersUpdateTransactionRequest};
use crate::transaction::vault::quorum_transaction::{QuorumUpdateTransactionBuilder, QuorumUpdateTransactionRequest};
use crate::transaction::vault::vault_naming_transaction::{VaultNamingUpdateTransactionBuilder, VaultNamingUpdateTransactionRequest};
use crate::transaction::wallet::wallet_create_transaction::{WalletCreateTransactionBuilder, WalletCreateTransactionRequest};
use crate::transaction::wallet::wallet_update_name_transaction::{WalletUpdateNameTransactionBuilder, WalletUpdateNameTransactionRequest};
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
    PurgeTransactionRequestV(PurgeTransactionRequest),
    VaultNamingUpdateTransactionRequestV(VaultNamingUpdateTransactionRequest),
    PolicyCreateTransactionRequestV(PolicyCreateTransactionRequest),
    PolicyUpdateTransactionRequestV(PolicyUpdateTransactionRequest),
    PolicyRemoveTransactionRequestV(PolicyRemoveTransactionRequest),
    TransferTransactionRequestV(TransferTransactionRequest),
    TopUpTransactionRequestV(TopUpTransactionRequest),
    VersionUpgradeTransactionRequestV(VersionUpgradeTransactionRequest),
    ControllersUpdateTransactionRequestV(ControllersUpdateTransactionRequest)
}


pub async fn handle_transaction_request(trr: TransactionRequest) -> TransactionCandid {
    let mut trs: Box<dyn ITransaction> = match trr {
        TransactionRequest::MemberCreateTransactionRequestV(request) => {
            MemberCreateTransactionBuilder::init(request).build().await
        }
        TransactionRequest::MemberUpdateNameTransactionRequestV(r) => {
            MemberUpdateNameTransactionBuilder::init(r).build().await
        }
        TransactionRequest::MemberUpdateRoleTransactionRequestV(r) => {
            MemberUpdateRoleTransactionBuilder::init(r).build().await
        }
        TransactionRequest::QuorumUpdateTransactionRequestV(r) => {
            QuorumUpdateTransactionBuilder::init(r).build().await
        }
        TransactionRequest::WalletCreateTransactionRequestV(trs) => {
            WalletCreateTransactionBuilder::init(trs).build().await
        }
        TransactionRequest::WalletUpdateNameTransactionRequestV(request) => {
            WalletUpdateNameTransactionBuilder::init(request).build().await
        }
        TransactionRequest::MemberRemoveTransactionRequestV(request) => {
            MemberRemoveTransactionBuilder::init(request).build().await
        }
        TransactionRequest::PolicyCreateTransactionRequestV(request) => {
            PolicyCreateTransactionBuilder::init(request).build().await
        }
        TransactionRequest::PolicyUpdateTransactionRequestV(request) => {
            PolicyUpdateTransactionBuilder::init(request).build().await
        }
        TransactionRequest::PolicyRemoveTransactionRequestV(request) => {
            PolicyRemoveTransactionBuilder::init(request).build().await
        }
        TransactionRequest::VaultNamingUpdateTransactionRequestV(request) => {
            VaultNamingUpdateTransactionBuilder::init(request).build().await
        }
        TransactionRequest::TransferTransactionRequestV(request) => {
            TransferTransactionBuilder::init(request).build().await
        }
        TransactionRequest::TopUpTransactionRequestV(request) => {
            TopUpTransactionBuilder::init(request).build().await
        }
        TransactionRequest::VersionUpgradeTransactionRequestV(request) => {
            VersionUpgradeTransactionBuilder::init(request).build().await
        }
        TransactionRequest::PurgeTransactionRequestV(request) => {
            PurgeTransactionBuilder::init(request).build().await
        }
        TransactionRequest::ControllersUpdateTransactionRequestV(request) => {
            ControllersUpdateTransactionBuilder::init(request).build().await
        }
    };
    verify_caller(trs.get_accepted_roles());
    let approve = Approve {
        signer: caller_to_address(),
        created_date: time(),
        status: TransactionState::Approved,
    };
    trs.handle_approve(approve);
    store_transaction(trs.clone());
    trs.to_candid()
}
