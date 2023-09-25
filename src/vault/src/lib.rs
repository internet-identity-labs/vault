extern crate core;
#[macro_use]
extern crate maplit;

use candid::{candid_method, export_service, Principal};
use ic_cdk::{call, caller, id, trap};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_cdk::export::candid;
use ic_cdk_macros::*;

use crate::enums::{Backup, TransactionState};
use crate::memory::{Conf, CONF};
use crate::policy_service::{Policy, PolicyType, ThresholdPolicy};
use crate::policy_service::Currency::ICP;
use crate::request::{CanisterIdRequest, PolicyRegisterRequest, TransactionApproveRequest, TransactionRegisterRequest, VaultMemberRequest, VaultRegisterRequest, WalletRegisterRequest};
use crate::security_service::{trap_if_not_permitted, verify_wallets};
use crate::transaction_service::Transaction;
use crate::TransactionState::Approved;
use crate::transfer_service::transfer;
use crate::user_service::{get_or_new_by_caller, migrate_to_address, User};
use crate::util::{caller_to_address, caller_to_address_legacy, to_array};
use crate::vault_service::{Vault, VaultRole};
use crate::wallet_service::{generate_address, Wallet};

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

#[init]
#[candid_method(init)]
fn init(conf: Option<Conf>) {
    match conf {
        None => {}
        Some(conf) => {
            CONF.with(|c| c.replace(conf));
        }
    };
}

#[update]
#[candid_method(update)]
async fn config(conf: Conf) {
    trap_if_not_authenticated();
    CONF.with(|c| c.replace(conf));
}

#[query]
#[candid_method(query)]
async fn get_config() -> Conf {
    trap_if_not_authenticated();
    CONF.with(|c| c.borrow().clone())
}

#[update]
#[candid_method(update)]
async fn register_vault(request: VaultRegisterRequest) -> Vault {
    let mut user = user_service::get_or_new_by_caller();
    let mut vault = vault_service::register(user.address.clone(), request.name, request.description);
    let threshold_policy = ThresholdPolicy {
        amount_threshold: 0,
        currency: ICP,
        member_threshold: None,
        wallets: None,
    };
    let default_pt = PolicyType::ThresholdPolicy(threshold_policy);
    let default_policy = policy_service::register_policy(vault.id, default_pt);
    vault.policies.insert(default_policy.id);
    user.vaults.insert(vault.id);
    user_service::restore(user);
    vault_service::restore(&vault)
}

#[update]
#[candid_method(update)]
async fn update_vault(vault: Vault) -> Vault {
    trap_if_not_permitted(vault.id, vec![VaultRole::Admin]);
    vault_service::update(vault)
}

#[query]
#[candid_method(query)]
async fn get_vaults() -> Vec<Vault> {
    let vault_ids = user_service::get_or_new_by_caller().vaults;
    vault_service::get(vault_ids)
}

#[update]
#[candid_method(update)]
async fn store_member(request: VaultMemberRequest) -> Vault {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]);
    let mut user = user_service::get_or_new_by_address(request.address);
    let vault = vault_service::add_vault_member(request.vault_id, &user, request.role, request.name, request.state);
    user.vaults.insert(vault.id.clone());
    user_service::restore(user);
    vault_service::restore(&vault)
}

#[update]
#[candid_method(update)]
async fn register_wallet(request: WalletRegisterRequest) -> Wallet {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]);
    let mut vault = vault_service::get_by_id(&request.vault_id);
    let address = generate_address().await;
    let new_wallet = wallet_service::new_and_store(request.name, request.vault_id, address);
    vault.wallets.insert(new_wallet.uid.clone());
    vault_service::restore(&vault);
    new_wallet
}

#[update]
#[candid_method(update)]
async fn update_wallet(wallet: Wallet) -> Wallet {
    let old = wallet_service::get_by_uid(&wallet.uid);
    for vault_id in &old.vaults {
        trap_if_not_permitted(*vault_id, vec![VaultRole::Admin]);
    }
    wallet_service::update(wallet)
}

#[update]
#[candid_method(update)]
async fn register_policy(request: PolicyRegisterRequest) -> Policy {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]);
    verify_wallets(request.vault_id, &request.policy_type);
    let mut vault = vault_service::get_by_id(&request.vault_id);
    let policy = policy_service::register_policy(request.vault_id, request.policy_type);
    vault.policies.insert(policy.id);
    vault_service::restore(&vault);
    policy
}

#[query]
#[candid_method(query)]
async fn get_wallets(vault_id: u64) -> Vec<Wallet> {
    trap_if_not_permitted(vault_id, vec![VaultRole::Admin, VaultRole::Member]);
    let vault = vault_service::get(hashset![vault_id]);
    wallet_service::get_wallets(vault[0].wallets.clone())
}


#[query]
#[candid_method(query)]
async fn get_policies(vault_id: u64) -> Vec<Policy> {
    trap_if_not_permitted(vault_id, vec![VaultRole::Admin, VaultRole::Member]);
    let vault = vault_service::get(hashset![vault_id]);
    policy_service::get(vault[0].policies.clone())
}

#[update]
#[candid_method(update)]
async fn update_policy(policy: Policy) -> Policy {
    let old = policy_service::get_by_id(policy.id);
    trap_if_not_permitted(old.vault, vec![VaultRole::Admin]);
    verify_wallets(old.vault, &policy.policy_type);
    policy_service::update_policy(policy)
}

#[update]
#[candid_method(update)]
async fn register_transaction(request: TransactionRegisterRequest) -> Transaction {
    let wallet = wallet_service::get_by_uid(&request.wallet_id);
    let vaults = vault_service::get(wallet.vaults.clone());
    let vault = vaults.first().unwrap(); //for now one2one
    trap_if_not_permitted(vault.id, vec![VaultRole::Admin, VaultRole::Member]);
    let policy = policy_service::define_correct_policy(vault.policies.clone(), request.amount, &wallet.uid);
    let transaction = transaction_service::register_transaction(request.amount, request.address, wallet.uid, policy, vault.members.len());
    let approve = TransactionApproveRequest {
        transaction_id: transaction.id,
        state: TransactionState::Approved,
    };
    approve_transaction(approve).await
}

#[query]
#[candid_method(query)]
async fn get_transactions() -> Vec<Transaction> {
    let tr_owner = user_service::get_or_new_by_caller();
    return transaction_service::get_all(tr_owner.vaults);
}

#[update]
#[candid_method(update)]
async fn approve_transaction(request: TransactionApproveRequest) -> Transaction {
    let ts = transaction_service::get_by_id(request.transaction_id);
    trap_if_not_permitted(ts.vault_id, vec![VaultRole::Admin, VaultRole::Member]);
    let mut approved_transaction = transaction_service::approve_transaction(ts, request.state);
    if Approved.eq(&approved_transaction.state) {
        let result = transfer(approved_transaction.amount,
                              approved_transaction.to.clone(),
                              approved_transaction.from.clone()).await;
        match result {
            Ok(block) => {
                approved_transaction.block_index = Some(block);
                transaction_service::store_transaction(approved_transaction.clone());
            }
            Err(e) => {
                approved_transaction.state = TransactionState::Rejected;
                approved_transaction.memo = Some(e);
                transaction_service::store_transaction(approved_transaction.clone());
            }
        }
    }
    approved_transaction
}

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
    memory::pre_upgrade()
}


#[post_upgrade]
pub fn post_upgrade() {
    memory::post_upgrade()
}

#[update]
async fn migrate_user(to_address: String) -> bool {
    let conf =  CONF.with(|c| c.borrow().clone());
    let mut from_address = caller_to_address_legacy();
    //we can not emulate test user with legacy account type
    if conf.is_test_env.is_some() && conf.is_test_env.unwrap().eq(&true) {
        from_address = caller_to_address();
    }
    if !user_service::has_vaults(&from_address) {
        return true
    }
    let user = user_service::get_or_new_by_address(from_address.clone());
    let vault_ids = user.vaults;
    transaction_service::migrate_all(vault_ids.clone(), from_address.clone(), to_address.clone());
    vault_service::migrate_all(vault_ids, from_address.clone(), to_address.clone());
    migrate_to_address(from_address, to_address)
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