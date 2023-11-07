use std::cell::RefCell;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::collections::HashSet;
use ic_cdk::api::time;
use sha2::digest::typenum::private::IsEqualPrivate;
use crate::enums::{ObjectState, TransactionState};
use crate::transaction::quorum_transaction::QuorumTransaction;
use crate::transaction::transaction::{TransactionNew, TrType};
use candid::CandidType;
use serde::{Deserialize, Serialize};

thread_local! {
    pub static QUORUM: RefCell<Quorum> = RefCell::new(Quorum::default());
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Quorum {
    pub quorum: u64,
    pub modified_date: u64, //todo
}

impl Quorum {
    fn default() -> Self {
        Quorum {
            quorum: 1,
            modified_date: time(),
        }
    }
}

pub fn get_quorum() -> Quorum {
  QUORUM.with(|qp| {
        qp.borrow().clone()
    })
}

pub fn update_quorum(mut q: Quorum) {
  QUORUM.with(|qp| {
      q.modified_date = time();
      qp.replace(q);
    })
}


