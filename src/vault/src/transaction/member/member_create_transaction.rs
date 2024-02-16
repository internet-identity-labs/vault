use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::{TransactionState, VaultRole};
use crate::errors::VaultError::MemberAlreadyExists;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::Member;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberCreateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberCreateTransaction {
    pub common: BasicTransactionFields,
    pub member_id: String,
    pub role: VaultRole,
    pub name: String,
}

impl MemberCreateTransaction {
    pub fn new(state: TransactionState, batch_uid: Option<String>, member_id: String, name: String, role: VaultRole) -> Self {
        MemberCreateTransaction {
            common: BasicTransactionFields::new(state, batch_uid,
                                                true),
            member_id,
            name,
            role,
        }
    }
}

#[async_trait]
impl ITransaction for MemberCreateTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        let member = Member {
            member_id: self.member_id.clone(),
            role: self.role,
            name: self.name.clone(),
            modified_date: time(),
            created_date: time(),
        };
        if state.members.iter()
            .any(|m| m.member_id.eq(&self.member_id)) {
            self.set_state(TransactionState::Rejected);
            self.common.error = Some(MemberAlreadyExists);
            state
        } else {
            state.members.push(member);
            self.set_state(TransactionState::Executed);
            state
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberCreateTransaction = self.clone();
        TransactionCandid::MemberCreateTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberCreateTransactionRequest {
    member_id: String,
    name: String,
    role: VaultRole,
    batch_uid: Option<String>,
}

pub struct MemberCreateTransactionBuilder {
    request: MemberCreateTransactionRequest,
}

impl MemberCreateTransactionBuilder {
    pub fn init(request: MemberCreateTransactionRequest) -> Self {
        return MemberCreateTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for MemberCreateTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberCreateTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.member_id.clone(),
            self.request.name.clone(),
            self.request.role,
        );
        Box::new(trs)
    }
}


