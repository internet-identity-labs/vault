use std::collections::{HashSet};
use ic_cdk::api::time;
use ic_cdk::{call, trap};
use crate::enums::TransactionState;
use crate::enums::TransactionState::{Approved, Blocked, Pending};

use crate::transaction::quorum::{get_quorum, Quorum, update_quorum};
use crate::transaction::transaction::{QuorumTransactionRequest, TransactionNew, TransactionBuilder, TrType, TransactionCandid};
use crate::transaction::transactions_service::{get_id, get_unfinished_transactions, is_blocked};
use crate::transaction_service::Approve;
use crate::util::caller_to_address;
use candid::CandidType;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct QuorumTransaction {
    pub id: u64,
    pub approves: HashSet<Approve>,
    pub state: TransactionState,
    pub initiator: String,
    pub created_date: u64,
    pub modified_date: u64,
    pub memo: Option<String>,
    pub transaction_type: TrType,
    pub new_quorum: Quorum,
    pub quorum: Quorum,
}

impl QuorumTransaction {
    fn new(state: TransactionState, new_quorum: Quorum, quorum: Quorum) -> Self {
        QuorumTransaction {
            id: get_id(),
            approves: Default::default(),
            state,
            initiator: caller_to_address(),
            created_date: time(),
            modified_date: time(),
            memo: None,
            transaction_type: TrType::Quorum,
            new_quorum,
            quorum,
        }
    }
}

impl QuorumTransactionBuilder {
    pub fn init(trt: QuorumTransactionRequest) -> Self {
        let trt = trt;
        let initial_state = None;
        let quorum = None;
        return QuorumTransactionBuilder {
            trt,
            initial_state,
            quorum,
        };
    }
}

pub struct QuorumTransactionBuilder {
    trt: QuorumTransactionRequest,
    initial_state: Option<TransactionState>,
    quorum: Option<Quorum>,
}

impl TransactionBuilder for QuorumTransactionBuilder {
    type OutputType = QuorumTransaction;

    fn define_initial_state(&mut self) {
        if is_blocked(|tr| {
            match tr.get_type() {
                TrType::Quorum => true,
                _ => false,
            }
        }) {
            self.initial_state = Some(Blocked)
        } else { self.initial_state = Some(Pending); }
    }

    fn define_initial_policy(&mut self) {
        let quorum = get_quorum();
        self.quorum = Some(quorum)
    }

    fn build(self) -> Self::OutputType {
        QuorumTransaction::new(
            self.initial_state.expect("Define state"),
            Quorum { quorum: self.trt.amount, modified_date: time() },
            self.quorum.expect("Define policy"),
        )
    }
}


impl TransactionNew for QuorumTransaction {
    fn execute(&self) {
        update_quorum(self.new_quorum.clone());
    }

    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_type(&self) -> &TrType {
        &self.transaction_type
    }

    fn get_state(&self) -> &TransactionState {
        &self.state
    }

    fn define_state(&mut self) {
        if !is_blocked(|tr| {
            match tr.get_type() {
                TrType::Quorum => true,
                _ => false,
            }
        }) {
            if self.quorum.quorum <= self.approves.len() as u64 {
                self.state = Approved
            } else {
                self.state = Pending
            }
        }
    }

    fn set_state(&mut self, ts: TransactionState) {
        self.state = ts;
    }

    fn handle_approve(&mut self, approve: Approve) {
        self.approves.insert(approve);
        if !self.state.eq(&Blocked) {
            if self.quorum.quorum <= self.approves.len() as u64 {
                self.state = Approved
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        TransactionCandid::QuorumTransaction(self.clone())
    }

    fn clone_self(&self) -> Box<dyn TransactionNew> {
        Box::new(self.clone())
    }

}