use std::cell::RefCell;
use std::collections::VecDeque;

use candid::CandidType;
use candid::types::Serializer;
use ic_cdk::{print, storage, trap};
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Approved, Executed};
use crate::state::{get_current_state, restore_state, VaultState};
use crate::transaction::transaction::{Candid, ITransaction, TransactionCandid, TransactionIterator};

thread_local! {
    pub static TRANSACTIONS: RefCell<Vec<Box<dyn ITransaction >>> = RefCell::new(Default::default());
}

pub async fn execute_approved_transactions() {
    let mut unfinished_transactions = get_unfinished_transactions();
    unfinished_transactions.sort();
    let mut trs_to_restore: VecDeque<Box<dyn ITransaction>> = Default::default();
    let mut state = get_current_state();
    while let Some(mut trs) = unfinished_transactions.pop() {
        trs.define_state();
        print(trs.get_id().to_string());
        if trs.get_state().eq(&Approved) {
            state = trs.execute(state).await;
            // if trs.get_state().eq(&Rejected) && trs.get_batch_uid().is_some() {
            //     let a: Vec<Box<dyn ITransaction>> = unfinished_transactions.clone()
            //         .into_iter()
            //         .filter(|t| t.get_batch_uid().is_some())
            //         .map(|mut t| {
            //             t.set_state(Rejected);
            //             t
            //         })
            //         .collect();
            // }
            // trs_to_restore.push(trs);
            // unfinished_transactions.retain(|existing| existing.get_id() != trs.get_id());
            // unfinished_transactions.push(trs);
            TRANSACTIONS.with(|trsss| {
                let mut transactions = trsss.borrow_mut();
                transactions.retain(|existing| existing.get_id() != trs.get_id());
                transactions.push(trs);
            });
        }
    }

    // if (trs_to_restore.into_iter().any(|t| t.get_state().eq(&Rejected) && t.get_common_ref().batch_uid.is_some())) {
    //
    // }
    // restore_trs(trs_to_restore);
    restore_state(state);
    TRANSACTIONS.with(|trss| {
        trss.borrow_mut().sort();
    });
}

fn restore_trs(trsss: Vec<Box<dyn ITransaction>>) {
    TRANSACTIONS.with(|trss| {
        let mut transactions = trss.borrow_mut();
        for trs in trsss {
            transactions.retain(|existing| existing.get_id() != trs.get_id());
            transactions.push(trs);
        }
    });
}

pub async fn get_vault_state(tr_id: Option<u64>) -> VaultState {
    let mut state = VaultState::default();
    let mut transactions = get_all_transactions();
    transactions.sort();
    let mut sorted_trs = transactions
        .into_iter()
        .filter(|transaction| {
            let is_vault_state = transaction.is_vault_state();
            let is_executed = transaction.get_state().eq(&Executed);
            match tr_id {
                None => is_vault_state && is_executed,
                Some(tr) => is_vault_state && is_executed && transaction.get_id() <= tr,
            }
        })
        .collect::<Vec<Box<dyn ITransaction>>>();

    while let Some(mut trs) = sorted_trs.pop() {        //TODO
        state = trs.execute(state).await;
    }
    state
}

pub fn restore_transaction(transaction: Box<dyn ITransaction>) {
    TRANSACTIONS.with(|trss| {
        let mut transactions = trss.borrow_mut();
        transactions.retain(|existing| existing.get_id() != transaction.get_id());
        transactions.push(transaction);
        transactions.sort()
    });
}

pub fn store_transaction(transaction: Box<dyn ITransaction>) {
    TRANSACTIONS.with(|utrs| {
        let mut borrowed = utrs.borrow_mut();
        borrowed.push(transaction);
    });
}

pub fn get_by_id(transaction_id: u64) -> Box<dyn ITransaction> {
    TRANSACTIONS.with(|trss| {
        let transactions = trss.borrow();
        let trs = transactions.iter().find(|b| b.get_id() == transaction_id);

        match trs {
            None => trap("No such transaction"),
            Some(x) => x.clone(),
        }
    })
}

pub fn get_unfinished_transactions() -> Vec<Box<dyn ITransaction>> {
    return TRANSACTIONS.with(|utrs| {
        let a = utrs.borrow_mut();
        let trs = TransactionIterator::new(a);
        trs.into_iter()
            .filter(|t| {
                ![TransactionState::Executed, TransactionState::Rejected, TransactionState::Canceled].contains(t.get_state())
            })
            .collect()
    });
}

pub fn get_all_transactions() -> Vec<Box<dyn ITransaction>> {
    return TRANSACTIONS.with(|utrs| {
        let a = utrs.borrow_mut();
        let trs = TransactionIterator::new(a);
        trs.into_iter()
            .collect()
    });
}

pub fn is_blocked<F>(mut f: F) -> bool
                     where
                         F: FnMut(&Box<dyn ITransaction>) -> bool,
{
    is_blocked_line(f, get_unfinished_transactions())
}


pub fn is_blocked_line<F>(mut f: F, trs: Vec<Box<dyn ITransaction>>) -> bool
                          where
                              F: FnMut(&Box<dyn ITransaction>) -> bool,
{
    trs
        .into_iter()
        .any(|tr| f(&tr))
}

pub fn get_id() -> u64 {
    return TRANSACTIONS.with(|utrs| {
        utrs.borrow_mut().len() as u64 + 1
    });
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    transactions: Vec<TransactionCandid>,
}


pub fn stable_save() {
    let trs: Vec<TransactionCandid> = TRANSACTIONS.with(|trss| {
        let a = trss.borrow();
        a.iter()
            .map(|a| a.to_candid())
            .collect()
    });
    let mem = Memory {
        transactions: trs,
    };
    storage::stable_save((mem, )).unwrap();
}

pub async fn stable_restore() {
    let (mo, ): (Memory, ) = storage::stable_restore().unwrap();
    let mut state = VaultState::default();
    let mut trs: Vec<Box<dyn ITransaction>> = mo.transactions
        .into_iter().map(|x| x.to_transaction()).collect();
    trs.sort();
    for transaction in &trs {
        if transaction.is_vault_state() && transaction.get_state().eq(&Executed) {
            //TODO
            state = transaction.clone().execute(state).await;
        }
    }
    restore_state(state);
    TRANSACTIONS.with(|utrs| {
        utrs.borrow_mut();
        utrs.replace(trs)
    });
}


