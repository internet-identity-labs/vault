extern crate core;
extern crate maplit;

use std::cell::RefCell;

use candid::export_service;
use ic_cdk::api::time;
use ic_cdk_macros::*;

use crate::config::{Conf, CONF};
use crate::enums::{TransactionState, VaultRole};
use crate::state::{get_vault_state, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::member::member_create_transaction::MemberCreateTransaction;
use crate::transaction::transaction::TransactionCandid;
use crate::transaction::transaction_approve_handler::{Approve, handle_approve, TransactionApproveRequest};
use crate::transaction::transaction_request_handler::{handle_transaction_request, TransactionRequest};
use crate::transaction::transaction_service::{execute_approved_transactions, get_all_transactions, stable_restore, stable_save, store_transaction};
use crate::transaction::wallet::wallet::generate_address;
use crate::util::{caller_to_address, to_array};

mod util;
mod enums;
mod security_service;
mod transfer_service;
mod config;
mod transaction;
mod state;

thread_local! {
    pub static HEART_COUNT: RefCell<u8> = RefCell::new(0);
}

#[init]
async fn init(conf: Option<Conf>) {
    match conf {
        None => {}
        Some(conf) => {
            CONF.with(|c| c.replace(conf));
        }
    };

    let mut mc = MemberCreateTransaction::new(
        TransactionState::Approved, None, "6eee6eb5aeb5b94688a1f1831b246560797db6b0c80d8a004f64a0498519d632".to_string(), "Admin".to_string(), VaultRole::Admin,
    );
    mc.get_common_mut().approves.insert(Approve {
        signer: "6eee6eb5aeb5b94688a1f1831b246560797db6b0c80d8a004f64a0498519d632".to_string(),
        created_date: time(),
        status: TransactionState::Approved,
    });
    store_transaction(mc.clone_self());

    execute_approved_transactions().await
}

#[update]
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

#[heartbeat]
async fn execute_heartbeat() {
    let is_exec = HEART_COUNT.with(|qp| {
        let mut i = qp.borrow_mut();
        let should_execute = (*i % 30) == 0;
        *i += 1;
        if should_execute {
            *i += 0;
        }
        should_execute
    });

    if is_exec {
        execute_approved_transactions().await;
    }
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

#[update]
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

#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}


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
    CONF.with(|c| c.borrow().clone().origins.unwrap())
}
