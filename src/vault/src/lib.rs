extern crate core;
extern crate maplit;

use std::cell::RefCell;

use candid::export_service;
use ic_cdk::api::time;
use ic_cdk_macros::*;

use crate::config::{Conf, CONF};
use crate::enums::TransactionState;
use crate::request::TransactionApproveRequest;
use crate::state::{get_vault_state, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::member::member_create_transaction::MemberCreateTransaction;
use crate::transaction::transaction::TransactionCandid;
use crate::transaction::transaction_approve_handler::handle_approve;
use crate::transaction::transaction_request_handler::{handle_transaction_request, TransactionRequest};
use crate::transaction::transaction_service::{execute_approved_transactions, get_all_transactions, stable_restore, stable_save, store_transaction};
use crate::transaction_service::Approve;
use crate::util::to_array;
use crate::vault_service::VaultRole;

mod vault_service;
mod wallet_service;
mod policy_service;
mod transaction_service;
mod util;
mod enums;
mod security_service;
mod transfer_service;
mod request;
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

// #[heartbeat]
// async fn execute_heartbeat() {
//     let is_exec = HEART_COUNT.with(|qp| {
//         let mut i = qp.borrow_mut();
//         let should_execute = (*i % 5) == 0;
//         *i += 1;
//         if should_execute {
//             *i += 0;
//         }
//         should_execute
//     });
//
//     if is_exec {
//         execute_approved_transactions().await;
//     }
// }

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

//
//
// #[update]
// async fn register_wallet(request: WalletRegisterRequest) -> Wallet {
//     trap_if_not_permitted(vec![VaultRole::Admin]);
//     let address = generate_address().await;
//     let new_wallet = wallet_service::new_and_store(request.name, address);
//     new_wallet
// }
//
// #[update]
// async fn update_wallet(wallet: Wallet) -> Wallet {
//     let old = wallet_service::get_by_uid(&wallet.uid);
//     trap_if_not_permitted(vec![VaultRole::Admin]);
//     wallet_service::update(wallet)
// }
//
// #[update]
// async fn register_policy(request: PolicyRegisterRequest) -> Policy {
//     trap_if_not_permitted(vec![VaultRole::Admin]);
//     // verify_wallets(request.vault_id, &request.policy_type);
//     let policy = policy_service::register_policy( request.policy_type);
//     policy
// }
//
// #[query]
// async fn get_wallets(vault_id: u64) -> Vec<Wallet> {
//     trap_if_not_permitted(vec![VaultRole::Admin, VaultRole::Member]);
//     wallet_service::get_wallets()
// }
//
//
// #[query]
// async fn get_policies(vault_id: u64) -> Vec<Policy> {
//     trap_if_not_permitted(vec![VaultRole::Admin, VaultRole::Member]);
//     policy_service::get()
// }
//
// #[update]
// async fn update_policy(policy: Policy) -> Policy {
//     let old = policy_service::get_by_id(policy.id);
//     trap_if_not_permitted(vec![VaultRole::Admin]);
//     // verify_wallets(old.vault, &policy.policy_type);
//     policy_service::update_policy(policy)
// }

// #[update]
// async fn register_transaction(request: TransactionRegisterRequest) -> Transaction {
//     let wallet = wallet_service::get_by_uid(&request.wallet_id);
//     // let vaults = vault_service::get(wallet.vaults.clone());
//     // let vault = vaults.first().unwrap(); //for now one2one
//     trap_if_not_permitted(vec![VaultRole::Admin, VaultRole::Member]);
//     let policy = policy_service::define_correct_policy(request.amount, &wallet.uid);
//     let members = 0;
//     let transaction = transaction_service::register_transaction(request.amount.clone(), request.address, wallet.uid, policy, members);
//     let approve = TransactionApproveRequest {
//         transaction_id: transaction.id,
//         state: TransactionState::Approved,
//     };
//     approve_transaction(approve).await
// }

// #[query]
// async fn get_transactions() -> Vec<Transaction> {
//     let tr_owner = user_service::get_or_new_by_caller();
//     return transaction_service::get_all();
// }

// #[update]
// async fn approve_transaction(request: TransactionApproveRequest) -> Transaction {
//     let ts = transaction_service::get_by_id(request.transaction_id);
//     trap_if_not_permitted( vec![VaultRole::Admin, VaultRole::Member]);
//     let mut approved_transaction = transaction_service::approve_transaction(ts, request.state);
//     if Approved.eq(&approved_transaction.state) {
//         let result = transfer(approved_transaction.amount,
//                               approved_transaction.to.clone(),
//                               approved_transaction.from.clone()).await;
//         match result {
//             Ok(block) => {
//                 approved_transaction.block_index = Some(block);
//                 transaction_service::store_transaction(approved_transaction.clone());
//             }
//             Err(e) => {
//                 approved_transaction.state = TransactionState::Rejected;
//                 approved_transaction.memo = Some(e);
//                 transaction_service::store_transaction(approved_transaction.clone());
//             }
//         }
//     }
//     approved_transaction
// }


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
