use std::collections::{HashSet};
use ic_cdk::api::time;
use ic_cdk::{call, trap};
use crate::enums::{Action, ObjectState, TransactionState};
use crate::enums::TransactionState::{Approved, Blocked, Pending};

use crate::transaction::quorum::{get_quorum, Quorum, update_quorum};
use crate::transaction::transactions_service::{get_id, get_unfinished_transactions, is_blocked};
use crate::transaction_service::Approve;
use crate::util::caller_to_address;
use candid::CandidType;
use serde::{Deserialize, Serialize};
use crate::transaction::members::{get_member_by_id, Member, restore_member, store_member};
use crate::transaction::transaction::{TransactionCandid, TransactionNew, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;


#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberTransaction {
    pub id: u64,
    pub approves: HashSet<Approve>,
    pub state: TransactionState,
    pub initiator: String,
    pub created_date: u64,
    pub modified_date: u64,
    pub memo: Option<String>,
    pub transaction_type: TrType,
    pub member: Member,
    pub quorum: Quorum,
}

impl MemberTransaction {
    fn new(state: TransactionState, member: Member, transaction_type: TrType, quorum: Quorum) -> Self {
        MemberTransaction {
            id: get_id(),
            approves: Default::default(),
            state,
            initiator: caller_to_address(),
            created_date: time(),
            modified_date: time(),
            memo: None,
            transaction_type,
            member,
            quorum,
        }
    }
}

impl MemberTransactionBuilder {
    pub fn init(tr_type: TrType, member: Member) -> Self {
        return MemberTransactionBuilder {
            member,
            tr_type,
            initial_state: None,
            quorum: None,
        };
    }
}

pub struct MemberTransactionBuilder {
    member: Member,
    tr_type: TrType,
    initial_state: Option<TransactionState>,
    quorum: Option<Quorum>,
}

impl TransactionBuilder for MemberTransactionBuilder {
    // type OutputType = MemberTransaction;

    fn define_initial_state(&mut self) {
        if is_blocked(|tr| {
            get_block_predicate(tr)
        }) {
            self.initial_state = Some(Blocked)
        } else { self.initial_state = Some(Pending); }
    }

    fn define_initial_policy(&mut self) {
        let quorum = get_quorum();
        self.quorum = Some(quorum)
    }

    fn build(mut self) -> Box<dyn TransactionNew> {
        self.define_initial_state();
        self.define_initial_policy();
        let trs = MemberTransaction::new(
            self.initial_state.expect("Define state"),
            self.member,
            self.tr_type,
            self.quorum.expect("Define policy"),
        );
        Box::new(trs)
    }
}


impl TransactionNew for MemberTransaction {
    fn execute(&self) {
        match self.transaction_type {
            TrType::MemberCreate => {
                store_member(self.member.clone())
            }
            TrType::MemberUpdateName | TrType::MemberUpdateRole | TrType::MemberArchive | TrType::MemberUnArchive => {
                restore_member(self.member.clone())
            }
            _ => {
                trap("Incorrect type")
            }
        }
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
            get_block_predicate(tr)
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
        let a: MemberTransaction = self.clone();
        TransactionCandid::MemberTransaction(a)
    }

    fn clone_self(&self) -> Box<dyn TransactionNew> {
        Box::new(self.clone())
    }
}


fn get_block_predicate(tr: &Box<dyn TransactionNew>) -> bool {
    return match tr.get_type() {
        TrType::Quorum => true,
        TrType::MemberArchive => true,
        TrType::MemberUnArchive => true,
        TrType::MemberCreate => true,
        TrType::MemberUpdateName => true,
        TrType::MemberUpdateRole => true,
        _ => false,
    };
}