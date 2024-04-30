extern crate core;
extern crate maplit;

use std::cell::RefCell;
use std::string::ToString;

use candid::{CandidType, export_service, Principal};
use ic_cdk::{call, id};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_cdk::api::time;
use ic_cdk_macros::*;
use serde::Deserialize;

use nfid_certified::{CertifiedResponse, get_trusted_origins_cert, update_trusted_origins};

use crate::config::{Conf, CONF};
use crate::enums::{TransactionState, VaultRole};
use crate::security_service::verify_caller;
use crate::state::{get_vault_state, save_icrc1_canister, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::member::member_create_transaction::MemberCreateTransaction;
use crate::transaction::transaction::TransactionCandid;
use crate::transaction::transaction_approve_handler::{Approve, handle_approve, TransactionApproveRequest};
use crate::transaction::transaction_request_handler::{handle_transaction_request, TransactionRequest};
use crate::transaction::transaction_service::{execute_approved_transactions, get_all_transactions, stable_restore, stable_save, store_transaction};
use crate::util::{to_address, to_array};
use crate::version_const::VERSION;
use crate::security_service::is_caller_registered;

mod util;
mod enums;
mod security_service;
mod transfer_service;
mod config;
mod transaction;
mod state;
mod errors;
mod version_const;


thread_local! {
    pub static HEART_COUNT: RefCell<u8> = RefCell::new(0);
}

#[init]
async fn init(initiator: Principal, conf: Conf) {
    update_trusted_origins(conf.origins.clone());
    CONF.with(|c| c.replace(conf));
    let member_id = to_address(initiator);
    let mut mc = MemberCreateTransaction::new(
        TransactionState::Approved, None, member_id.clone(), "Initiator".to_string(), VaultRole::Admin,
    );
    mc.get_common_mut().approves.insert(Approve {
        signer: member_id,
        created_date: time(),
        status: TransactionState::Approved,
    });
    store_transaction(mc.clone_self());
    execute_approved_transactions().await
}

#[query]
async fn get_version() -> String {
    VERSION.to_string()
}

#[update (guard = "is_caller_registered")]
async fn request_transaction(transaction_request: Vec<TransactionRequest>) -> Vec<TransactionCandid> {
    let mut trs: Vec<TransactionCandid> = Default::default();
    for tr in transaction_request {
        let a = handle_transaction_request(tr).await;
        trs.push(a);
    }
    trs
}

#[update]
async fn execute() {
    execute_approved_transactions().await
}


#[query]
async fn get_transactions_all() -> Vec<TransactionCandid> {
    get_all_transactions()
        .into_iter()
        .map(|l| l.to_candid())
        .collect()
}


#[query]
async fn get_state(tr_id: Option<u64>) -> VaultState {
    get_vault_state(tr_id).await
}

#[update (guard = "is_caller_registered")]
async fn approve(request: Vec<TransactionApproveRequest>) -> Vec<TransactionCandid> {
    let mut approved_trs = Vec::default();
    for approve in request {
        let trs = handle_approve(approve.transaction_id, approve.state);
        approved_trs.push(trs);
    }
    approved_trs
}

#[query]
async fn canister_balance() -> u64 {
    ic_cdk::api::canister_balance()
}

#[test]
fn sub_account_test() {}
export_service!();

#[pre_upgrade]
fn pre_upgrade() {
    stable_save();
}

#[post_upgrade]
pub async fn post_upgrade() {
    stable_restore().await
}

#[update]
async fn get_trusted_origins() -> Vec<String> {
    CONF.with(|c| c.borrow().clone().origins)
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct CanisterIdRequest {
    #[serde(rename = "canister_id")]
    pub canister_id: Principal,
}

#[update]
async fn get_controllers() -> Vec<Principal> {
    let res: CallResult<(CanisterStatusResponse, )> = call(
        Principal::management_canister(),
        "canister_status",
        (CanisterIdRequest {
            canister_id: id(),
        }, ),
    ).await;

    res.unwrap().0.settings.controllers
}

#[update (guard = "is_caller_registered")]
async fn store_icrc1_canisters(canister_id: Vec<Principal>) -> VaultState {
    save_icrc1_canister(canister_id).await
}

#[query]
async fn get_trusted_origins_certified() -> CertifiedResponse {
    get_trusted_origins_cert()
}


#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}
