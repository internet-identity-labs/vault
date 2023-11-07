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
use crate::transaction::quorum::Quorum;
use crate::transaction::quorum_transaction;
use crate::transaction::quorum_transaction::{QuorumTransaction, QuorumTransactionBuilder};
use crate::transaction::transactions_service::{get_all_transactions, get_by_id, get_unfinished_transactions, restore_transaction, store_transaction, TRANSACTIONS};
use crate::transaction_service::Approve;
use crate::util::caller_to_address;

pub trait TransactionNew<T : CandidType> {
    fn execute(&self);
    fn get_id(&self) -> u64;
    fn get_type(&self) -> &TrType;
    fn get_state(&self) -> &TransactionState;
    fn define_state(&mut self);
    fn set_state(&mut self, ts: TransactionState);
    fn handle_approve(&mut self, approve: Approve);
    fn to_candid(&self) -> T;
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
    Quorum
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

pub trait TransactionBuilder {
    type OutputType;
    fn define_initial_state(&mut self);
    fn define_initial_policy(&mut self);
    fn build(self) -> Self::OutputType;
}


pub fn handle_transaction_request(trr: TransactionRequestType) {
    match trr {
        TransactionRequestType::TransferTransaction(_) => {}
        TransactionRequestType::WalletCreateTransaction(_) => {}
        TransactionRequestType::QuorumStateTransaction(x) => {
            let mut trs = QuorumTransactionBuilder::init(x);
            trs.define_initial_state();
            trs.define_initial_policy();
            let mut transaction = trs.build();
            let approve = Approve {
                signer: caller_to_address(),
                created_date: time(),
                status: TransactionState::Approved,
            };
            transaction.handle_approve(approve);
            transaction.define_state();
            store_transaction(Box::new(transaction));
        }
    }
}

pub fn handle_approve(tr_id: u64, state: TransactionState) -> TransactionCandid {
    let mut trs = get_by_id(tr_id);
    let approve = Approve {
        signer: caller_to_address(),
        created_date: time(),
        status: state,
    };
    trs.handle_approve(approve);
    restore_transaction(trs.clone());
    trs.to_candid()
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct WalletCreateTransaction {
    pub network: Network,
    pub name: String,
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct TransferTransaction {
    pub wallet: String,
    pub to: String,
    pub amount: u64,
    pub block_index: Option<BlockIndex>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct
QuorumTransactionRequest {
    pub amount: u64,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionRequestType {
    TransferTransaction(TransferTransaction),
    WalletCreateTransaction(WalletCreateTransaction),
    QuorumStateTransaction(QuorumTransactionRequest),
}


#[test]
pub fn test() {
    let transaction_request = TransactionRequestType::QuorumStateTransaction(QuorumTransactionRequest { amount: 7 });
    handle_transaction_request(transaction_request);
    print!("");
}


#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionCandid {
    QuorumTransaction(QuorumTransaction),
}

//TODO do something here
impl Candid for TransactionCandid {
    fn to_transaction(&self) -> Box<dyn TransactionNew> {
        match self {
            TransactionCandid::QuorumTransaction(quorum_transaction) => {
                Box::new(quorum_transaction.to_owned())
            }
        }
    }
}
