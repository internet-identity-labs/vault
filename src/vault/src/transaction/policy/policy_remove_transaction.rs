use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(PolicyRemoveTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct PolicyRemoveTransaction {
    pub common: BasicTransactionFields,
    uid: String,
}

impl PolicyRemoveTransaction {
    fn new(uid: String, state: TransactionState) -> Self {
        PolicyRemoveTransaction {
            common: BasicTransactionFields::new(state, TrType::PolicyRemove, true),
            uid,
        }
    }
}

pub struct PolicyRemoveTransactionBuilder {
    uid: String,
}

impl PolicyRemoveTransactionBuilder {
    pub fn init(uid: String) -> Self {
        return PolicyRemoveTransactionBuilder {
            uid,
        };
    }
}

impl TransactionBuilder for PolicyRemoveTransaction {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = PolicyRemoveTransaction::new(
            self.uid.clone(),
            state,
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for PolicyRemoveTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        state.policies.retain(|p| p.uid.eq(&self.uid));
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: PolicyRemoveTransaction = self.clone();
        TransactionCandid::PolicyRemoveTransactionV(trs)
    }
}



