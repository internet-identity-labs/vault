use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::{Currency, TransactionState};
use crate::enums::TransactionState::{Executed, Rejected};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::policy::policy::Policy;
use crate::transaction::transaction::{ITransaction, TransactionCandid, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(PolicyCreateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct PolicyCreateTransaction {
    pub common: BasicTransactionFields,
    uid: String,
    currency: Currency,
    amount_threshold: u64,
    member_threshold: u8,
    wallets: Vec<String>,
}

impl PolicyCreateTransaction {
    fn new(uid: String, currency: Currency, amount_threshold: u64,
           member_threshold: u8, wallets: Vec<String>, state: TransactionState) -> Self {
        PolicyCreateTransaction {
            common: BasicTransactionFields::new(state, TrType::PolicyCreate, true),
            uid,
            currency,
            amount_threshold,
            member_threshold,
            wallets,
        }
    }
}

#[async_trait]
impl ITransaction for PolicyCreateTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        for w in self.wallets.clone() {
            match state.wallets.iter().find(|wal| wal.uid.eq(&w)) {
                None => {
                    self.set_state(Rejected);
                    self.common.memo = Some("Wallet not exists".to_string());
                    return state;
                }
                Some(_) => {}
            }
        }

        match state.policies.iter().find(|policy| policy.amount_threshold.eq(&self.amount_threshold)) {
            None => {}
            Some(_) => {
                self.set_state(Rejected);
                self.common.memo = Some("Threshold already exists".to_string());
                return state;
            }
        }

        let p = Policy::new(self.uid.clone(), self.currency.clone(),
                            self.amount_threshold, self.member_threshold, self.wallets.clone());
        state.policies.push(p);
        self.set_state(Executed);
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: PolicyCreateTransaction = self.clone();
        TransactionCandid::PolicyCreateTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PolicyCreateTransactionRequest {
    currency: Currency,
    amount_threshold: u64,
    member_threshold: u8,
    wallets: Vec<String>,
}

pub struct PolicyCreateTransactionBuilder {
    uid: String,
    currency: Currency,
    amount_threshold: u64,
    member_threshold: u8,
    wallets: Vec<String>,
}

impl PolicyCreateTransactionBuilder {
    pub fn init(request: PolicyCreateTransactionRequest, uid: String) -> Self {
        return PolicyCreateTransactionBuilder {
            uid,
            currency: request.currency,
            amount_threshold: request.amount_threshold,
            member_threshold: request.member_threshold,
            wallets: request.wallets,
        };
    }
}

impl TransactionBuilder for PolicyCreateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = PolicyCreateTransaction::new(
            self.uid.clone(),
            self.currency.clone(),
            self.amount_threshold.clone(),
            self.member_threshold.clone(),
            self.wallets.clone(),
            state,
        );
        Box::new(trs)
    }
}


