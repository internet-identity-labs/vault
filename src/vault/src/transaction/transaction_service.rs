use std::cell::RefCell;

use candid::CandidType;
use ic_cdk::{storage, trap};
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Approved, Rejected};
use crate::state::{define_state, get_current_state, get_vault_state, restore_state};
use crate::transaction::transaction::{Candid, ITransaction, TransactionCandid, TransactionIterator};

thread_local! {
    pub static TRANSACTIONS: RefCell<Vec<Box<dyn ITransaction >>> = RefCell::new(Default::default());
}

pub async fn execute_approved_transactions() {
    let mut unfinished_transactions = get_unfinished_transactions();
    unfinished_transactions.sort();
    let mut state = get_current_state();
    while let Some(mut trs) = unfinished_transactions.pop() {
        let state_before = trs.get_state().clone();
        trs.define_state();

        if trs.get_state().eq(&Approved) {
            state = trs.execute(state).await;
            //check that rejected transaction not in batch - if so reject whole batch and rollback the state
            if trs.get_state().eq(&Rejected) && trs.get_batch_uid().is_some() {
                let mut rejected_batch: Vec<Box<dyn ITransaction>> = get_all_transactions()
                    .into_iter()
                    .filter(|t| t.get_batch_uid() == trs.get_batch_uid())
                    .map(|mut t| {
                        t.set_state(Rejected);
                        t
                    })
                    .collect();
                rejected_batch.sort();
                //id of transaction before batch
                let id_before_reject = rejected_batch[0].get_id() - 1;
                state = get_vault_state(Some(id_before_reject)).await;
                restore_trs(rejected_batch);
            }
        }
        if !state_before.eq(trs.get_state()) {
            restore_transaction(trs);
        }
    }
    restore_state(state);
    TRANSACTIONS.with(|trss| {
        trss.borrow_mut().sort();
    });
}

fn restore_trs(trsss: Vec<Box<dyn ITransaction>>) {
    TRANSACTIONS.with(|transactions| {
        let mut transactions = transactions.borrow_mut();
        for trs in trsss {
            transactions.retain(|existing| existing.get_id() != trs.get_id());
            transactions.push(trs);
        }
    });
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
        let trss = utrs.borrow_mut();
        let trs = TransactionIterator::new(trss);
        trs.into_iter()
            .filter(|t| {
                ![TransactionState::Executed, TransactionState::Rejected].contains(t.get_state())
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

pub fn is_blocked<F>(f: F) -> bool
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
    let mut trs: Vec<Box<dyn ITransaction>> = mo.transactions
        .into_iter().map(|x| x.to_transaction())
        .collect();
    trs.sort_by(|a,b| -> std::cmp::Ordering {
        a.get_id().cmp(&b.get_id())
    });
    let state = define_state(trs.clone(), None).await;
    restore_state(state);
    TRANSACTIONS.with(|utrs| {
        utrs.borrow_mut();
        utrs.replace(trs)
    });
}


