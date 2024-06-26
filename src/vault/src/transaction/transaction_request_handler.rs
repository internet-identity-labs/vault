use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::security_service::verify_caller;
use crate::transaction::member::member_create_transaction::{MemberCreateTransactionBuilder, MemberCreateTransactionRequest};
use crate::transaction::member::member_create_transaction_v2::{MemberCreateTransactionBuilderV2, MemberCreateTransactionRequestV2};
use crate::transaction::member::member_extend_account_transaction::{MemberExtendICRC1AccountBuilder, MemberExtendICRC1AccountRequest};
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
use crate::transaction::transfer::top_up_quorum_transaction::{TopUpQuorumTransaction, TopUpQuorumTransactionBuilder, TopUpQuorumTransactionRequest};
use crate::transaction::transfer::top_up_transaction::{TopUpTransactionBuilder, TopUpTransactionRequest};
use crate::transaction::transfer::transfer_icrc1_quorum_transaction::{TransferICRC1QuorumTransactionBuilder, TransferICRC1QuorumTransactionRequest};
use crate::transaction::transfer::transfer_quorum_transaction::{TransferQuorumTransactionBuilder, TransferQuorumTransactionRequest};
use crate::transaction::transfer::transfer_transaction::{TransferTransactionBuilder, TransferTransactionRequest};
use crate::transaction::upgrade::upgrade_transaction::{VersionUpgradeTransactionBuilder, VersionUpgradeTransactionRequest};
use crate::transaction::vault::controllers_transaction::{ControllersUpdateTransactionBuilder, ControllersUpdateTransactionRequest};
use crate::transaction::vault::add_icrc1_canisters_transaction::{ICRC1CanistersAddTransactionBuilder, ICRC1CanistersAddTransactionRequest};
use crate::transaction::vault::quorum_transaction::{QuorumUpdateTransactionBuilder, QuorumUpdateTransactionRequest};
use crate::transaction::vault::remove_icrc1_canisters_transaction::{ICRC1CanistersRemoveTransactionBuilder, ICRC1CanistersRemoveTransactionRequest};
use crate::transaction::vault::vault_naming_transaction::{VaultNamingUpdateTransactionBuilder, VaultNamingUpdateTransactionRequest};
use crate::transaction::wallet::wallet_create_transaction::{WalletCreateTransactionBuilder, WalletCreateTransactionRequest};
use crate::transaction::wallet::wallet_update_name_transaction::{WalletUpdateNameTransactionBuilder, WalletUpdateNameTransactionRequest};
use crate::util::caller_to_address;

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionRequest {
    MemberCreateTransactionRequestV(MemberCreateTransactionRequest),
    MemberCreateTransactionRequestV2(MemberCreateTransactionRequestV2),
    MemberExtendICRC1AccountRequestV(MemberExtendICRC1AccountRequest),
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
    TransferQuorumTransactionRequestV(TransferQuorumTransactionRequest),
    TopUpTransactionRequestV(TopUpTransactionRequest),
    TopUpQuorumTransactionRequestV(TopUpQuorumTransactionRequest),
    VersionUpgradeTransactionRequestV(VersionUpgradeTransactionRequest),
    ControllersUpdateTransactionRequestV(ControllersUpdateTransactionRequest),
    TransferICRC1QuorumTransactionRequestV(TransferICRC1QuorumTransactionRequest),
    ICRC1CanistersAddTransactionRequestV(ICRC1CanistersAddTransactionRequest),
    ICRC1CanistersRemoveTransactionRequestV(ICRC1CanistersRemoveTransactionRequest),
}


pub async fn handle_transaction_request(trr: TransactionRequest) -> TransactionCandid {
    let mut trs: Box<dyn ITransaction> = match trr {
        TransactionRequest::MemberCreateTransactionRequestV(request) => {
            MemberCreateTransactionBuilder::init(request).build().await
        }
        TransactionRequest::MemberCreateTransactionRequestV2(request) => {
            MemberCreateTransactionBuilderV2::init(request).build().await
        }
        TransactionRequest::MemberExtendICRC1AccountRequestV(request) => {
            MemberExtendICRC1AccountBuilder::init(request).build().await
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
        TransactionRequest::TopUpQuorumTransactionRequestV(request) => {
            TopUpQuorumTransactionBuilder::init(request).build().await
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
        TransactionRequest::TransferQuorumTransactionRequestV(request) => {
            TransferQuorumTransactionBuilder::init(request).build().await
        }
        TransactionRequest::TransferICRC1QuorumTransactionRequestV(request) => {
            TransferICRC1QuorumTransactionBuilder::init(request).build().await
        }
        TransactionRequest::ICRC1CanistersAddTransactionRequestV(request) => {
            ICRC1CanistersAddTransactionBuilder::init(request).build().await
        }
        TransactionRequest::ICRC1CanistersRemoveTransactionRequestV(request) => {
            ICRC1CanistersRemoveTransactionBuilder::init(request).build().await
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
