use std::collections::{HashSet};
use std::hash::{Hash, Hasher};

use candid::CandidType;
use ic_cdk::trap;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};

use crate::{caller_to_address, Policy, PolicyType};
use crate::enums::TransactionState;
use crate::memory::TRANSACTIONS;
use crate::policy_service::Currency;
use crate::TransactionState::{Approved, Canceled, Pending, Rejected};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Transaction {
    pub id: u64,
    pub from: String,
    pub vault_id: u64,
    pub to: String,
    pub approves: HashSet<Approve>,
    pub amount: u64,
    pub state: TransactionState,
    pub policy_id: u64,
    pub block_index: Option<BlockIndex>,
    pub amount_threshold: u64,
    pub currency: Currency,
    pub member_threshold: u8,
    pub owner: String,
    pub created_date: u64,
    pub modified_date: u64,
    pub memo: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq)]
pub struct Approve {
    pub signer: String,
    pub created_date: u64,
    pub status: TransactionState,
}

impl PartialEq for Approve {
    fn eq(&self, other: &Self) -> bool {
        self.signer.eq(&other.signer)
    }
}

impl Hash for Approve {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.signer.hash(state)
    }
}

pub fn register_transaction(amount: u64, to: String, wallet: String, policy: Policy, members: usize) -> Transaction {
    let amount_threshold: u64;
    let member_threshold: u8;

    match policy.policy_type.clone() {
        PolicyType::ThresholdPolicy(tp) => {
            amount_threshold = tp.amount_threshold;
            member_threshold = match tp.member_threshold {
                None => { members as u8 } //means required all
                Some(threshold) => { threshold }
            };
        }
    }
    TRANSACTIONS.with(|transactions| {
        let mut ts = transactions.borrow_mut();

        let t: Transaction = Transaction {
            id: (ts.len() + 1) as u64,
            from: wallet,
            vault_id: policy.vault,
            to,
            approves: hashset! {},
            amount,
            state: TransactionState::Pending,
            policy_id: policy.id,
            block_index: None,
            amount_threshold,
            currency: Currency::ICP,
            member_threshold,
            owner: caller_to_address(),
            created_date: ic_cdk::api::time(),
            modified_date: ic_cdk::api::time(),
            memo: None,
        };
        ts.insert(t.id, t.clone());
        t
    })
}

pub fn approve_transaction(mut transaction: Transaction, state: TransactionState) -> Transaction {
    if !transaction.state.eq(&Pending) {
        trap("Transaction not pending")
    }

    transaction.approves.replace(
        Approve {
            signer: caller_to_address(),
            created_date: ic_cdk::api::time(),
            status: state.clone(),
        });

    match state {
        Approved => {
            if is_transaction_approved(&transaction) {
                transaction.state = Approved
            }
        }
        Rejected => {
            transaction.state = Rejected
        }
        Pending => {
            trap("Incorrect state")
        }
        Canceled => {
            transaction.state = Canceled
        }
    }

    transaction.modified_date = ic_cdk::api::time();


    store_transaction(transaction.clone());
    transaction
}

pub fn is_transaction_approved(transaction: &Transaction) -> bool {
    if !transaction.state.eq(&Pending) {
        return false;
    }
    return transaction.approves.clone()
        .into_iter()
        .filter(|l| l.status.eq(&TransactionState::Approved))
        .count() as u8 >= transaction.member_threshold;
}

pub fn store_transaction(transaction: Transaction) -> Option<Transaction> {
    TRANSACTIONS.with(|transactions| {
        return transactions.borrow_mut().insert(transaction.id, transaction);
    })
}

pub fn get_all(vaults: HashSet<u64>) -> Vec<Transaction> {
    TRANSACTIONS.with(|transactions| {
        return transactions.borrow().iter()
            .map(|a| a.1.clone())
            .filter(|t| vaults.contains(&t.vault_id))
            .collect();
    })
}

pub fn migrate_all(vaults: HashSet<u64>, address_from: String, address_to: String) {
    TRANSACTIONS.with(|transactions| {
        let mut tr_to_migrate: Vec<Transaction> = transactions.borrow_mut().iter()
            .map(|a| a.1.clone())
            .filter(|t| vaults.contains(&t.vault_id))
            .filter(|t| t.owner.eq(&address_from))
            .collect();
        for mut tr in tr_to_migrate {
            tr.owner = address_to.clone();
            transactions.borrow_mut().insert(tr.id, tr);
        }
    })
}


pub fn get_by_id(id: u64) -> Transaction {
    TRANSACTIONS.with(|transactions| {
        match transactions.borrow_mut().get(&id) {
            None => {
                trap("Nonexistent key error")
            }
            Some(transaction) => {
                transaction.clone()
            }
        }
    })
}