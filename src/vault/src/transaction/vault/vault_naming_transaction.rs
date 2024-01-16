use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::Executed;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(VaultNamingUpdateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct VaultNamingUpdateTransaction {
    common: BasicTransactionFields,
    name: Option<String>,
    description: Option<String>,
}

impl VaultNamingUpdateTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, name: Option<String>, description: Option<String>) -> Self {
        VaultNamingUpdateTransaction {
            common: BasicTransactionFields::new(state, batch_uid,  true),
            name,
            description,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultNamingUpdateTransactionRequest {
    name: Option<String>,
    description: Option<String>,
    batch_uid: Option<String>,
}

pub struct VaultNamingUpdateTransactionBuilder {
    request: VaultNamingUpdateTransactionRequest,
}

impl VaultNamingUpdateTransactionBuilder {
    pub fn init(request: VaultNamingUpdateTransactionRequest) -> Self {
        return VaultNamingUpdateTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for VaultNamingUpdateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = VaultNamingUpdateTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.name.clone(),
            self.request.description.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for VaultNamingUpdateTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        state.name = self.name.clone();
        state.description = self.description.clone();
        self.set_state(Executed);
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        TransactionCandid::VaultNamingUpdateTransactionV(self.clone())
    }
}