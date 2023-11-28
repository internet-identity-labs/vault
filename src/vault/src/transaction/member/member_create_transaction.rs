use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::{ObjectState, TransactionState};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::Member;
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::vault_service::VaultRole;

impl_basic_for_transaction!(MemberCreateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberCreateTransaction {
    pub common: BasicTransactionFields,
    pub member_id: String,
    pub role: VaultRole,
    pub name: String,
}

impl MemberCreateTransaction {
    //TODO private
   pub fn new(state: TransactionState, member_id: String, name: String, role: VaultRole) -> Self {
        MemberCreateTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberCreate, true),
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
            state: ObjectState::Active,
            modified_date: time(),
            created_date: time(),
        };
        if state.members.iter()
            .any(|m| m.member_id.eq(&self.member_id)) {
            self.set_state(TransactionState::Rejected);
            //TODO Enum with errors
            self.common.memo = Some("Member already exists".to_string());
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
    pub member_id: String,
    pub name: String,
    pub role: VaultRole,
}

pub struct MemberCreateTransactionBuilder {
    member_id: String,
    role: VaultRole,
    name: String,
}

impl MemberCreateTransactionBuilder {
    pub fn init(request: MemberCreateTransactionRequest) -> Self {
        return MemberCreateTransactionBuilder {
            member_id: request.member_id,
            name: request.name,
            role: request.role,
        };
    }
}

impl TransactionBuilder for MemberCreateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberCreateTransaction::new(
            state,
            self.member_id.clone(),
            self.name.clone(),
            self.role,
        );
        Box::new(trs)
    }
}


