use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{get_member_by_id, restore_member};
use crate::transaction::transaction::{ TransactionCandid, TransactionNew, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::basic_transaction::{ Basic};

impl_basic_for_transaction!(MemberUpdateNameTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberUpdateNameTransaction {
    pub common: BasicTransactionFields,
    pub id: String,
    pub name: String,
}

impl MemberUpdateNameTransaction {
    fn new(state: TransactionState, id: String, name: String) -> Self {
        MemberUpdateNameTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberUpdateName),
            id,
            name,
        }
    }
}

impl MemberUpdateNameTransactionBuilder {
    pub fn init(id: String, name: String) -> Self {
        return MemberUpdateNameTransactionBuilder {
            id,
            name,
        };
    }
}

pub struct MemberUpdateNameTransactionBuilder {
    pub id: String,
    pub name: String,
}

impl TransactionBuilder for MemberUpdateNameTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn TransactionNew> {
        let trs = MemberUpdateNameTransaction::new(
            state,
            self.id.clone(),
            self.name.clone(),
        );
        Box::new(trs)
    }
}


#[async_trait]
impl TransactionNew for MemberUpdateNameTransaction {
    async fn execute(&self) {
        let mut m = get_member_by_id(&self.id);
        m.name = self.name.clone();
        restore_member(m); //TODO use ref
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberUpdateNameTransaction = self.clone();
        TransactionCandid::MemberUpdateNameTransaction(trs)
    }
}



