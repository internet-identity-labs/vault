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

impl_basic_for_transaction!(ICRC1CanistersAddTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct ICRC1CanistersAddTransaction {
    common: BasicTransactionFields,
    ledger_canister: Principal,
    index_canister: Option<Principal>,

}

impl ICRC1CanistersAddTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, ledger_canister: Principal, index_canister: Option<Principal>) -> Self {
        ICRC1CanistersAddTransaction {
            common: BasicTransactionFields::new(state, batch_uid,  true),
            ledger_canister,
            index_canister,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ICRC1CanistersAddTransactionRequest {
    ledger_canister: Principal,
    index_canister: Option<Principal>,
    batch_uid: Option<String>,
}

pub struct ICRC1CanistersAddTransactionBuilder {
    request: ICRC1CanistersAddTransactionRequest,
}

impl ICRC1CanistersAddTransactionBuilder {
    pub fn init(request: ICRC1CanistersAddTransactionRequest) -> Self {
        return ICRC1CanistersAddTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for ICRC1CanistersAddTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = ICRC1CanistersAddTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.ledger_canister.clone(),
            self.request.index_canister.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for ICRC1CanistersAddTransaction {
    fn get_block_predicate(&mut self, _: &Box<dyn ITransaction>) -> bool {
        false
    }

    fn define_threshold(&mut self) -> Result<u8, VaultError> {
        Ok(1)
    }

    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        let icrc1 = ICRC1 {
            ledger: self.ledger_canister.clone(),
            index: self.index_canister.clone(),
        };
        state.icrc1_canisters.push(icrc1);
        self.set_state(Executed);
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        TransactionCandid::ICRC1CanistersAddTransactionV(self.clone())
    }
}