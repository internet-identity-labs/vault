use std::cell::RefMut;
use std::cmp::Ordering;

use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState::{Approved, Blocked, Pending, Rejected};
use crate::enums::VaultRole;
use crate::state::{get_current_state, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::member::member_create_transaction::MemberCreateTransaction;
use crate::transaction::member::member_remove_transaction::MemberRemoveTransaction;
use crate::transaction::member::member_update_name_transaction::MemberUpdateNameTransaction;
use crate::transaction::member::member_update_role_transaction::MemberUpdateRoleTransaction;
use crate::transaction::policy::policy_create_transaction::PolicyCreateTransaction;
use crate::transaction::policy::policy_remove_transaction::PolicyRemoveTransaction;
use crate::transaction::policy::policy_update_transaction::PolicyUpdateTransaction;
use crate::transaction::transaction_approve_handler::Approve;
use crate::transaction::transaction_service::is_blocked;
use crate::transaction::transfer::top_up_transaction::TopUpTransaction;
use crate::transaction::transfer::transfer_transaction::TransferTransaction;
use crate::transaction::upgrade::upgrade_transaction::VersionUpgradeTransaction;
use crate::transaction::vault::quorum::get_quorum;
use crate::transaction::vault::quorum_transaction::QuorumUpdateTransaction;
use crate::transaction::vault::vault_naming_transaction::VaultNamingUpdateTransaction;
use crate::transaction::wallet::wallet_create_transaction::WalletCreateTransaction;
use crate::transaction::wallet::wallet_update_name_transaction::WalletUpdateNameTransaction;

#[async_trait]
pub trait ITransaction: BasicTransaction {
    fn define_state(&mut self) {
        if  !is_blocked(|tr| {
            return self.get_block_predicate(tr);
        }) {
            if self.get_state().eq(&Blocked) {
                self.set_state(Pending);
            }

            let threshold_response = self.define_threshold();
            if self.get_state().eq(&Rejected) {
                return;
            }
            let threshold;
            match threshold_response {
                Ok(t) => {
                    threshold = t;
                }
                Err(s) => {
                    self.set_state(Rejected);
                    self.get_common_mut().memo = Some(s);
                    return;
                }
            };

            let approves = self.get_common_ref().approves
                .iter()
                .filter(|a| a.status == Approved)
                .count();

            let rejects = self.get_common_ref().approves
                .iter()
                .filter(|a| a.status == Rejected)
                .count();

            let voting_members = get_current_state().members
                .iter()
                .filter(|a| self.get_accepted_roles().contains(&a.role))
                .count();

            if threshold <= approves as u8 {
                self.set_state(Approved)
            } else if voting_members - rejects < threshold as usize {
                self.set_state(Rejected)
            }
        }
    }

    fn get_block_predicate(&mut self, tr: &Box<dyn ITransaction>) -> bool {
        if tr.get_id() >= self.get_id() {
            return false;
        }
        if get_vault_state_block_predicate(tr) {
            return true;
        }
        false
    }

    fn define_threshold(&mut self) -> Result<u8, String> {
        match self.get_threshold() {
            None => {
                let t = get_quorum().quorum;
                self.set_threshold(t);
                Ok(t)
            }
            Some(t) => {
                Ok(t)
            }
        }
    }

    async fn execute(&mut self, state: VaultState) -> VaultState;

    fn handle_approve(&mut self, approve: Approve) {
        self.store_approve(approve);
        self.define_state();
    }

    fn get_accepted_roles(&self) -> Vec<VaultRole> {
        return if self.is_vault_state() {
            vec![VaultRole::Admin]
        } else { vec![VaultRole::Admin, VaultRole::Member] };
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
    VaultNamingUpdate,
    Transfer,
    TopUp,
    VersionUpgrade,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum TransactionCandid {
    QuorumUpdateTransactionV(QuorumUpdateTransaction),
    VaultNamingUpdateTransactionV(VaultNamingUpdateTransaction),
    MemberCreateTransactionV(MemberCreateTransaction),
    MemberUpdateNameTransactionV(MemberUpdateNameTransaction),
    MemberUpdateRoleTransactionV(MemberUpdateRoleTransaction),
    MemberRemoveTransactionV(MemberRemoveTransaction),
    WalletCreateTransactionV(WalletCreateTransaction),
    WalletUpdateNameTransactionV(WalletUpdateNameTransaction),
    PolicyCreateTransactionV(PolicyCreateTransaction),
    PolicyUpdateTransactionV(PolicyUpdateTransaction),
    PolicyRemoveTransactionV(PolicyRemoveTransaction),
    TransferTransactionV(TransferTransaction),
    TopUpTransactionV(TopUpTransaction),
    UpgradeTransactionV(VersionUpgradeTransaction),
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
            TransactionCandid::WalletUpdateNameTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::PolicyCreateTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::PolicyUpdateTransactionV(tr) => Box::new(tr.to_owned()),
            TransactionCandid::PolicyRemoveTransactionV(tr) => { Box::new(tr.to_owned()) }
            TransactionCandid::VaultNamingUpdateTransactionV(tr) => { Box::new(tr.to_owned()) }
            TransactionCandid::TransferTransactionV(tr) => { Box::new(tr.to_owned()) }
            TransactionCandid::TopUpTransactionV(tr) => { Box::new(tr.to_owned()) }
            TransactionCandid::UpgradeTransactionV(tr) => { Box::new(tr.to_owned()) }
        }
    }
}


pub fn get_vault_state_block_predicate(tr: &Box<dyn ITransaction>) -> bool {
    return tr.get_common_ref().is_vault_state;
}