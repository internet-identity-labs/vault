use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::{Currency, TransactionState};
use crate::enums::TransactionState::{Executed, Rejected};
use crate::errors::VaultError::{ThresholdAlreadyExists, UIDAlreadyExists, WalletNotExists};
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
    common: BasicTransactionFields,
    uid: String,
    currency: Currency,
    amount_threshold: u64,
    member_threshold: u8,
    wallets: Vec<String>,
}

impl PolicyCreateTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, uid: String, currency: Currency, amount_threshold: u64,
           member_threshold: u8, wallets: Vec<String>, ) -> Self {
        PolicyCreateTransaction {
            common: BasicTransactionFields::new(state, batch_uid, TrType::PolicyCreate, true),
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
        if state.policies.iter().find(|p| p.uid.eq(&self.uid)).is_some() {
            self.set_state(Rejected);
            self.common.error = Some(UIDAlreadyExists);
            return state;
        }

        for w in self.wallets.clone() {
            match state.wallets.iter().find(|wal| wal.uid.eq(&w)) {
                None => {
                    self.set_state(Rejected);
                    self.common.error = Some(WalletNotExists);
                    return state;
                }
                Some(_) => {}
            }
        }

        match state.policies.iter()
            .filter(|policy| {
                for w in policy.wallets.clone() {
                    if self.wallets.contains(&w) {
                        return true;
                    }
                }
                return false;
            })
            .find(|policy| policy.amount_threshold.eq(&self.amount_threshold)) {
            None => {}
            Some(_) => {
                self.set_state(Rejected);
                self.common.error = Some(ThresholdAlreadyExists);
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
    uid: String,
    currency: Currency,
    amount_threshold: u64,
    member_threshold: u8,
    wallets: Vec<String>,
    batch_uid: Option<String>,
}

pub struct PolicyCreateTransactionBuilder {
    request: PolicyCreateTransactionRequest,
}

impl PolicyCreateTransactionBuilder {
    pub fn init(request: PolicyCreateTransactionRequest) -> Self {
        return PolicyCreateTransactionBuilder {
            request,
        };
    }
}

impl TransactionBuilder for PolicyCreateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = PolicyCreateTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.uid.clone(),
            self.request.currency.clone(),
            self.request.amount_threshold.clone(),
            self.request.member_threshold.clone(),
            self.request.wallets.clone(),
        );
        Box::new(trs)
    }
}


