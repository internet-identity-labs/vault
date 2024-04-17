use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_cdk::{storage, trap};
use serde::{Deserialize, Serialize};
use nfid_certified::update_trusted_origins;
use crate::config::{Conf, CONF};
use crate::state::{ICRC1};

use crate::enums::TransactionState::{Approved, Executed, Failed, Purged, Rejected};
use crate::execute;
use crate::state::{define_state, get_current_state, get_icrc1_canisters, get_vault_state, restore_state};
use crate::transaction::transaction::{Candid, ITransaction, TransactionCandid, TransactionIterator};

thread_local! {
    pub static TRANSACTIONS: RefCell<Vec<Box<dyn ITransaction >>> = RefCell::new(Default::default());
}

pub async fn execute_approved_transactions() {
    let mut unfinished_transactions = get_unfinished_transactions();
    unfinished_transactions.sort();
    let mut state = get_current_state();
    let mut i = unfinished_transactions.len();
    while 0 < i {
        let mut trs = unfinished_transactions[i - 1].clone();
        let state_before = trs.get_state().clone();
        trs.define_state();
        let mut new_circle = false;
        if trs.get_state().eq(&Approved) {
            state = trs.execute(state).await;
            trs.update_modified_date();
            let current_state = trs.get_state();
            //check that rejected/failed transaction not in batch - if so reject whole batch and rollback the state
            if (current_state.eq(&Rejected) || current_state.eq(&Failed)) && trs.get_batch_uid().is_some() {
                let mut rejected_batch: Vec<Box<dyn ITransaction>> = get_all_transactions()
                    .into_iter()
                    .filter(|t| t.get_batch_uid() == trs.get_batch_uid())
                    .map(|mut t| {
                        t.set_state(current_state.clone());
                        t
                    })
                    .collect();
                rejected_batch.sort_by(|a, b| a.get_id().cmp(&b.get_id()));
                //id of transaction before batch
                let id_before_reject = rejected_batch[0].get_id() - 1;
                state = get_vault_state(Some(id_before_reject)).await;
                restore_trs(rejected_batch);
                unfinished_transactions = get_unfinished_transactions();
                unfinished_transactions.sort();
                new_circle = true;
            }
        }
        if !state_before.eq(trs.get_state()) {
            restore_transaction(trs);
        }
        if new_circle {
            i = unfinished_transactions.len();
        } else {
            i -= 1;
        }
    }
    restore_state(state);
}

fn restore_trs(trs_to_restore: Vec<Box<dyn ITransaction>>) {
    TRANSACTIONS.with(|transactions| {
        let mut transactions = transactions.borrow_mut();
        for mut trs in trs_to_restore {
            transactions.retain(|existing| existing.get_id() != trs.get_id());
            trs.update_modified_date();
            transactions.push(trs);
        }
    })
}

pub fn restore_transaction(mut transaction: Box<dyn ITransaction>) {
    TRANSACTIONS.with(|trss| {
        let mut transactions = trss.borrow_mut();
        transaction.update_modified_date();
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
        let mut transactions: Vec<Box<dyn ITransaction>> = trs.into_iter()
            .filter(|t| {
                ![Executed, Rejected,  Failed, Purged].contains(t.get_state())
            })
            .collect();
        transactions.sort();
        transactions
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
    config: Conf,
    icrc1_canisters: Option<Vec<Principal>>,
}


pub fn stable_save() {
    let trs: Vec<TransactionCandid> = TRANSACTIONS.with(|trss| {
        let a = trss.borrow();
        a.iter()
            .map(|a| a.to_candid())
            .collect()
    });
    let conf: Conf = CONF.with(|conf| {
        let a = conf.borrow();
        a.clone()
    });
    let icrc1_canisters = get_icrc1_canisters();
    let mem = Memory {
        config: conf,
        transactions: trs,
        icrc1_canisters: Some(icrc1_canisters)
    };
    storage::stable_save((mem, )).unwrap();
}

pub async fn stable_restore() {
    let (mo, ): (Memory, ) = storage::stable_restore().unwrap();
    CONF.with(|conf| {
        conf.borrow_mut();
        update_trusted_origins(mo.config.origins.clone());
        conf.replace(mo.config.clone())
    });
    let mut trs: Vec<Box<dyn ITransaction>> = mo.transactions
        .into_iter()
        .map(|x| x.to_transaction())
        .collect();
    trs.sort_by(|a, b| -> std::cmp::Ordering {
        a.get_id().cmp(&b.get_id())
    });
    let icrc1_canisters_opt = mo.icrc1_canisters.clone();
    ICRC1.with(|icrc1| {
        icrc1.borrow_mut();
        icrc1.replace( match icrc1_canisters_opt {
            None => {
                Vec::default()
            }
            Some(icrc1) => {
                icrc1
            }
        } );
    });
    let state = define_state(trs.clone(), None).await;
    restore_state(state);
    TRANSACTIONS.with(|utrs| {
        utrs.borrow_mut();
        utrs.replace(trs)
    });
    execute().await;
}


