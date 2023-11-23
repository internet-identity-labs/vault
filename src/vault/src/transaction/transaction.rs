use std::cell::RefMut;
use std::cmp::Ordering;

use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState::{Approved, Pending};
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::member::member_create_transaction::MemberCreateTransaction;
use crate::transaction::member::member_remove_transaction::MemberRemoveTransaction;
use crate::transaction::member::member_update_name_transaction::MemberUpdateNameTransaction;
use crate::transaction::member::member_update_role_transaction::MemberUpdateRoleTransaction;
use crate::transaction::policy::policy_create_transaction::PolicyCreateTransaction;
use crate::transaction::policy::policy_remove_transaction::PolicyRemoveTransaction;
use crate::transaction::policy::policy_update_transaction::PolicyUpdateTransaction;
use crate::transaction::quorum::quorum::get_quorum;
use crate::transaction::quorum::quorum_transaction::QuorumUpdateTransaction;
use crate::transaction::transaction_builder::get_vault_state_block_predicate;
use crate::transaction::transactions_service::is_blocked;
use crate::transaction::wallet::wallet_create_transaction::WalletCreateTransaction;
use crate::transaction::wallet::wallet_update_transaction::WalletUpdateNameTransaction;
use crate::transaction_service::Approve;

#[async_trait]
pub trait ITransaction: BasicTransaction {
    fn define_state(&mut self) {
        if !is_blocked(|tr| {
            return get_vault_state_block_predicate(tr) && !tr.get_id().eq(&self.get_id());
        }) {
            let threshold = self.define_threshold();
            if threshold <= self.get_common_mut().approves.len() as u8 {
                self.set_state(Approved)
            } else {
                self.set_state(Pending)
            }
        }
    }
    fn define_threshold(&mut self) -> u8 {
        if self.get_threshold().is_none() {
            self.set_threshold(get_quorum().quorum)
        }
        self.get_threshold().unwrap()
    }
    async fn execute(&mut self, state: VaultState) -> VaultState;
    fn handle_approve(&mut self, approve: Approve) {
        self.store_approve(approve);
        if self.get_common_mut().state.eq(&Pending) {
            if get_quorum().quorum <= self.get_common_mut().approves.len() as u8 {
                self.set_state(Approved)
            }
        }
    }
    fn to_candid(&self) -> TransactionCandid;
}


impl Clone for Box<dyn ITransaction> {
    fn clone(&self) -> Box<dyn ITransaction> {
        self.as_ref().clone_self()
    }
}

pub trait TransactionClone: ITransaction + Clone {}

impl<T> TransactionClone for T where T: ITransaction + Clone {}

pub struct TransactionIterator<'a> {
    inner: RefMut<'a, Vec<Box<dyn ITransaction>>>,
    index: usize,
}

impl<'a> TransactionIterator<'a> {
    pub fn new(trs: RefMut<'a, Vec<Box<dyn ITransaction>>>) -> Self {
        TransactionIterator { inner: trs, index: 0 }
    }
}

//TODO use a reference
impl<'a> Iterator for TransactionIterator<'a> {
    type Item = Box<dyn ITransaction>;

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


impl Eq for dyn ITransaction {}


impl PartialEq for dyn ITransaction {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Ord for dyn ITransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        other.get_id().cmp(&self.get_id())
    }
}

impl PartialOrd for dyn ITransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.get_id().cmp(&self.get_id()))
    }
}

//todo do we need this one?
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, Eq, Hash, PartialEq)]
pub enum TrType {
    QuorumUpdate,
    MemberCreate,
    MemberUpdateName,
    MemberUpdateRole,
    MemberRemove,
    WalletCreate,
    WalletUpdateName,
    PolicyUpdate,
    PolicyCreate,
    PolicyRemove,
    Transfer,
}


#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionCandid {
    QuorumUpdateTransactionV(QuorumUpdateTransaction),
    MemberCreateTransactionV(MemberCreateTransaction),
    MemberUpdateNameTransactionV(MemberUpdateNameTransaction),
    MemberUpdateRoleTransactionV(MemberUpdateRoleTransaction),
    MemberRemoveTransactionV(MemberRemoveTransaction),
    WalletCreateTransactionV(WalletCreateTransaction),
    WalletUpdateTransactionV(WalletUpdateNameTransaction),
    PolicyCreateTransactionV(PolicyCreateTransaction),
    PolicyUpdateTransactionV(PolicyUpdateTransaction),
    PolicyRemoveTransactionV(PolicyRemoveTransaction),
    // TransferTransaction(TransferTransaction),
}

pub trait Candid {
    fn to_transaction(&self) -> Box<dyn ITransaction>;
}

impl Candid for TransactionCandid {
    fn to_transaction(&self) -> Box<dyn ITransaction> {
        match self {
            TransactionCandid::QuorumUpdateTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::MemberCreateTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::MemberUpdateNameTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::MemberUpdateRoleTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::MemberRemoveTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::WalletCreateTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::WalletUpdateTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::PolicyCreateTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::PolicyUpdateTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::PolicyRemoveTransactionV(tr) => { Box::new(tr.to_owned()) }
        }
    }
}
