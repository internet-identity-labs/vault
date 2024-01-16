use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transaction_service::{get_unfinished_transactions, restore_transaction};

impl_basic_for_transaction!(PurgeTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct PurgeTransaction {
    common: BasicTransactionFields,
}

impl PurgeTransaction {
    fn new(state: TransactionState) -> Self {
        PurgeTransaction {
            common: BasicTransactionFields::new(state, None,  false),
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PurgeTransactionRequest {}

pub struct PurgeTransactionBuilder {
    request: PurgeTransactionRequest,
}

impl PurgeTransactionBuilder {
    pub fn init(request: PurgeTransactionRequest) -> Self {
        return PurgeTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for PurgeTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = PurgeTransaction::new(
            state,
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for PurgeTransaction {

    fn get_block_predicate(&mut self, _: &Box<dyn ITransaction>) -> bool {
        false
    }

    async fn execute(&mut self, state: VaultState) -> VaultState {
        get_unfinished_transactions()
            .into_iter()
            .for_each(|mut tr| {
                tr.set_state(TransactionState::Purged);
                restore_transaction(tr);
            });
        self.set_state(TransactionState::Executed);
        state
    }


    fn to_candid(&self) -> TransactionCandid {
        TransactionCandid::PurgeTransactionV(self.clone())
    }
}