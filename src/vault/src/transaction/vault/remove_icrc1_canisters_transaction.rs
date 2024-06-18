use async_trait::async_trait;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::Executed;
use crate::errors::VaultError;
use crate::impl_basic_for_transaction;
use crate::state::{ICRC1, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(ICRC1CanistersRemoveTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct ICRC1CanistersRemoveTransaction {
    common: BasicTransactionFields,
    ledger_canister: Principal,

}

impl ICRC1CanistersRemoveTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, ledger_canister: Principal) -> Self {
        ICRC1CanistersRemoveTransaction {
            common: BasicTransactionFields::new(state, batch_uid,  true),
            ledger_canister,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ICRC1CanistersRemoveTransactionRequest {
    ledger_canister: Principal,
    batch_uid: Option<String>,
}

pub struct ICRC1CanistersRemoveTransactionBuilder {
    request: ICRC1CanistersRemoveTransactionRequest,
}

impl ICRC1CanistersRemoveTransactionBuilder {
    pub fn init(request: ICRC1CanistersRemoveTransactionRequest) -> Self {
        return ICRC1CanistersRemoveTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for ICRC1CanistersRemoveTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = ICRC1CanistersRemoveTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.ledger_canister.clone()
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for ICRC1CanistersRemoveTransaction {
    fn get_block_predicate(&mut self, _: &Box<dyn ITransaction>) -> bool {
        false
    }

    fn define_threshold(&mut self) -> Result<u8, VaultError> {
        Ok(1)
    }

    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        state.icrc1_canisters.retain(|icrc1| icrc1.ledger != self.ledger_canister);
        self.set_state(Executed);
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        TransactionCandid::ICRC1CanistersRemoveTransactionV(self.clone())
    }
}