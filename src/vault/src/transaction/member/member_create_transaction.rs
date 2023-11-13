use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::{ObjectState, TransactionState};
use crate::impl_basic_for_transaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{Member, store_member};
use crate::transaction::transaction::{TransactionCandid, TransactionNew, TrType};
use crate::transaction::basic_transaction::{Basic};
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
            common: BasicTransactionFields::new(state, TrType::MemberCreate),
            id,
            name,
            role,
        }
    }
}

pub struct MemberCreateTransactionBuilder {
    id: String,
    role: VaultRole,
    name: String,
}

impl MemberCreateTransactionBuilder {
    pub fn init(id: String, name: String, role: VaultRole) -> Self {
        return MemberCreateTransactionBuilder {
            id,
            name,
            role,
        };
    }
}

impl TransactionBuilder for MemberCreateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn TransactionNew> {
        let trs = MemberCreateTransaction::new(
            state,
            self.id.clone(),
            self.name.clone(),
            self.role,
        );
        Box::new(trs)
    }
}


#[async_trait]
impl TransactionNew for MemberCreateTransaction {
    async fn execute(&self) {
        store_member(Member {
            id: self.id.clone(),
            role: self.role,
            name: self.name.clone(),
            state: ObjectState::Active,
            modified_date: time(),
            created_date: time(),
        })
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberCreateTransaction = self.clone();
        TransactionCandid::MemberCreateTransaction(trs)
    }
}



