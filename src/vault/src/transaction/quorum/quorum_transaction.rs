use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Rejected};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::quorum::quorum::Quorum;
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::vault_service::VaultRole::Admin;

impl_basic_for_transaction!(QuorumUpdateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct QuorumUpdateTransaction {
    pub common: BasicTransactionFields,
    pub transaction_type: TrType,
    pub quorum: u8,
}

impl QuorumUpdateTransaction {
    fn new(state: TransactionState, quorum: u8) -> Self {
        QuorumUpdateTransaction {
            common: BasicTransactionFields::new(state, TrType::QuorumUpdate, true),
            transaction_type: TrType::QuorumUpdate,
            quorum,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct
QuorumUpdateTransactionRequest {
    pub quorum: u8,
}

pub struct QuorumUpdateTransactionBuilder {
    quorum: u8,
}

impl QuorumUpdateTransactionBuilder {
    pub fn init(request: QuorumUpdateTransactionRequest) -> Self {
        return QuorumUpdateTransactionBuilder {
            quorum: request.quorum
        };
    }
}

impl TransactionBuilder for QuorumUpdateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = QuorumUpdateTransaction::new(
            state,
            self.quorum.clone(),
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
            let aaa = state.members.iter()
                .filter(|m| m.role.eq(&Admin))
                .count();
            self.set_state(Rejected);
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