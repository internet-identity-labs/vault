use std::cell::RefCell;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::ops::Deref;
use ic_cdk::{print, storage, trap};
use crate::enums::TransactionState;
use crate::enums::TransactionState::{Approved, Executed};
use crate::transaction::transaction::{TransactionNew, TransactionIterator, TrType, TransactionCandid};
use crate::transaction_service::Transaction;

use candid::CandidType;
use candid::types::{Serializer, Type, TypeId};
use serde::{Deserialize, Serialize};

thread_local! {
    pub static TRANSACTIONS: RefCell<Vec<Box<dyn TransactionNew>>> = RefCell::new(Default::default());
}

pub fn execute_approved_transactions() {
    let mut unfinished_transactions = get_unfinished_transactions();
    unfinished_transactions.sort();
    while let Some(mut trs) = unfinished_transactions.pop() {
        trs.define_state();
        if trs.get_state().eq(&Approved) {
            trs.execute();
            trs.set_state(Executed);
            TRANSACTIONS.with(|trss| {
                let mut transactions = trss.borrow_mut();
                transactions.retain(|existing| existing.get_id() != trs.get_id());
                transactions.push(trs);
            });
        }
    }
    TRANSACTIONS.with(|trss| {
        trss.borrow_mut().sort();
    });
}

pub fn restore_transaction(transaction: Box<dyn TransactionNew>) {
    TRANSACTIONS.with(|trss| {
        let mut transactions = trss.borrow_mut();
        transactions.retain(|existing| existing.get_id() != transaction.get_id());
        transactions.push(transaction);
        transactions.sort()
    });
}

pub fn store_transaction(transaction: Box<dyn TransactionNew>) {
    TRANSACTIONS.with(|utrs| {
        let mut borrowed = utrs.borrow_mut();
        borrowed.push(transaction);
    });
}

pub fn get_by_id(transaction_id: u64) -> Box<dyn TransactionNew> {
    TRANSACTIONS.with(|trss| {
        let transactions = trss.borrow();
        let trs = transactions.iter().find(|b| b.get_id() == transaction_id);

        match trs {
            None => trap("No such transaction"),
            Some(x) => x.clone(),
        }
    })
}

pub fn get_unfinished_transactions() -> Vec<Box<dyn TransactionNew>> {
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

pub fn get_all_transactions() -> Vec<Box<dyn TransactionNew>> {
    return TRANSACTIONS.with(|utrs| {
        let a = utrs.borrow_mut();
        let trs = TransactionIterator::new(a);
        trs.into_iter()
            .collect()
    });
}

pub fn is_blocked<F>(mut f: F) -> bool
    where
        F: FnMut(&Box<dyn TransactionNew>) -> bool,
{
    is_blocked_heap(f, get_unfinished_transactions())
}


pub fn is_blocked_heap<F>(mut f: F, trs: Vec<Box<dyn TransactionNew>>) -> bool
    where
        F: FnMut(&Box<dyn TransactionNew>) -> bool,
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

pub fn stable_restore() {
    let (mo, ): (Memory, ) = storage::stable_restore().unwrap();
    let trs : Vec<TransactionCandid> = mo.transactions;

    TRANSACTIONS.with(|utrs| {
        utrs.borrow_mut();
        utrs.replace(trs)
    });
    storage::stable_save((trs, )).unwrap();
}

