use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::transaction::basic_transaction::Basic;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::quorum::quorum::{Quorum, update_quorum};
use crate::transaction::transaction::{TransactionCandid, TransactionNew, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transaction_request_handler::QuorumTransactionRequest;

impl_basic_for_transaction!(QuorumTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct QuorumTransaction {
    pub common: BasicTransactionFields,
    pub transaction_type: TrType,
    pub quorum: u8,
}

impl QuorumTransaction {
    fn new(state: TransactionState, quorum: u8) -> Self {
        QuorumTransaction {
            common: BasicTransactionFields::new(state, TrType::Quorum),
            transaction_type: TrType::Quorum,
            quorum,
        }
    }
}

impl QuorumTransactionBuilder {
    pub fn init(trt: QuorumTransactionRequest) -> Self {
        let trt = trt;
        return QuorumTransactionBuilder {
            quorum: trt.amount,
        };
    }
}

pub struct QuorumTransactionBuilder {
    quorum: u8,
}

impl TransactionBuilder for QuorumTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn TransactionNew> {
        let trs = QuorumTransaction::new(
            state,
            self.quorum.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl TransactionNew for QuorumTransaction {
    async fn execute(&self) {
        let q = Quorum {
            quorum: self.quorum.clone(),
            modified_date: time(),
        };
        update_quorum(q);
    }


    fn to_candid(&self) -> TransactionCandid {
        let trs: QuorumTransaction = self.clone();
        TransactionCandid::QuorumTransaction(trs)
    }
}