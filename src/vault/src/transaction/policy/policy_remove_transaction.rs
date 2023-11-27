use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Rejected};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid, TrType};
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

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PolicyRemoveTransactionRequest {
    uid: String,
}

pub struct PolicyRemoveTransactionBuilder {
    uid: String,
}

impl PolicyRemoveTransactionBuilder {
    pub fn init(request: PolicyRemoveTransactionRequest) -> Self {
        return PolicyRemoveTransactionBuilder {
            uid: request.uid,
        };
    }
}

impl TransactionBuilder for PolicyRemoveTransactionBuilder {
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
        match state.policies.iter().find(|p| p.uid.eq(&self.uid)) {
            None => {
                self.set_state(Rejected);
                state
            }
            Some(_) => {
                state.policies.retain(|p| p.uid.eq(&self.uid));
                self.set_state(Executed);
                state
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: PolicyRemoveTransaction = self.clone();
        TransactionCandid::PolicyRemoveTransactionV(trs)
    }
}



