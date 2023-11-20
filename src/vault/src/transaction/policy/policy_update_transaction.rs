use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(PolicyUpdateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct PolicyUpdateTransaction {
    pub common: BasicTransactionFields,
    uid: String,
    amount_threshold: u64,
    member_threshold: u8,
}

impl PolicyUpdateTransaction {
    fn new(uid: String, amount_threshold: u64,
           member_threshold: u8, state: TransactionState) -> Self {
        PolicyUpdateTransaction {
            common: BasicTransactionFields::new(state, TrType::PolicyUpdate, true),
            uid,
            amount_threshold,
            member_threshold,
        }
    }
}

#[async_trait]
impl ITransaction for PolicyUpdateTransaction {
    async fn execute(&self, mut state: VaultState) -> VaultState {
        let mut p = state.policies.iter()
            .find(|p| p.uid.eq(&self.uid)).unwrap().clone();
        p.amount_threshold = self.amount_threshold.clone();
        p.member_threshold = self.member_threshold.clone();
        p.modified_date = time();
        state.policies.retain(|p| p.uid.eq(&self.uid));
        state.policies.push(p);
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: PolicyUpdateTransaction = self.clone();
        TransactionCandid::PolicyUpdateTransactionV(trs)
    }
}

pub struct PolicyUpdateTransactionBuilder {
    uid: String,
    amount_threshold: u64,
    member_threshold: u8,
}

impl PolicyUpdateTransactionBuilder {
    pub fn init(uid: String, amount_threshold: u64,
                member_threshold: u8) -> Self {
        return PolicyUpdateTransactionBuilder {
            uid,
            amount_threshold,
            member_threshold,
        };
    }
}

impl TransactionBuilder for PolicyUpdateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = PolicyUpdateTransaction::new(
            self.uid.clone(),
            self.amount_threshold.clone(),
            self.member_threshold.clone(),
            state,
        );
        Box::new(trs)
    }
}


