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
    pub id: String,
    pub role: VaultRole,
    pub name: String,
}

impl MemberCreateTransaction {
    fn new(state: TransactionState, id: String, name: String, role: VaultRole) -> Self {
        MemberCreateTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberCreate, true),
            id,
            name,
            role,
        }
    }
}

#[async_trait]
impl ITransaction for MemberCreateTransaction {
    async fn execute(&self, mut state: VaultState) -> VaultState {
        let member = Member {
            id: self.id.clone(),
            role: self.role,
            name: self.name.clone(),
            state: ObjectState::Active,
            modified_date: time(),
            created_date: time(),
        };
        state.members.push(member);
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberCreateTransaction = self.clone();
        TransactionCandid::MemberCreateTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberCreateTransactionRequest {
    pub member: String,
    pub name: String,
    pub role: VaultRole,
}

pub struct MemberCreateTransactionBuilder {
    id: String,
    role: VaultRole,
    name: String,
}

impl MemberCreateTransactionBuilder {
    pub fn init(request: MemberCreateTransactionRequest) -> Self {
        return MemberCreateTransactionBuilder {
            id: request.member,
            name: request.name,
            role: request.role,
        };
    }
}

impl TransactionBuilder for MemberCreateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberCreateTransaction::new(
            state,
            self.id.clone(),
            self.name.clone(),
            self.role,
        );
        Box::new(trs)
    }
}


