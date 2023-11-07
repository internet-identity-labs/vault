use std::cell::{Ref, RefCell, RefMut};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::collections::HashSet;

use candid::CandidType;
use ic_cdk::api::time;
use ic_cdk::print;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};

use crate::enums::{Network, TransactionState};
use crate::transaction::member_transaction::{MemberTransaction, MemberTransactionBuilder};
use crate::transaction::members::{get_member_by_id, Member};
use crate::transaction::quorum::Quorum;
use crate::transaction::quorum_transaction;
use crate::transaction::quorum_transaction::{QuorumTransaction, QuorumTransactionBuilder};
use crate::transaction::transaction_request_handler::handle_transaction_request;
use crate::transaction::transactions_service::{get_all_transactions, get_by_id, get_unfinished_transactions, restore_transaction, store_transaction, TRANSACTIONS};
use crate::transaction_service::Approve;
use crate::util::caller_to_address;
use crate::vault_service::VaultRole;

pub trait TransactionNew {
    fn execute(&self);
    fn get_id(&self) -> u64;
    fn get_type(&self) -> &TrType;
    fn get_state(&self) -> &TransactionState;
    fn define_state(&mut self);
    fn set_state(&mut self, ts: TransactionState);
    fn handle_approve(&mut self, approve: Approve);
    fn to_candid(&self) -> TransactionCandid;
    fn clone_self(&self) -> Box<dyn TransactionNew>;
}

pub trait Candid {
    fn to_transaction(&self) -> Box<dyn TransactionNew>;
}

impl Clone for Box<dyn TransactionNew> {
    fn clone(&self) -> Box<dyn TransactionNew> {
        self.as_ref().clone_self()
    }
}

pub trait TransactionClone: TransactionNew + Clone {}

impl<T> TransactionClone for T where T: TransactionNew + Clone {}

pub struct TransactionIterator<'a> {
    inner: RefMut<'a, Vec<Box<dyn TransactionNew>>>,
    index: usize,
}

impl<'a> TransactionIterator<'a> {
    pub fn new(trs: RefMut<'a, Vec<Box<dyn TransactionNew>>>) -> Self {
        TransactionIterator { inner: trs, index: 0 }
    }
}

//TODO use reference
impl<'a> Iterator for TransactionIterator<'a> {
    type Item = Box<dyn TransactionNew>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.inner.len() {
            // Clone the item at the current index
            let cloned_item = self.inner[self.index].clone();
            self.index += 1;
            Some(cloned_item)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum TrType {
    Quorum,
    MemberCreate,
    MemberUpdateName,
    MemberUpdateRole,
    MemberArchive,
    MemberUnArchive,
}

impl Eq for dyn TransactionNew {}


impl PartialEq for dyn TransactionNew {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Ord for dyn TransactionNew {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_id().cmp(&other.get_id())
    }
}

impl PartialOrd for dyn TransactionNew {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.get_id().cmp(&other.get_id()))
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionCandid {
    QuorumTransaction(QuorumTransaction),
    MemberTransaction(MemberTransaction),
}

//TODO do something here
impl Candid for TransactionCandid {
    fn to_transaction(&self) -> Box<dyn TransactionNew> {
        match self {
            TransactionCandid::QuorumTransaction(trs) => {
                Box::new(trs.to_owned())
            }
            TransactionCandid::MemberTransaction(trs) => {
                Box::new(trs.to_owned())
            }
        }
    }
}
