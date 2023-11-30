use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Rejected};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid, TrType};
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
    fn new(state: TransactionState, batch_uid: Option<String>, uid: String, amount_threshold: u64,
           member_threshold: u8, ) -> Self {
        PolicyUpdateTransaction {
            common: BasicTransactionFields::new(state, batch_uid, TrType::PolicyUpdate, true),
            uid,
            amount_threshold,
            member_threshold,
        }
    }
}

#[async_trait]
impl ITransaction for PolicyUpdateTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        match state.policies.iter()
            .find(|p| p.uid.eq(&self.uid)) {
            None => {
                self.set_state(Rejected);
                self.common.memo = Some("No such policy".to_string());
                state
            }
            Some(policy) => {
                match state.policies.iter()
                    .find(|pp| pp.amount_threshold.eq(&self.amount_threshold)) {
                    None => {}
                    Some(_) => {
                        self.set_state(Rejected);
                        self.common.memo = Some("Threshold already exists".to_string());
                        return state;
                    }
                }
                let mut cloned = policy.clone();
                cloned.amount_threshold = self.amount_threshold.clone();
                cloned.member_threshold = self.member_threshold.clone();
                cloned.modified_date = time();
                state.policies.retain(|pp| !pp.uid.eq(&self.uid));
                state.policies.push(cloned);
                self.set_state(Executed);
                state
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: PolicyUpdateTransaction = self.clone();
        TransactionCandid::PolicyUpdateTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PolicyUpdateTransactionRequest {
    uid: String,
    amount_threshold: u64,
    member_threshold: u8,
    batch_uid: Option<String>,
}

pub struct PolicyUpdateTransactionBuilder {
    request: PolicyUpdateTransactionRequest,
}

impl PolicyUpdateTransactionBuilder {
    pub fn init(request: PolicyUpdateTransactionRequest) -> Self {
        return PolicyUpdateTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for PolicyUpdateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = PolicyUpdateTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.uid.clone(),
            self.request.amount_threshold.clone(),
            self.request.member_threshold.clone(),
        );
        Box::new(trs)
    }
}


