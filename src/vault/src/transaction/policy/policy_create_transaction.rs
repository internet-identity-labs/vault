use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::{Currency, TransactionState};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::policy::policy::Policy;
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
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
    async fn execute(&self, mut state: VaultState) -> VaultState {
        let p = Policy::new(self.uid.clone(), self.currency.clone(),
                            self.amount_threshold, self.member_threshold, self.wallets.clone());
        state.policies.push(p);
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: PolicyCreateTransaction = self.clone();
        TransactionCandid::PolicyCreateTransactionV(trs)
    }
}

pub struct PolicyCreateTransactionBuilder {
    uid: String,
    currency: Currency,
    amount_threshold: u64,
    member_threshold: u8,
    wallets: Vec<String>,
}

impl PolicyCreateTransactionBuilder {
    pub fn init(uid: String, currency: Currency, amount_threshold: u64,
                member_threshold: u8, wallets: Vec<String>) -> Self {
        return PolicyCreateTransactionBuilder {
            uid,
            currency,
            amount_threshold,
            member_threshold,
            wallets,
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


