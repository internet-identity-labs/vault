use std::cell::RefMut;
use std::cmp::Ordering;

use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState::{Approved, Blocked, Pending};
use crate::transaction::basic_transaction::{Basic};
use crate::transaction::member::member_archive_transaction::MemberArchiveTransaction;
use crate::transaction::member::member_create_transaction::MemberCreateTransaction;
use crate::transaction::member::member_update_name_transaction::MemberUpdateNameTransaction;
use crate::transaction::member::member_update_role_transaction::MemberUpdateRoleTransaction;
use crate::transaction::quorum::quorum::{get_quorum, get_vault_admin_block_predicate};
use crate::transaction::quorum::quorum_transaction::QuorumTransaction;
use crate::transaction::transactions_service::is_blocked;
use crate::transaction::wallet::wallet_create_transaction::WalletCreateTransaction;
use crate::transaction::wallet::wallet_update_transaction::WalletUpdateNameTransaction;
use crate::transaction_service::Approve;

#[async_trait]
pub trait TransactionNew: Basic {
    fn define_state(&mut self) {
        if !is_blocked(|tr| {
            return get_vault_admin_block_predicate(tr);
        }) {
            if get_quorum().quorum <= self.get_common_mut().approves.len() as u64 {
                self.set_state(Approved)
            } else {
                self.set_state(Pending)
            }
        }
    }
    async fn execute(&self);
    fn handle_approve(&mut self, approve: Approve) {
        self.store_approve(approve);
        if self.get_common_mut().state.eq(&Pending) {
            if get_quorum().quorum <= self.get_common_mut().approves.len() as u64 {
                self.set_state(Approved)
            }
        }
    }
    fn to_candid(&self) -> TransactionCandid;
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

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, Eq, Hash, PartialEq)]
pub enum TrType {
    Quorum,
    MemberCreate,
    MemberUpdateName,
    MemberUpdateRole,
    MemberArchive,
    MemberUnarchive,
    WalletCreate,
    WalletUpdateName,
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
    MemberCreateTransaction(MemberCreateTransaction),
    MemberUpdateNameTransaction(MemberUpdateNameTransaction),
    MemberUpdateRoleTransaction(MemberUpdateRoleTransaction),
    MemberArchiveTransaction(MemberArchiveTransaction),
    WalletCreateTransaction(WalletCreateTransaction),
    WalletUpdateTransaction(WalletUpdateNameTransaction),
}

//TODO do something here
impl Candid for TransactionCandid {
    fn to_transaction(&self) -> Box<dyn TransactionNew> {
        match self {
            trs => {
                Box::new(trs.to_owned())
            }
        }
    }
}
