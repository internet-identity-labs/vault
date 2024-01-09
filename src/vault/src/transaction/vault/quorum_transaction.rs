use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Rejected};
use crate::enums::VaultRole::Admin;
use crate::errors::VaultError::QuorumNotReached;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::vault::quorum::Quorum;
use crate::transaction::transaction::{ITransaction, TransactionCandid, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(QuorumUpdateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct QuorumUpdateTransaction {
    common: BasicTransactionFields,
    quorum: u8,
}

impl QuorumUpdateTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, quorum: u8) -> Self {
        QuorumUpdateTransaction {
            common: BasicTransactionFields::new(state, batch_uid, TrType::QuorumUpdate, true),
            quorum,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct
QuorumUpdateTransactionRequest {
    quorum: u8,
    batch_uid: Option<String>,
}

pub struct QuorumUpdateTransactionBuilder {
    request: QuorumUpdateTransactionRequest,
}

impl QuorumUpdateTransactionBuilder {
    pub fn init(request: QuorumUpdateTransactionRequest) -> Self {
        return QuorumUpdateTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for QuorumUpdateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = QuorumUpdateTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.quorum.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for QuorumUpdateTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        if state.members.iter()
            .filter(|m| m.role.eq(&Admin))
            .count() < self.quorum as usize {
            self.set_state(Rejected);
            self.common.error = Some(QuorumNotReached);
            state
        } else {
            let q = Quorum {
                quorum: self.quorum.clone(),
                modified_date: time(),
            };
            state.quorum = q;
            self.set_state(Executed);
            state
        }
    }


    fn to_candid(&self) -> TransactionCandid {
        TransactionCandid::QuorumUpdateTransactionV(self.clone())
    }
}