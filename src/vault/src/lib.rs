extern crate core;
#[macro_use]
extern crate maplit;

use std::collections::HashSet;
use candid::{export_service, Principal};
use ic_cdk::{call, caller, id, print, storage, trap};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_cdk::api::time;
use ic_cdk_macros::*;

use crate::enums::{Backup, TransactionState};
use crate::enums::ObjectState::Archived;
use crate::memory::{Conf, CONF};
use crate::policy_service::{Policy, PolicyType, ThresholdPolicy};
use crate::policy_service::Currency::ICP;
use crate::request::{CanisterIdRequest, PolicyRegisterRequest, TransactionApproveRequest, TransactionRegisterRequest, VaultMemberRequest, VaultRegisterRequest, WalletRegisterRequest};
use crate::security_service::{trap_if_not_permitted, verify_wallets};
use crate::transaction::quorum_transaction::QuorumTransaction;
use crate::transaction::transaction::{TransactionNew, TransactionCandid};
use crate::transaction::transactions_service::{execute_approved_transactions, get_all_transactions, get_unfinished_transactions, stable_restore, stable_save, TRANSACTIONS};
use crate::TransactionState::Approved;
use crate::transfer_service::transfer;
use crate::user_service::{get_or_new_by_caller, User};
use crate::util::{caller_to_address, to_array};
use crate::vault_service::{VaultRole};
use crate::wallet_service::{generate_address, Wallet};
use std::cell::RefCell;
use crate::transaction::members::{get_members, Member};
use crate::transaction::quorum;
use crate::transaction::quorum::Quorum;
use crate::transaction::transaction_approve_handler::handle_approve;
use crate::transaction::transaction_request_handler::{handle_transaction_request, TransactionRequestType};
use crate::transaction_service::{approve_transaction, Transaction};

mod user_service;
mod vault_service;
mod wallet_service;
mod policy_service;
mod transaction_service;
mod util;
mod enums;
mod security_service;
mod transfer_service;
mod request;
mod memory;
mod transaction;

thread_local! {
    pub static HEART_COUNT: RefCell<u8> = RefCell::new(0);
}

#[init]
fn init(conf: Option<Conf>) {
    match conf {
        None => {}
        Some(conf) => {
            CONF.with(|c| c.replace(conf));
        }
    };
}

#[update]
async fn request_transaction(transaction_request: TransactionRequestType) {
    handle_transaction_request(transaction_request)
}

#[update]
async fn execute() {
    execute_approved_transactions()
}

#[heartbeat]
fn execute_heartbeat() {
    HEART_COUNT.with(|qp| {
       let mut i = qp.borrow_mut();
        if (*i % 30) == 0 {
            print(i.to_string());
            execute_approved_transactions();
            *i = 0
        }
        *i += 1;
    });
}

#[query]
async fn get_transactions_all() -> Vec<TransactionCandid> {
    get_all_transactions()
        .into_iter()
        .map(|l| l.to_candid())
        .collect()
}


#[query]
async fn get_members_all() -> Vec<Member> {
    get_members()
}

#[update]
async fn config(conf: Conf) {
    trap_if_not_authenticated();
    CONF.with(|c| c.replace(conf));
}

#[query]
async fn get_config() -> Conf {
    trap_if_not_authenticated();
    CONF.with(|c| c.borrow().clone())
}

#[query]
async fn get_quorum() -> Quorum {
    quorum::get_quorum()
}

#[update]
async fn approve(request: TransactionApproveRequest) -> TransactionCandid {
    handle_approve(request.transaction_id, request.state)
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

#[update]
async fn sync_controllers() -> Vec<String> {
    let res: CallResult<(CanisterStatusResponse, )> = call(
        Principal::management_canister(),
        "canister_status",
        (CanisterIdRequest {
            canister_id: id(),
        }, ),
    ).await;

    let controllers = res.unwrap().0.settings.controllers;
    CONF.with(|c| c.borrow_mut().controllers.replace(controllers.clone()));
    controllers.iter().map(|x| x.to_text()).collect()
}

#[query]
async fn get_all_json(from: u32, to: u32, obj: Backup) -> String {
    trap_if_not_authenticated();
    memory::get_all_json(from, to, obj)
}

#[query]
async fn count(obj: Backup) -> u64 {
    trap_if_not_authenticated();
    memory::count(obj) as u64
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
pub fn post_upgrade() {
    stable_restore();
}

#[update]
async fn get_trusted_origins() -> Vec<String> {
    CONF.with(|c| c.borrow().clone().origins.unwrap())
}

fn trap_if_not_authenticated() {
    let princ = caller();
    match CONF.with(|c| c.borrow_mut().controllers.clone())
    {
        None => {
            trap("Unauthorised")
        }
        Some(controllers) => {
            if !controllers.contains(&princ) {
                trap("Unauthorised")
            }
        }
    }
}